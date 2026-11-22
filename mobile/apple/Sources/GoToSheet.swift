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
    @Environment(\.dismiss) private var dismiss
    @State private var query = ""
    @State private var peek: Peek?
    private struct Peek: Identifiable {
        let row: GoToPane
        let boot: String
        var id: String { row.id }
    }
    @State private var historyTarget: InputTarget?
    @State private var request: Jump?
    @State private var issue: String?
    @State private var showsProgress = false

    private struct Jump {
        let started: ContinuousClock.Instant
        let pane: GoToPane
        let expectedAgent: String?
        let boot: String
        let generation: UUID
        let revision: UInt64
    }

    var body: some View {
        NavigationStack {
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
                    if let disconnect, !isSearching {
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
            .searchable(text: $query, placement: .navigationBarDrawer(displayMode: .always),
                        prompt: "Search agents, titles, workspaces, tabs")
            .navigationDestination(item: $historyTarget) { target in
                TerminalHistoryView(connection: connection, presentation: presentation, target: target, done: { dismiss() })
            }
            .navigationTitle("Go To")
            .navigationBarTitleDisplayMode(.inline)
            .toolbar {
                ToolbarItem(placement: .topBarLeading) {
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
        .task(id: request?.revision) {
            showsProgress = false
            guard request != nil else { return }
            do { try await Task.sleep(for: .milliseconds(180)) } catch { return }
            showsProgress = true
        }
        .preferredColorScheme(.dark)
        .presentationCornerRadius(34)
        .presentationDragIndicator(.visible)
        .onChange(of: connection.view?.version) { _, _ in checkArrival() }
        .onDisappear { if connection.presentationID == presentation, let request { connection.cancelGoTo(generation: request.generation) } }
    }

    private var isSearching: Bool { query.contains { !$0.isWhitespace } }

    @ViewBuilder private func browse(_ view: MobileView, token: UUID) -> some View {
        let workspace = view.workspaces.first { $0.id == view.focusedWorkspace }
        let tabs = view.tabs.filter { $0.workspaceId == workspace?.id }
        let tab = tabs.first { $0.id == view.focusedTab }
        let agents = GoToPane.agents(workspaces: view.workspaces, tabs: view.tabs, panes: view.panes)
        Section("Agents") {
            if agents.isEmpty { Text("No agents in this session.").foregroundStyle(.secondary) }
            ForEach(agents) { row in
                destinationRow(row, in: view, token: token)
            }
        }.id(GoToSection.agents)
        Section("Workspaces") {
            workspaceSegments(view, token: token)
        }.id(GoToSection.workspaces)
        Section("Tabs") {
            ForEach(tabs, id: \.id) { item in
                choice(item.label, status: item.status, selected: item.id == tab?.id) {
                    guard connection.acceptsInput(from: presentation), token == connection.generation, item.id != tab?.id else { return }
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

    private func destinationRow(_ row: GoToPane, in view: MobileView, token: UUID) -> some View {
        let item = Peek(row: row, boot: view.bootId)
        return agentRow(row, here: row.isFocused(in: view)) {
            jump(to: row, view: view, token: token, expectedAgent: row.pane.agentName)
        }
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
        let started = ContinuousClock.now
        guard let revision = connection.goTo(pane, boot: view.bootId, generation: token, expectedAgent: expectedAgent) else {
            issue = connection.message ?? "The selected terminal changed."
            connection.message = nil
            return
        }
        request = Jump(started: started, pane: pane, expectedAgent: expectedAgent, boot: view.bootId, generation: token, revision: revision)
        checkArrival()
    }

    private func checkArrival() {
        guard let request else { return }
        guard let view = connection.view, view.bootId == request.boot, connection.generation == request.generation else {
            self.request = nil
            dismiss()
            return
        }
        guard view.version >= request.revision, !view.goToPending else { return }
        self.request = nil
        let agentMatches = request.expectedAgent.map { expected in
            view.panes.contains { $0.id == request.pane.id && $0.agentName == expected }
        } ?? true
        if view.goToArrival == request.revision, agentMatches, request.pane.isFocused(in: view), view.inputTarget?.paneId == request.pane.id {
            #if DEBUG
            let elapsed = request.started.duration(to: .now)
            Logger(subsystem: "ai.2389.spaceport.mobile", category: "Navigation")
                .notice("Go To input ready after \(elapsed.formatted(.units(allowed: [.milliseconds])), privacy: .public)")
            #endif
            dismiss()
        } else {
            issue = view.error ?? "The selected pane changed before Herdr confirmed focus."
            connection.message = nil
        }
    }

    private func agentRow(_ row: GoToPane, here: Bool, action: @escaping () -> Void) -> some View {
        let activity = PaneActivity(row.pane.status)
        let role = row.pane.agentName == nil ? "result" : (activity == .blocked ? "needsYou" : "agent")
        let status = activity?.title
        let name = row.pane.agentName.flatMap { $0.isEmpty ? nil : $0 } ?? row.pane.label
        let path = row.location
        let label = AgentSelectorLabel(name: name, terminalTitle: row.pane.terminalTitle,
                                       location: path, font: .body.weight(.medium))
        return Button(action: action) {
            HStack(spacing: 10) {
                label
                Spacer(minLength: 4)
                VStack(alignment: .trailing, spacing: 3) {
                    HStack(spacing: 4) {
                        if let activity {
                            activity.dot
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

    private func workspaceSegments(_ view: MobileView, token: UUID) -> some View {
        ScrollView(.horizontal) {
            HStack(spacing: 2) {
                ForEach(view.workspaces, id: \.id) { item in
                    let selected = item.id == view.focusedWorkspace
                    Button {
                        guard connection.acceptsInput(from: presentation), token == connection.generation, !selected else { return }
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
