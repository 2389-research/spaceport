// ABOUTME: Groups saved sessions by host and puts their observed agents first.
// ABOUTME: Opens each terminal from Home so its Home action returns to the session list.
import SwiftUI

private enum HomeRoute: Hashable { case settings, chooseWorkspaceSession, host(UUID), session(SessionPresentation, GoToSection?) }

private enum CatalogSheet: Identifiable {
    case addHost, editHost(SavedHost), session(SavedHost, HostProfile?)

    var id: String {
        switch self {
        case .addHost: "addHost"
        case let .editHost(host): "edit-\(host.id)"
        case let .session(host, profile): "session-\(host.id)-\(profile?.id.uuidString ?? "add")"
        }
    }
}

struct HostList: View {
    @AppStorage(TerminalAppearance.preference) private var terminalMode = TerminalAppearance.dark
    @StateObject private var appearance = WindowTerminalAppearance()
    private let store = ProfileStore()
    @StateObject private var sessions = SceneSessions()
    @Environment(\.scenePhase) private var scenePhase
    @AppStorage(ProfileStore.catalogKey) private var storedCatalog: Data?
    @State private var catalog = HostCatalog()
    @State private var loadedCatalog = false
    @State private var shortcutWindow = UUID()
    @State private var sheet: CatalogSheet?
    @State private var sessionAfterSheet: HostProfile?
    @State private var path: [HomeRoute] = []
    @AppStorage("home.hostTag") private var hostTag = ""
    private var visibleCatalog: HostCatalog { catalog.filtered(by: hostTag) }
    @State private var reviewOnly = false
    @State private var markClock = Date()
    @State private var notice: String?
    @State private var removal: HostProfile?
    @State private var refreshing: HostProfile?
    @State private var refreshPass = UUID()
    @State private var refreshFailures: Set<UUID> = []

    var body: some View {
        NavigationStack(path: $path) {
            List {
                if !catalog.hosts.isEmpty {
                    HomeAgentsSection(agents: visibleAgents, reviewOnly: reviewOnly, now: markClock, mark: { row, action in
                        do { try store.mark(action, row: row); reload() }
                        catch { notice = error.localizedDescription }
                    }) { row in
                        open(row.profile, agent: row.agent)
                    }
                }
                if !catalog.sessions.isEmpty {
                    Section {
                        workspaceBrowser(visibleCatalog.profiles)
                    }
                }
                ForEach(visibleCatalog.hosts) { host in
                    Section {
                        ForEach(catalog.sessions.filter { $0.hostID == host.id }) { session in
                            let profile = host.profile(session: session)
                            Button { open(profile) } label: {
                                HomeSessionRow(session: session, refreshFailed: refreshFailures.contains(profile.id))
                            }
                                .buttonStyle(.plain)
                                .accessibilityLabel("\(host.address.endpoint), \(session.name)")
                                .accessibilityValue(accessibilityStatus(session))
                                .swipeActions(allowsFullSwipe: false) {
                                    Button("Remove", role: .destructive) { removal = profile }
                                    Button("Edit") { sheet = .session(host, profile) }.tint(.blue)
                                }
                                .contextMenu {
                                    Button("Edit Session") { sheet = .session(host, profile) }
                                    if sessions.contains(profile) {
                                        Button("Disconnect", systemImage: "power", role: .destructive) {
                                            sessions.disconnect(profile)
                                        }.accessibilityIdentifier("session.disconnect")
                                    }
                                }
                        }
                        Button("New Session", systemImage: "plus") { sheet = .session(host, nil) }
                    } header: {
                        HStack(spacing: 8) {
                            Text(verbatim: host.listHeading(in: catalog.hosts))
                            Spacer()
                            Button { path.append(.host(host.id)) } label: {
                                Image(systemName: "info.circle").imageScale(.large).frame(minWidth: 44, minHeight: 32)
                            }.accessibilityLabel("Host details for \(host.address.endpoint)")
                        }
                    }
                }
            }
            .listStyle(.insetGrouped)
            .textCase(nil)
            .overlay {
                if loadedCatalog && catalog.hosts.isEmpty {
                    WelcomeCover { sheet = .addHost }
                }
            }
            .navigationTitle(catalog.hosts.isEmpty ? "" : "Agents")
            .toolbar {
                ToolbarItem(placement: .topBarLeading) {
                    Menu {
                        Toggle("Finished for review", isOn: $reviewOnly)
                        if !catalog.availableTags.isEmpty {
                            Picker("Host tag", selection: $hostTag) {
                                Text("All hosts").tag("")
                                ForEach(catalog.availableTags, id: \.self) { Text($0).tag($0.lowercased()) }
                            }
                        }
                    } label: { Image(systemName: reviewOnly || !hostTag.isEmpty ? "line.3.horizontal.decrease.circle.fill" : "line.3.horizontal.decrease.circle") }
                    .accessibilityLabel("Filter Home")
                    Button("Settings", systemImage: "gearshape") { path.append(.settings) }.accessibilityIdentifier("Settings")
                }
                ToolbarItem(placement: .topBarTrailing) {
                    Button("Add Host", systemImage: "plus") { sheet = .addHost }.accessibilityIdentifier("Add Host")
                }
                if let refreshing {
                    ToolbarItem(placement: .topBarTrailing) {
                        ProgressView().controlSize(.small)
                            .accessibilityLabel("Refreshing agents in \(refreshing.session)")
                            .accessibilityIdentifier("home.agents.refreshing")
                    }
                }
            }
            .navigationDestination(for: HomeRoute.self) { route in
                switch route {
                case .settings: MobileSettings(hosts: catalog.hosts, openHostDetails: { path = [.host($0)] })
                case .chooseWorkspaceSession:
                    List {
                        ForEach(visibleCatalog.hosts) { host in
                            let sessions = catalog.sessions.filter { $0.hostID == host.id }
                            if !sessions.isEmpty {
                                Section(host.listHeading(in: catalog.hosts)) {
                                    ForEach(sessions) { session in
                                        Button(session.name) { open(host.profile(session: session), navigation: .workspaces) }
                                            .accessibilityLabel("\(host.address.endpoint), \(session.name)")
                                            .accessibilityIdentifier("home.workspaces." + session.id.uuidString)
                                    }
                                }
                            }
                        }
                    }
                    .navigationTitle("Choose a session")
                case let .session(presentation, navigation):
                    SessionScreen(presentation: presentation, initialNavigation: navigation, hostDetails: {
                        guard let session = catalog.sessions.first(where: { $0.id == presentation.connection.profile.id }) else { return }
                        sessions.detach(presentation)
                        path = [.host(session.hostID)]
                    }, home: {
                        sessions.detach(presentation)
                        path = []
                    }) {
                        sessions.disconnect(presentation.connection.profile)
                        path = []
                    }
                    .id(presentation.id)
                case let .host(id):
                    if let host = catalog.hosts.first(where: { $0.id == id }) {
                        HostDetail(host: host, sessions: catalog.sessions.filter { $0.hostID == id },
                                   saveTags: { try store.tagHost(id, text: $0); reload() },
                                   open: { open($0) }, edit: { sheet = .editHost(host) },
                                   add: { sheet = .session(host, nil) }, remove: {
                            try store.removeHost(id)
                            path = []
                            reload()
                        })
                    }
                }
            }
        }
        .sheet(item: $sheet, onDismiss: {
            if let profile = sessionAfterSheet { sessionAfterSheet = nil; open(profile) }
        }) { request in
            switch request {
            case .addHost:
                HostEditor { address, login, name in
                    let target = try HostProfile.make(hostname: address.hostname, port: String(address.port),
                                                      username: address.username, session: name, login: login)
                    if let existing = try store.load().first(where: {
                        $0.address == address && $0.login == login && $0.session == target.session
                    }) {
                        sessionAfterSheet = existing
                    } else {
                        try store.save(target)
                        sessionAfterSheet = target
                    }
                    reload()
                }
            case let .editHost(host):
                HostEditor(host: host) { address, login, name in
                    var updated = host
                    updated.address = address
                    updated.login = login
                    let trimmed = name.trimmingCharacters(in: .whitespacesAndNewlines)
                    updated.name = trimmed.isEmpty ? nil : trimmed
                    try store.saveHost(updated)
                    reload()
                }
            case let .session(host, profile):
                SessionEditor(profile: profile) { name in
                    try store.saveSession(on: host.id, id: profile?.id, name: name)
                    reload()
                }
            }
        }
        .confirmationDialog("Remove \(removal?.session ?? "session") from this device?",
                            isPresented: Binding(get: { removal != nil }, set: { if !$0 { removal = nil } }),
                            titleVisibility: .visible, presenting: removal) { profile in
            Button("Remove Session", role: .destructive) {
                do { try store.remove(profile); reload() } catch { notice = error.localizedDescription }
            }
        } message: { _ in Text("Its Herdr session keeps running on the host.") }
        .onAppear {
            reload()
            if scenePhase == .active { activateShortcuts() }
        }
        .onOpenURL { url in
            guard let destination = AgentWidgetSnapshot.destinationID(from: url) else { return }
            do { try openShortcut(.widgetNextWaiting(destination: destination)) }
            catch { notice = error.localizedDescription }
        }
        .onDisappear { SpaceportShortcutRouter.shared.deactivate(shortcutWindow) }
        .onChange(of: storedCatalog) { reload() }
        .onChange(of: scenePhase) { _, phase in
            if phase == .active { markClock = Date() }
            sessions.scenePhaseChanged(phase)
            if phase == .active {
                activateShortcuts()
            } else {
                SpaceportShortcutRouter.shared.deactivate(shortcutWindow)
            }
        }
        .onChange(of: path) { _, path in
            if let current = sessions.current, !path.contains(where: { $0.presentation == current }) {
                sessions.detach(current)
            }
        }
        .onReceive(NotificationCenter.default.publisher(for: SceneSessions.forgotPin)) { notification in
            if let account = notification.object as? String {
                sessions.forget(pinAccount: account)
                closeInvalidPresentation()
            }
        }
        .environmentObject(appearance)
        .preferredColorScheme((appearance.reported ?? terminalMode).scheme)
        .task(id: homeRefreshTargets) { await refreshRosters(homeRefreshTargets) }
        .onChange(of: terminalMode) { _, mode in
            appearance.reported = nil
            GhosttyPaneSurface.controller.setTheme(mode.theme)
        }
        .task(id: snoozeDeadlines) {
            markClock = Date()
            for deadline in snoozeDeadlines {
                let delay = deadline.timeIntervalSinceNow
                if delay > 0 {
                    do { try await Task.sleep(for: .seconds(delay)) } catch { return }
                }
                markClock = Date()
            }
        }
        .alert("Sessions", isPresented: $notice.presented) { Button("OK") { notice = nil } } message: { Text(notice ?? "") }
    }

    private var visibleAgents: [HomeAgent] {
        let rows = visibleCatalog.observedAgents
        guard reviewOnly else { return rows }
        return rows.filter { $0.section == .finished && $0.agent.marks?.settled != true && $0.displaySection(at: markClock) != .snoozed }
    }

    private var homeRefreshTargets: [HostProfile] {
        guard loadedCatalog, scenePhase == .active, path.isEmpty, sheet == nil, sessions.current == nil else { return [] }
        return catalog.sessions.filter { $0.lastOpened != nil }.sorted {
            ($0.lastOpened ?? .distantPast) > ($1.lastOpened ?? .distantPast)
        }.compactMap { catalog.profile($0) }.filter {
            sessions.permitsHomeRefresh($0) && (sessions.hasParkedSession(for: $0) || $0.login == .phoneKey)
        }
    }

    @MainActor
    private func refreshRosters(_ targets: [HostProfile]) async {
        let pass = UUID()
        refreshPass = pass
        refreshing = nil
        refreshFailures = []
        defer { if refreshPass == pass { refreshing = nil } }
        for profile in targets {
            guard !Task.isCancelled, refreshPass == pass, homeRefreshTargets.contains(profile) else { return }
            refreshing = profile
            do {
                let previouslySeen = catalog.sessions.first { $0.id == profile.id }?.lastStatus?.seenAt
                let roster: MobileView?
                if let retained = try await sessions.observeRetainedRoster(for: profile) {
                    roster = retained
                } else if profile.login == .phoneKey {
                    try Task.checkCancellation()
                    roster = try await HomeRosterProbe.read(profile)
                } else {
                    roster = nil
                }
                try Task.checkCancellation()
                guard refreshPass == pass, homeRefreshTargets.contains(profile) else { return }
                guard let roster, let date = roster.rosterObservationDate else {
                    refreshFailures.insert(profile.id)
                    continue
                }
                try store.recordSeen(profile: profile, snapshot: SessionStatusSnapshot(view: roster, seenAt: date),
                                     source: .home(previouslySeen: previouslySeen))
            } catch {
                guard !Task.isCancelled, refreshPass == pass else { return }
                refreshFailures.insert(profile.id)
            }
        }
    }

    private var snoozeDeadlines: [Date] {
        catalog.sessions.flatMap { $0.lastStatus?.agents?.compactMap { $0.marks?.snoozeUntil } ?? [] }.sorted()
    }

    private func workspaceBrowser(_ profiles: [HostProfile]) -> some View {
        Button {
            if profiles.count == 1, let profile = profiles.first { open(profile, navigation: .workspaces) }
            else { path.append(.chooseWorkspaceSession) }
        } label: {
            Label("Workspaces", systemImage: "rectangle.stack")
                .frame(maxWidth: .infinity, minHeight: 44, alignment: .leading)
                .contentShape(Rectangle())
        }
        .accessibilityIdentifier("home.workspaces")
    }

    private func open(_ profile: HostProfile, agent: SeenAgent? = nil, navigation: GoToSection? = nil) {
        do {
            guard try store.load().contains(profile) else { reload(); return }
            let presentation = sessions.open(profile, agent: agent)
            path = [.session(presentation, navigation)]
        } catch { notice = error.localizedDescription }
    }

    private func activateShortcuts() {
        do {
            try SpaceportShortcutRouter.shared.activate(shortcutWindow, handler: openShortcut)
        } catch {
            notice = error.localizedDescription
        }
    }

    private func openShortcut(_ request: SpaceportShortcut) throws {
        guard sheet == nil, removal == nil else { throw ShortcutFailure.editing }
        let presentation = try sessions.openShortcut(request, in: store.catalog())
        if path.last?.presentation != presentation { path = [.session(presentation, nil)] }
    }

    private func closeInvalidPresentation() {
        if let presentation = path.last?.presentation, presentation != sessions.current { path = [] }
    }

    private func reload() {
        do {
            catalog = try store.catalog()
            if !catalog.availableTags.contains(where: { $0.lowercased() == hostTag }) { hostTag = "" }
            loadedCatalog = true
            sessions.reconcile(catalog.profiles)
            closeInvalidPresentation()
        } catch {
            sessions.reconcile([])
            closeInvalidPresentation()
            notice = error.localizedDescription
        }
    }

    private func accessibilityStatus(_ session: SavedSession) -> String {
        guard let status = session.lastStatus else { return "Not opened yet" }
        let failure = refreshFailures.contains(session.id) ? "Couldn’t refresh, " : ""
        return "\(failure)\(status.blocked) need you, \(status.working) working, last seen \(status.seenAt.formatted(.relative(presentation: .named)))"
    }
}

private extension HomeRoute {
    var presentation: SessionPresentation? {
        if case let .session(presentation, _) = self { return presentation }
        return nil
    }
}
