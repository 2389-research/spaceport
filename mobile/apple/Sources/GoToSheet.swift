// ABOUTME: Lists active agents across Herdr workspaces before workspace and pane navigation.
// ABOUTME: Dismisses after Herdr confirms the selected pane and its input target.
import OSLog
import SwiftUI

enum GoToSection: String, Identifiable {
    case agents, workspaces, tabs, panes
    var id: Self { self }
}

struct GoToSheet: View {
    @ObservedObject var connection: MobileConnection
    let presentation: UUID
    let initialSection: GoToSection
    var disconnect: (() -> Void)?
    var home: (() -> Void)?
    var onArrival: (() -> Void)?

    struct Sidebar {
        let hide: () -> Void
        let expand: () -> Void
        let hostDetails: () -> Void
        let rail: Bool
        let keyboard: Binding<Bool>
        let request: Binding<Jump?>
        let isVisible: Bool
    }

    var sidebar: Sidebar?
    @Environment(\.scenePhase) private var scenePhase
    @Environment(\.dismiss) private var dismiss
    @State private var query = ""
    @State private var workspacesExpanded = false
    @FocusState private var searching: Bool
    @State private var peek: Peek?
    private struct Peek: Identifiable {
        let row: GoToPane
        let boot: String
        var id: String { row.id }
    }
    @State private var historyTarget: InputTarget?
    @State private var sheetRequest: Jump?
    private var request: Jump? {
        get { if let sidebar { return sidebar.request.wrappedValue }; return sheetRequest }
        nonmutating set {
            if let sidebar { sidebar.request.wrappedValue = newValue } else { sheetRequest = newValue }
        }
    }
    @State private var issue: String?
    @State private var showsProgress = false

    struct Jump {
        let started: ContinuousClock.Instant
        let pane: GoToPane
        let expectedAgent: String?
        let boot: String
        let generation: UUID
        let revision: UInt64
        let restoreKeyboard: Bool

        @MainActor static func begin(to pane: GoToPane, view: MobileView, connection: MobileConnection,
                                     presentation: UUID, generation: UUID, expectedAgent: String?, restoreKeyboard: Bool) -> Self? {
            guard connection.acceptsInput(from: presentation) else { return nil }
            let started = ContinuousClock.now
            guard let revision = connection.goTo(pane, boot: view.bootId, generation: generation, expectedAgent: expectedAgent) else { return nil }
            return Jump(started: started, pane: pane, expectedAgent: expectedAgent, boot: view.bootId,
                        generation: generation, revision: revision, restoreKeyboard: restoreKeyboard)
        }
    }

    var body: some View {
        VStack(spacing: 0) { picker }
            .task(id: request?.revision) {
                showsProgress = false
                guard request != nil else { return }
                do { try await Task.sleep(for: .milliseconds(180)) } catch { return }
                showsProgress = true
            }
            .onChange(of: request?.revision, initial: true) { _, _ in checkArrival() }
            .onChange(of: connection.view?.version) { _, _ in checkArrival() }
            .onDisappear {
                if connection.presentationID == presentation, let request {
                    self.request = nil
                    connection.cancelGoTo(generation: request.generation)
                }
            }
            .overlay(alignment: .bottom) {
                if showsProgress, let request {
                    ProgressView("Opening \(request.expectedAgent ?? request.pane.pane.label)…")
                        .lineLimit(1).truncationMode(.middle)
                        .padding(8).frame(maxWidth: .infinity).background(.bar)
                }
            }
            .sheet(item: $peek) { item in
                NavigationStack {
                    ScrollView { peekView(item) }
                        .navigationTitle("Pane preview").navigationBarTitleDisplayMode(.inline)
                        .toolbar { ToolbarItem(placement: .confirmationAction) { Button("Done") { peek = nil } } }
                }
                .presentationDetents([.medium, .large])
                .onChange(of: connection.view?.bootId) { _, boot in if boot != item.boot { peek = nil } }
            }
            .alert("Go To", isPresented: Binding(get: { issue != nil }, set: { if !$0 { issue = nil } })) {
                Button("OK") { issue = nil }
            } message: { Text(issue ?? "") }
    }

    @ViewBuilder private var picker: some View {
        if let sidebar {
            if sidebar.rail {
                let view = connection.view
                let token = connection.generation
                AgentRail(sections: view.map { AgentRail.groups(GoToPane.agents(workspaces: $0.workspaces, tabs: $0.tabs, panes: $0.panes)) } ?? [],
                          focused: view?.focusedPane, expand: sidebar.expand) { row in
                    guard let view else { return }
                    jump(to: row, view: view, token: token, expectedAgent: row.pane.agentName)
                }
            } else {
                VStack(spacing: 0) {
                    sidebarHeader(sidebar)
                    sidebarSearch
                    choices.listStyle(.plain).scrollContentBackground(.hidden)
                    sidebarFooter(sidebar)
                }
                .background(SessionChrome.sidebar)
            }
        } else {
            NavigationStack { sheetContent }
                .preferredColorScheme(.dark)
                .presentationCornerRadius(34)
                .presentationDragIndicator(.visible)
        }
    }

    private func sidebarHeader(_ sidebar: Sidebar) -> some View {
        HStack(spacing: 4) {
            Button(action: { home?() }) {
                HStack(spacing: 5) {
                    Text(verbatim: connection.profile.session).font(.subheadline.weight(.semibold))
                        .layoutPriority(1)
                    Image(systemName: "chevron.down").font(.caption2)
                    Text(verbatim: connection.profile.hostname).font(.caption).foregroundStyle(SessionChrome.secondary)
                    Spacer(minLength: 0)
                }.lineLimit(1).frame(minHeight: 44).contentShape(Rectangle())
            }
            .accessibilityLabel("Switch session")
            .accessibilityValue(Text(verbatim: "\(connection.profile.session), \(connection.profile.hostname)"))
            .accessibilityIdentifier("session.home")
            Button(action: sidebar.hide) {
                Image(systemName: "sidebar.leading").frame(width: 44, height: 44).contentShape(Rectangle())
            }
            .accessibilityLabel("Collapse agents sidebar")
            .accessibilityIdentifier("session.sidebar.hide")
        }.buttonStyle(.plain).padding(.leading, 14).padding(.trailing, 4)
    }

    private var sidebarSearch: some View {
        HStack(spacing: 6) {
            Image(systemName: "magnifyingglass").foregroundStyle(SessionChrome.secondary)
            TextField("Agents, panes, titles", text: $query)
                .focused($searching).font(.subheadline)
                .autocorrectionDisabled().textInputAutocapitalization(.never)
                .accessibilityLabel("Search agents and panes").accessibilityIdentifier("session.sidebar.search")
                .onChange(of: searching) { _, active in if active { sidebar?.keyboard.wrappedValue = false } }
            if !query.isEmpty {
                Button { query = "" } label: {
                    Image(systemName: "xmark.circle.fill").frame(width: 44, height: 44)
                }.accessibilityLabel("Clear search")
            }
        }
        .padding(.leading, 9).frame(height: 38)
        .background(SessionChrome.field, in: RoundedRectangle(cornerRadius: 10))
        .padding(.horizontal, 10).padding(.vertical, 3)
    }

    private func sidebarFooter(_ sidebar: Sidebar) -> some View {
        VStack(spacing: 0) {
            Divider()
            HStack(spacing: 6) {
                Circle().fill(connection.phase == .ready ? SessionChrome.connected : SessionChrome.secondary)
                    .frame(width: 8, height: 8)
                Text(connection.phase == .ready
                     ? ["Connected", connection.connectedUsing?.title].compactMap { $0 }.joined(separator: " · ")
                     : "Connecting")
                    .font(.caption).foregroundStyle(SessionChrome.secondary).lineLimit(1)
                Spacer()
                Button(action: sidebar.hostDetails) {
                    Image(systemName: "info.circle").frame(width: 44, height: 44).contentShape(Rectangle())
                }.accessibilityLabel("Host details").accessibilityIdentifier("session.hostDetails")
            }.padding(.leading, 14).padding(.trailing, 4)
        }.buttonStyle(.plain)
    }

    private var sheetContent: some View {
        choices
            .searchable(text: $query, placement: .navigationBarDrawer(displayMode: .always),
                        prompt: "Search agents, titles, workspaces, tabs")
            .navigationDestination(item: $historyTarget) { target in
                TerminalHistoryView(connection: connection, presentation: presentation, target: target, done: { dismiss() })
            }
            .navigationTitle("Go To")
            .navigationBarTitleDisplayMode(.inline)
            .toolbar {
                ToolbarItemGroup(placement: .topBarLeading) {
                    if let home {
                        Button(action: home) { Label("Home", systemImage: "house") }
                            .accessibilityIdentifier("session.home")
                    }
                    Button {
                        historyTarget = connection.view?.inputTarget
                    } label: {
                        Label("Find in Terminal", systemImage: "text.magnifyingglass")
                    }
                    .disabled(request != nil || connection.view?.inputTarget == nil || connection.view?.inputTarget?.popupId != nil)
                    .accessibilityIdentifier("session.find")
                }
                ToolbarItem(placement: .confirmationAction) {
                    Button("Done") { dismiss() }.accessibilityIdentifier("goTo.done")
                }
            }
    }

    private var choices: some View {
        ScrollViewReader { reader in
            List {
                if let view = connection.view {
                    let token = connection.generation
                    if isSearching {
                        let rows = GoToPane.search(query, workspaces: view.workspaces, tabs: view.tabs, panes: view.panes,
                                                   recent: connection.panePreviews.recent)
                        if rows.isEmpty {
                            ContentUnavailableView.search(text: query)
                        } else {
                            Section("Matching panes") {
                                ForEach(rows) { row in destinationRow(row, in: view, token: token) }
                            }
                        }
                    } else {
                        browse(view, token: token)
                    }
                }
                if let disconnect, !isSearching, sidebar == nil || workspacesExpanded {
                    Section {
                        Button("Disconnect", systemImage: "power", role: .destructive) {
                            if connection.presentationID == presentation { disconnect() }
                        }.accessibilityIdentifier("session.disconnect")
                    }
                }
            }
            .disabled(request != nil)
            .onAppear {
                if initialSection != .agents { reader.scrollTo(initialSection, anchor: .top) }
            }
        }
    }

    private var isSearching: Bool { query.contains { !$0.isWhitespace } }

    @ViewBuilder private func browse(_ view: MobileView, token: UUID) -> some View {
        let workspace = view.workspaces.first { $0.id == view.focusedWorkspace }
        let tabs = view.tabs.filter { $0.workspaceId == workspace?.id }
        let tab = tabs.first { $0.id == view.focusedTab }
        let agents = GoToPane.agents(workspaces: view.workspaces, tabs: view.tabs, panes: view.panes)
        if sidebar != nil {
            ForEach(AgentRail.groups(agents)) { section in
                Section {
                    ForEach(section.rows) { row in destinationRow(row, in: view, token: token) }
                } header: {
                    HStack(spacing: 6) {
                        Text(section.kind.title)
                        Text(section.rows.count.formatted()).monospacedDigit().foregroundStyle(SessionChrome.secondary)
                    }.font(.caption.weight(.semibold)).textCase(nil)
                        .foregroundStyle(section.rows.first.flatMap { PaneActivity($0.pane.status) }?.sessionTextColor ?? SessionChrome.secondary)
                        .accessibilityElement(children: .combine)
                }
            }
            if agents.isEmpty { Text("No agents in this session.").foregroundStyle(.secondary).listRowBackground(Color.clear) }
            Button {
                withAnimation(.easeInOut(duration: 0.18)) { workspacesExpanded.toggle() }
            } label: {
                HStack {
                    Text("Workspaces").font(.subheadline.weight(.semibold))
                    Spacer()
                    Text(view.workspaces.count.formatted())
                    Image(systemName: workspacesExpanded ? "chevron.down" : "chevron.right")
                }
                .frame(minHeight: 44).contentShape(Rectangle())
            }
            .buttonStyle(.plain).foregroundStyle(SessionChrome.secondary)
            .accessibilityValue(workspacesExpanded ? "Expanded" : "Collapsed")
            .accessibilityIdentifier("session.sidebar.workspaces")
            .listRowBackground(Color.clear)
        } else {
            Section("Agents") {
                if agents.isEmpty { Text("No agents in this session.").foregroundStyle(.secondary) }
                ForEach(agents) { row in destinationRow(row, in: view, token: token) }
            }.id(GoToSection.agents)
        }
        if sidebar == nil || workspacesExpanded {
            Section("Workspaces") {
                workspaceSegments(view, token: token)
            }.id(GoToSection.workspaces)
            Section("Tabs") {
                ForEach(tabs, id: \.id) { item in
                    choice(item.label, status: item.status, selected: item.id == tab?.id) {
                        guard connection.acceptsInput(from: presentation), token == connection.generation, item.id != tab?.id else { return }
                        sidebar?.keyboard.wrappedValue = false
                        connection.focus(.tab, id: item.id, boot: view.bootId)
                    }.accessibilityIdentifier("goTo.tab.\(item.id)")
                }
            }.id(GoToSection.tabs)
            if let workspace, let tab {
                Section {
                    paneChoices(view, workspace: workspace, tab: tab, token: token)
                } header: { Text("Panes in \(tab.label)") }
                footer: { Text("Switching panes moves focus for every client on this tab.") }
                .id(GoToSection.panes)
            }
        }
    }

    private func destinationRow(_ row: GoToPane, in view: MobileView, token: UUID) -> some View {
        let item = Peek(row: row, boot: view.bootId)
        return agentRow(row, here: row.isFocused(in: view)) {
            jump(to: row, view: view, token: token, expectedAgent: row.pane.agentName)
        }
        .listRowBackground(sidebar == nil ? nil : Color.clear)
        .listRowSeparator(sidebar == nil ? .automatic : .hidden)
        .listRowInsets(sidebar == nil ? nil : EdgeInsets(top: 1, leading: 8, bottom: 1, trailing: 8))
        .contextMenu {
            Button("Peek", systemImage: "eye") { peek = item }
        } preview: {
            peekView(item).frame(idealWidth: 320)
        }
        .accessibilityAction(named: "Peek") { peek = item }
    }

    private func peekView(_ item: Peek) -> some View {
        AgentPeek(row: item.row) {
            connection.preview(pane: item.row.id, boot: item.boot)
        }
    }

    private func jump(to pane: GoToPane, view: MobileView, token: UUID, expectedAgent: String? = nil) {
        guard connection.acceptsInput(from: presentation), request == nil else { return }
        let restoreKeyboard = sidebar?.keyboard.wrappedValue == true
        guard let jump = Jump.begin(to: pane, view: view, connection: connection, presentation: presentation,
                                    generation: token, expectedAgent: expectedAgent, restoreKeyboard: restoreKeyboard) else {
            issue = connection.message ?? "The selected terminal changed."
            connection.message = nil
            return
        }
        searching = false
        sidebar?.keyboard.wrappedValue = false
        request = jump
        checkArrival()
    }

    private func checkArrival() {
        guard let request else { return }
        guard connection.acceptsInput(from: presentation), let view = connection.view,
              view.bootId == request.boot, connection.generation == request.generation else {
            self.request = nil
            if sidebar == nil { dismiss() }
            return
        }
        guard view.version >= request.revision, !view.goToPending else { return }
        self.request = nil
        let agentMatches = request.expectedAgent.map { expected in
            view.panes.contains { $0.id == request.pane.id && $0.agentName == expected }
        } ?? true
        if view.confirmsInput(to: request.pane.id, revision: request.revision), agentMatches, request.pane.isFocused(in: view) {
            #if DEBUG
            let elapsed = request.started.duration(to: .now)
            Logger(subsystem: "ai.2389.spaceport.mobile", category: "Navigation")
                .notice("Go To input ready after \(elapsed.formatted(.units(allowed: [.milliseconds])), privacy: .public)")
            #endif
            onArrival?()
            if let sidebar {
                if request.restoreKeyboard, scenePhase == .active, connection.acceptsInput(from: presentation) {
                    sidebar.keyboard.wrappedValue = true
                }
            } else { dismiss() }
        } else {
            let failure = view.error ?? "The selected pane changed before Herdr confirmed focus."
            if sidebar?.isVisible == false {
                connection.message = failure
            } else {
                connection.message = nil
                issue = failure
            }
        }
    }

    @ViewBuilder private func agentRow(_ row: GoToPane, here: Bool, action: @escaping () -> Void) -> some View {
        let activity = PaneActivity(row.pane.status)
        let role = row.pane.agentName == nil ? "result" : (activity == .blocked ? "needsYou" : "agent")
        let status = activity?.title
        let name = row.pane.agentName.flatMap { $0.isEmpty ? nil : $0 } ?? row.pane.label
        let path = row.location
        let label = AgentSelectorLabel(name: name, terminalTitle: row.pane.terminalTitle,
                                       workspace: row.workspace.label, tab: row.tab.label, font: .body.weight(.medium),
                                       isAgent: row.pane.agentName != nil)
        if sidebar != nil {
            Button(action: action) {
                HStack(spacing: 6) {
                    if row.pane.agentName != nil {
                        AgentRowBadge(identity: row.pane.agentIdentity, activity: activity,
                                      background: here ? SessionChrome.selected : SessionChrome.sidebar,
                                      statusColor: activity?.sessionColor ?? .secondary)
                    } else if let activity { activity.sessionDot }
                    VStack(alignment: .leading, spacing: 3) {
                        Text(verbatim: label.title ?? name).font(.subheadline.weight(.semibold)).lineLimit(1)
                        Text(label.subtitle(includingName: row.pane.agentName != nil)).font(.caption).lineLimit(2)
                    }
                    Spacer(minLength: 2)
                    if let age = row.sidebarAge {
                        Text(verbatim: age).font(.caption).foregroundStyle(SessionChrome.secondary)
                    }
                }
                .padding(.horizontal, 8).padding(.vertical, 6).frame(minHeight: 44)
                .background(here ? SessionChrome.selected : .clear, in: RoundedRectangle(cornerRadius: 10))
                .contentShape(Rectangle())
            }
            .buttonStyle(.plain).accessibilityLabel(Text(verbatim: name))
            .accessibilityValue(Text(verbatim: [label.title, path, status, here ? "Here" : nil].compactMap { $0 }.joined(separator: ", ")))
            .accessibilityAddTraits(here ? .isSelected : [])
            .accessibilityIdentifier("goTo.\(role).\(row.id)")
        } else {
            Button(action: action) {
                HStack(spacing: 10) {
                    if row.pane.agentName != nil {
                        AgentBadge(identity: row.pane.agentIdentity).frame(width: 20, height: 20).accessibilityHidden(true)
                    }
                    label
                    Spacer(minLength: 4)
                    VStack(alignment: .trailing, spacing: 3) {
                        HStack(spacing: 4) {
                            if let activity {
                                activity.agentDot(in: activity.color)
                                Text(activity.title).foregroundStyle(activity.textColor)
                            }
                        }
                        if here { Text("Here").accessibilityIdentifier("goTo.here") }
                    }
                    .font(.footnote).foregroundStyle(.secondary)
                }
                .padding(.vertical, 2).frame(minHeight: 44).contentShape(Rectangle())
            }
            .foregroundStyle(.primary)
            .accessibilityLabel(Text(verbatim: name == row.pane.label ? name : "\(name), \(row.pane.label)"))
            .accessibilityValue(Text(verbatim: [label.title, path, status, here ? "Here" : nil].compactMap { $0 }.joined(separator: ", ")))
            .accessibilityIdentifier("goTo.\(role).\(row.id)")
        }
    }

    private func workspaceSegments(_ view: MobileView, token: UUID) -> some View {
        ScrollView(.horizontal) {
            HStack(spacing: 2) {
                ForEach(view.workspaces, id: \.id) { item in
                    let selected = item.id == view.focusedWorkspace
                    Button {
                        guard connection.acceptsInput(from: presentation), token == connection.generation, !selected else { return }
                        sidebar?.keyboard.wrappedValue = false
                        connection.focus(.workspace, id: item.id, boot: view.bootId)
                    } label: {
                        HStack(spacing: 5) {
                            if let status = PaneActivity(item.status) { status.dot }
                            Text(verbatim: item.label).lineLimit(1)
                        }
                        .font(.subheadline.weight(selected ? .semibold : .regular))
                        .padding(.horizontal, 12).frame(minHeight: 42)
                        .background(selected ? Color(uiColor: .systemBackground) : .clear, in: Capsule())
                    }
                    .buttonStyle(.plain).accessibilityAddTraits(selected ? .isSelected : [])
                    .accessibilityValue(PaneActivity(item.status)?.title ?? "")
                    .accessibilityIdentifier("goTo.workspace.\(item.id)")
                }
            }.padding(2)
        }
        .scrollIndicators(.hidden)
        .background(Color(uiColor: .tertiarySystemFill), in: Capsule())
        .listRowInsets(EdgeInsets()).listRowBackground(Color.clear)
    }

    @ViewBuilder private func paneChoices(_ view: MobileView, workspace: MobileWorkspace, tab: MobileTab, token: UUID) -> some View {
        let panes = view.panes.filter { $0.tabId == tab.id }
        if view.paneMap != nil {
            PaneMapView(view: view) { id in
                if let pane = panes.first(where: { $0.id == id }) {
                    jump(to: GoToPane(workspace: workspace, tab: tab, pane: pane), view: view, token: token)
                }
            }
        }
        ForEach(panes, id: \.id) { pane in
            choice(pane.label, status: pane.status, selected: pane.id == view.focusedPane) {
                jump(to: GoToPane(workspace: workspace, tab: tab, pane: pane), view: view, token: token)
            }.accessibilityIdentifier("goTo.pane.\(pane.id)")
        }
    }

    private func choice(_ label: String, status: String, selected: Bool, action: @escaping () -> Void) -> some View {
        Button(action: action) {
            HStack {
                if let activity = PaneActivity(status) { activity.dot }
                Text(verbatim: label)
                Spacer()
                if let activity = PaneActivity(status) { Text(activity.title).font(.subheadline).foregroundStyle(.secondary) }
                if selected { Image(systemName: "checkmark").font(.body.weight(.semibold)).accessibilityHidden(true) }
            }
        }
        .foregroundStyle(.primary).accessibilityAddTraits(selected ? .isSelected : [])
    }
}
