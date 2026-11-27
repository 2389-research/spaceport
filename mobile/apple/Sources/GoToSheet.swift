// ABOUTME: Lists active agents across Herdr workspaces before workspace and pane navigation.
// ABOUTME: Dismisses after Herdr confirms the selected pane and its input target.
import OSLog
import SwiftUI

enum GoToSection: String, Identifiable {
    case agents, workspaces
    var id: Self { self }
}

struct GoToSheet: View {
    @ObservedObject var connection: MobileConnection
    let presentation: UUID
    let initialSection: GoToSection
    var showEmptyAgents = false
    var disconnect: (() -> Void)?
    var home: (() -> Void)?
    var onArrival: (() -> Void)?

    struct Sidebar {
        let hide: () -> Void
        let expand: () -> Void
        let hostDetails: () -> Void
        let rail: Bool
        let expandedWidth: CGFloat?
        let keyboard: Binding<Bool>
        let request: Binding<Jump?>
        let revealAgentsID: UUID?
        let isVisible: Bool
    }

    var sidebar: Sidebar?
    @Environment(\.scenePhase) private var scenePhase
    @Environment(\.dismiss) private var dismiss
    @State private var query = ""
    @State private var browsed: BrowsedWorkspace?
    @State private var sheetSearching = false
    @State private var searchReturnTarget: InputTarget?
    @State private var sheetPage: GoToSection?
    @FocusState private var searching: Bool
    @State private var peek: Peek?
    private struct Peek: Identifiable {
        let row: GoToPane
        let boot: String
        let generation: UUID
        var id: String { row.id }
    }
    @State private var rename: Peek?
    @State private var renameLabel = ""
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
                                     presentation: UUID, generation: UUID, expectedAgent: String?, restoreKeyboard: Bool, historyForward: Bool? = nil) -> Self? {
            guard connection.acceptsInput(from: presentation) else { return nil }
            let started = ContinuousClock.now
            guard let revision = connection.goTo(pane, boot: view.bootId, generation: generation, expectedAgent: expectedAgent, historyForward: historyForward) else { return nil }
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
            .onChange(of: sidebar?.revealAgentsID) { _, id in
                if id != nil { selectPage(.agents) }
            }
            .onChange(of: searching) { _, active in
                guard let sidebar else { return }
                if active {
                    if sidebar.keyboard.wrappedValue {
                        searchReturnTarget = connection.view?.focusedPaneTarget
                        sidebar.keyboard.wrappedValue = false
                    }
                } else {
                    let target = searchReturnTarget
                    searchReturnTarget = nil
                    if let target, target == connection.view?.focusedPaneTarget, request == nil,
                       scenePhase == .active, connection.acceptsInput(from: presentation) {
                        sidebar.keyboard.wrappedValue = true
                    }
                }
            }
            .onChange(of: scenePhase) { _, phase in
                if phase == .background { searchReturnTarget = nil }
            }
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
            .alert("Rename pane", isPresented: Binding(get: { rename != nil }, set: { if !$0 { rename = nil } }), presenting: rename) { item in
                TextField("Pane name", text: $renameLabel)
                Button("Rename") { apply(.rename(label: renameLabel), to: item) }
                Button("Cancel", role: .cancel) {}
            }
            .alert("Go To", isPresented: Binding(get: { issue != nil }, set: { if !$0 { issue = nil } })) {
                Button("OK") { issue = nil }
            } message: { Text(issue ?? "") }
    }

    private var agentSections: [AgentRail.Section] {
        guard let view = connection.view else { return [] }
        return AgentRail.groups(GoToPane.agents(workspaces: view.workspaces, tabs: view.tabs, panes: view.panes))
    }

    @ViewBuilder private var picker: some View {
        let workspaces = selectedPage == .workspaces ? connection.view.map {
            WorkspacePicker(workspaces: $0.workspaces, tabs: $0.tabs, panes: $0.panes)
        } : nil
        if let sidebar {
            let sections = agentSections
            if sidebar.rail {
                let view = connection.view
                let token = connection.generation
                AgentRail(sections: sections,
                          focused: view?.focusedPane, expand: sidebar.expand) { row in
                    guard let view else { return }
                    jump(to: row, view: view, token: token, expectedAgent: row.pane.agentName)
                }
                .frame(width: SessionLayout.Sidebar.rail.width)
                .transition(.opacity)
            } else {
                VStack(spacing: 0) {
                    sidebarHeader(sidebar)
                    pagePicker
                    if let detail = browsed, selectedPage == .workspaces {
                        HStack {
                            browseBackButton
                            Spacer(minLength: 4)
                        }.padding(.horizontal, 10)
                        if !searching {
                            VStack(alignment: .leading, spacing: 2) {
                                Text(verbatim: workspaceTitle(detail)).font(.title3.bold()).lineLimit(2)
                                counts(for: detail, in: workspaces)
                            }.frame(maxWidth: .infinity, alignment: .leading).padding(.horizontal, 14)
                        }
                    }
                    sidebarSearch
                    choices(sections, workspaces: workspaces).listStyle(.plain).scrollContentBackground(.hidden)
                    sidebarFooter(sidebar, sections: sections, workspaces: workspaces)
                }
                // Keep row labels at their reading width as the outer column clips them.
                .frame(width: sidebar.expandedWidth)
                .background(SessionChrome.sidebar)
                .transition(.opacity)
            }
        } else {
            NavigationStack { sheetContent(workspaces: workspaces) }
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
            TextField(searchPrompt, text: $query)
                .focused($searching).font(.subheadline)
                .autocorrectionDisabled().textInputAutocapitalization(.never)
                .accessibilityLabel(Text(verbatim: searchPrompt)).accessibilityIdentifier("session.sidebar.search")
                .submitLabel(.search)
                .overlay {
                    // Capture the terminal target before UIKit transfers first responder to search.
                    if let sidebar, sidebar.keyboard.wrappedValue, !searching {
                        Button {
                            searchReturnTarget = connection.view?.focusedPaneTarget
                            sidebar.keyboard.wrappedValue = false
                            searching = true
                        } label: { Color.clear.contentShape(Rectangle()) }
                        .buttonStyle(.plain).accessibilityLabel(Text(verbatim: searchPrompt))
                    }
                }
            if searching {
                Button {
                    query = ""
                    searching = false
                } label: {
                    Image(systemName: "xmark.circle.fill").frame(width: 44, height: 44)
                }.accessibilityLabel("Cancel search")
            }
        }
        .padding(.leading, 9).frame(height: 38)
        .background(SessionChrome.field, in: RoundedRectangle(cornerRadius: 10))
        .padding(.horizontal, 10).padding(.vertical, 3)
    }

    private func sidebarFooter(_ sidebar: Sidebar, sections: [AgentRail.Section], workspaces: WorkspacePicker?) -> some View {
        VStack(spacing: 0) {
            Divider()
            HStack(spacing: 6) {
                if browsed != nil, let workspaces {
                    currentTerminal(in: workspaces)
                } else if connection.view != nil, connection.phase == .ready {
                    Text(verbatim: AgentRail.summary(sections))
                        .font(.caption).monospacedDigit().foregroundStyle(SessionChrome.secondary)
                        .fixedSize(horizontal: false, vertical: true)
                        .accessibilityIdentifier("session.agentCounts")
                }
                Spacer()
                Button(action: sidebar.hostDetails) {
                    Image(systemName: "info.circle").frame(width: 44, height: 44).contentShape(Rectangle())
                }.accessibilityLabel("Host details").accessibilityIdentifier("session.hostDetails")
            }.padding(.leading, 14).padding(.trailing, 4)
        }.buttonStyle(.plain)
    }

    private var pagePicker: some View {
        Picker("Browse", selection: Binding(get: { selectedPage }, set: { page in
            guard page != selectedPage else { return }
            selectPage(page)
        })) {
            Text("Agents").tag(GoToSection.agents)
            Text("Workspaces").tag(GoToSection.workspaces)
        }
        .pickerStyle(.segmented).padding(.horizontal, sidebar == nil ? 16 : 10).padding(.vertical, 4)
        .accessibilityIdentifier("goTo.browse")
    }

    private func selectPage(_ page: GoToSection) {
        clearSearch()
        browsed = nil
        sheetPage = page
    }

    private func clearSearch() {
        query = ""
        searching = false
        sheetSearching = false
    }

    private func workspaceTitle(_ detail: BrowsedWorkspace) -> String {
        guard let view = connection.view, view.bootId == detail.boot else { return detail.label }
        return view.workspaces.first { $0.id == detail.id }?.label ?? detail.label
    }

    private var searchPrompt: String {
        guard selectedPage == .workspaces else { return sidebar == nil ? "Search agents and panes" : "Agents, panes, titles" }
        return browsed.map { "Search panes in " + workspaceTitle($0) } ?? "Search workspaces"
    }

    private var browseBackButton: some View {
        Button {
            clearSearch()
            browsed = nil
        } label: {
            Label("Workspaces", systemImage: "chevron.backward")
        }
        .labelStyle(.titleAndIcon)
        .frame(minHeight: 44)
        .accessibilityIdentifier("goTo.workspace.back")
    }

    @ViewBuilder private func counts(for detail: BrowsedWorkspace, in picker: WorkspacePicker?) -> some View {
        if let view = connection.view, let picker, detail.boot == view.bootId,
           let workspace = view.workspaces.first(where: { $0.id == detail.id }) {
            Text(verbatim: WorkspacePickerView.summary(picker.counts(in: workspace)))
                .font(.caption).foregroundStyle(.secondary)
        }
    }

    private func currentTerminal(in picker: WorkspacePicker) -> some View {
        let view = connection.view
        let pane = picker.current(workspaceID: view?.focusedWorkspace, tabID: view?.focusedTab, paneID: view?.focusedPane)
        let path = pane.map { row in
            let name = AgentSelectorLabel.displayTitle(row.pane.terminalTitle, address: connection.profile.address) ?? row.pane.label
            return "\(row.location) › \(name)"
        } ?? "Unavailable"
        return VStack(alignment: .leading, spacing: 1) {
            Text("Current terminal").font(.caption2).foregroundStyle(.secondary)
            Text(verbatim: path).font(.caption.weight(.medium)).lineLimit(2)
        }
        .accessibilityElement(children: .combine)
        .accessibilityIdentifier("goTo.currentTerminal")
    }

    private func sheetContent(workspaces: WorkspacePicker?) -> some View {
        VStack(spacing: 0) {
            if browsed == nil, !isSearching { pagePicker }
            if let browsed, !sheetSearching {
                counts(for: browsed, in: workspaces)
                    .frame(maxWidth: .infinity, alignment: .leading).padding(.horizontal)
            }
            choices(agentSections, workspaces: workspaces)
            if browsed != nil, let workspaces {
                Divider()
                currentTerminal(in: workspaces)
                    .frame(maxWidth: .infinity, alignment: .leading)
                    .padding(.horizontal).padding(.vertical, 6)
            }
        }
            .searchable(text: $query, isPresented: $sheetSearching, placement: .navigationBarDrawer(displayMode: .always),
                        prompt: Text(verbatim: searchPrompt))
            .navigationDestination(item: $historyTarget) { target in
                TerminalHistoryView(connection: connection, presentation: presentation, target: target, done: { dismiss() })
            }
            .navigationTitle(browsed.map(workspaceTitle) ?? "Go To")
            .navigationBarTitleDisplayMode(.inline)
            .toolbar {
                ToolbarItemGroup(placement: .topBarLeading) {
                    if browsed != nil {
                        browseBackButton
                    } else if let home {
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

    private func choices(_ sections: [AgentRail.Section], workspaces: WorkspacePicker?) -> some View {
        List {
            if let view = connection.view {
                let token = connection.generation
                if let workspaces {
                    WorkspacePickerView(view: view, picker: workspaces, location: browsed, query: query, address: connection.profile.address,
                                        inspect: { item in
                        clearSearch()
                        sheetPage = .workspaces
                        browsed = BrowsedWorkspace(id: item.id, boot: view.bootId, label: item.label)
                    }, choose: { row in jump(to: row, view: view, token: token) }, menu: { row in
                        paneMenu(row, in: view, token: token)
                    })
                    .id(browsed?.id)
                } else if isSearching {
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
                    agentGroups(view, token: token, sections: sections)
                }
            }
            if let disconnect, !isSearching, browsed == nil, sidebar == nil || selectedPage == .workspaces {
                Section {
                    #if DEBUG
                    ServicePreviewControls(connection: connection)
                    #endif
                    Button("Copy connection diagnostics", systemImage: "doc.on.doc") {
                        UIPasteboard.general.string = connection.diagnosticReport()
                    }
                    Button("Disconnect", systemImage: "power", role: .destructive) {
                        if connection.presentationID == presentation { disconnect() }
                    }.accessibilityIdentifier("session.disconnect")
                }
            }
        }
        .disabled(request != nil)
    }

    private var selectedPage: GoToSection {
        if let sheetPage { return sheetPage }
        guard initialSection == .agents else { return .workspaces }
        if sidebar == nil, !showEmptyAgents, let view = connection.view, !view.panes.contains(where: { $0.agentName != nil }) { return .workspaces }
        return .agents
    }

    private var isSearching: Bool { query.contains { !$0.isWhitespace } }

    @ViewBuilder private func agentGroups(_ view: MobileView, token: UUID, sections: [AgentRail.Section]) -> some View {
        ForEach(sections) { section in
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
        if sections.isEmpty { Text("No agents in this session.").foregroundStyle(.secondary).listRowBackground(Color.clear) }
    }

    private func destinationRow(_ row: GoToPane, in view: MobileView, token: UUID) -> some View {
        agentRow(row, here: row.isFocused(in: view)) {
            jump(to: row, view: view, token: token, expectedAgent: row.pane.agentName)
        }
        .listRowBackground(sidebar == nil ? nil : Color.clear)
        .listRowSeparator(sidebar == nil ? .automatic : .hidden)
        .listRowInsets(sidebar == nil ? nil : EdgeInsets(top: 1, leading: 8, bottom: 1, trailing: 8))
        .modifier(paneMenu(row, in: view, token: token))
    }

    private func paneMenu(_ row: GoToPane, in view: MobileView, token: UUID) -> PaneContextMenu {
        let item = Peek(row: row, boot: view.bootId, generation: token)
        return PaneContextMenu(offers: view.paneActions, zoomed: row.tab.zoomed,
                               preview: peekView(item), showPeek: { peek = item },
                               edit: { action in apply(action, to: item) }, rename: {
            renameLabel = row.pane.label
            rename = item
        })
    }

    private func apply(_ action: MobilePaneAction, to item: Peek) {
        guard connection.acceptsInput(from: presentation) else { return }
        connection.editPane(item.row, boot: item.boot, generation: item.generation, action: action)
    }

    private func peekView(_ item: Peek) -> AgentPeek {
        AgentPeek(row: item.row, address: connection.profile.address) {
            connection.preview(pane: item.row.id, boot: item.boot)
        }
    }

    private func jump(to pane: GoToPane, view: MobileView, token: UUID, expectedAgent: String? = nil) {
        guard connection.acceptsInput(from: presentation), request == nil else { return }
        let name = AgentSelectorLabel.displayTitle(pane.pane.terminalTitle, address: connection.profile.address) ?? pane.pane.label
        guard token == connection.generation, view.bootId == connection.view?.bootId,
              connection.view?.panes.contains(where: { $0.id == pane.id && $0.tabId == pane.tab.id }) == true else {
            issue = "“\(name)” is no longer available."
            return
        }
        let restoreKeyboard = sidebar?.keyboard.wrappedValue == true ||
            (searchReturnTarget != nil && searchReturnTarget == connection.view?.focusedPaneTarget)
        guard let jump = Jump.begin(to: pane, view: view, connection: connection, presentation: presentation,
                                    generation: token, expectedAgent: expectedAgent, restoreKeyboard: restoreKeyboard) else {
            issue = connection.message ?? "The selected terminal changed."
            connection.message = nil
            return
        }
        searchReturnTarget = nil
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
            let name = AgentSelectorLabel.displayTitle(request.pane.pane.terminalTitle, address: connection.profile.address) ?? request.pane.pane.label
            let stillPresent = view.panes.contains { $0.id == request.pane.id && $0.tabId == request.pane.tab.id }
            let failure = stillPresent ? (view.error ?? "Herdr could not confirm focus on “\(name)”.") : "“\(name)” is no longer available."
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
                                       paneTitle: row.pane.displayPaneTitle, isAgent: row.pane.agentName != nil, address: connection.profile.address)
        if sidebar != nil {
            Button(action: action) {
                HStack(spacing: 6) {
                    if row.pane.agentName != nil {
                        AgentRowBadge(identity: row.pane.agentIdentity, activity: activity,
                                      background: here ? SessionChrome.selected : SessionChrome.sidebar,
                                      statusColor: activity?.sessionColor ?? .secondary)
                    } else if let activity { activity.sessionDot }
                    VStack(alignment: .leading, spacing: 3) {
                        Text(label.heading).font(.subheadline.weight(.semibold)).lineLimit(1)
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
            .accessibilityValue(Text(verbatim: [String(label.heading.characters), path, status, here ? "Here" : nil].compactMap { $0 }.joined(separator: ", ")))
            .accessibilityAddTraits(here ? .isSelected : [])
            .accessibilityIdentifier("goTo.\(role).\(row.id)")
        } else {
            Button(action: action) {
                HStack(spacing: 10) {
                    if row.pane.agentName != nil {
                        AgentRowBadge(identity: row.pane.agentIdentity, activity: activity,
                                      background: Color(uiColor: .secondarySystemGroupedBackground),
                                      statusColor: activity?.color ?? .secondary)
                    }
                    label
                    Spacer(minLength: 4)
                    VStack(alignment: .trailing, spacing: 3) {
                        HStack(spacing: 4) {
                            if isSearching, let activity {
                                Text(activity.title).foregroundStyle(activity.textColor)
                            } else if let age = row.sidebarAge, activity == .idle {
                                Text(verbatim: age).monospacedDigit()
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
            .accessibilityValue(Text(verbatim: [String(label.heading.characters), path, status, here ? "Here" : nil].compactMap { $0 }.joined(separator: ", ")))
            .accessibilityIdentifier("goTo.\(role).\(row.id)")
        }
    }

}

struct PaneContextMenu: ViewModifier {
    let offers: MobilePaneActions?
    let zoomed: Bool
    let preview: AgentPeek
    let showPeek: () -> Void
    let edit: (MobilePaneAction) -> Void
    let rename: () -> Void

    func body(content: Content) -> some View {
        content.contextMenu {
            Button("Peek", systemImage: "eye", action: showPeek)
            if let offers {
                if offers.split {
                    Button("Split right", systemImage: "rectangle.split.2x1") { edit(.splitRight) }
                    Button("Split down", systemImage: "rectangle.split.1x2") { edit(.splitDown) }
                }
                if offers.zoom {
                    Button(zoomed ? "Restore panes" : "Zoom pane", systemImage: "arrow.up.left.and.arrow.down.right") {
                        edit(.zoom(enabled: !zoomed))
                    }
                }
                if offers.rename { Button("Rename pane", systemImage: "pencil", action: rename) }
            }
        } preview: { preview.frame(idealWidth: 320) }
        .accessibilityAction(named: "Peek") { showPeek() }
    }
}
