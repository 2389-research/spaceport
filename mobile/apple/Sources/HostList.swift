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
    private let store = ProfileStore()
    @StateObject private var sessions = SceneSessions()
    @Environment(\.scenePhase) private var scenePhase
    @AppStorage(ProfileStore.catalogKey) private var storedCatalog: Data?
    @State private var catalog = HostCatalog()
    @State private var sheet: CatalogSheet?
    @State private var sessionAfterSheet: HostProfile?
    @State private var path: [HomeRoute] = []
    @State private var markClock = Date()
    @State private var notice: String?
    @State private var removal: HostProfile?

    var body: some View {
        NavigationStack(path: $path) {
            List {
                if !catalog.hosts.isEmpty {
                    HomeAgentsSection(agents: catalog.observedAgents, now: markClock, mark: { row, action in
                        do { try store.mark(action, row: row); reload() }
                        catch { notice = error.localizedDescription }
                    }) { row in
                        open(row.profile, agent: row.agent)
                    }
                }
                if !catalog.sessions.isEmpty {
                    Section {
                        workspaceBrowser(catalog.profiles)
                    }
                }
                ForEach(catalog.hosts) { host in
                    Section {
                        ForEach(catalog.sessions.filter { $0.hostID == host.id }) { session in
                            let profile = host.profile(session: session)
                            Button { open(profile) } label: { HomeSessionRow(session: session) }
                                .buttonStyle(.plain)
                                .accessibilityLabel("\(profile.destination), \(session.name)")
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
                            Text("\(host.title) · \(host.address.destination)")
                            Spacer()
                            Button { path.append(.host(host.id)) } label: {
                                Image(systemName: "info.circle").imageScale(.large).frame(minWidth: 44, minHeight: 32)
                            }.accessibilityLabel("Host details for \(host.address.destination)")
                        }
                    }
                }
            }
            .listStyle(.insetGrouped)
            .textCase(nil)
            .overlay {
                if catalog.hosts.isEmpty {
                    ContentUnavailableView {
                        Label("Add your first host", systemImage: "server.rack")
                    } description: {
                        Text("Connect to a Herdr session over SSH to see its agents here.")
                    } actions: {
                        Button("Add Host", systemImage: "plus") { sheet = .addHost }
                            .buttonStyle(.borderedProminent).buttonBorderShape(.capsule)
                            .controlSize(.large).accessibilityIdentifier("home.empty.addHost")
                        Text("The host needs an SSH account with a POSIX shell and Herdr installed.")
                            .font(.footnote).foregroundStyle(.secondary)
                            .frame(maxWidth: 300).multilineTextAlignment(.center).padding(.top, 12)
                    }
                }
            }
            .navigationTitle("Agents")
            .toolbar {
                ToolbarItem(placement: .topBarLeading) {
                    Button("Settings", systemImage: "gearshape") { path.append(.settings) }.accessibilityIdentifier("Settings")
                }
                ToolbarItem(placement: .topBarTrailing) {
                    Button("Add Host", systemImage: "plus") { sheet = .addHost }.accessibilityIdentifier("Add Host")
                }
            }
            .navigationDestination(for: HomeRoute.self) { route in
                switch route {
                case .settings: MobileSettings(hosts: catalog.hosts, openHostDetails: { path = [.host($0)] })
                case .chooseWorkspaceSession:
                    List {
                        ForEach(catalog.hosts) { host in
                            let sessions = catalog.sessions.filter { $0.hostID == host.id }
                            if !sessions.isEmpty {
                                Section(host.address.destination) {
                                    ForEach(sessions) { session in
                                        Button(session.name) { open(host.profile(session: session), navigation: .workspaces) }
                                            .accessibilityLabel("\(host.address.destination), \(session.name)")
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
        .onAppear { reload() }
        .onChange(of: storedCatalog) { reload() }
        .onChange(of: scenePhase) { _, phase in
            if phase == .active { markClock = Date() }
            sessions.scenePhaseChanged(phase)
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

    private func closeInvalidPresentation() {
        if let presentation = path.last?.presentation, presentation != sessions.current { path = [] }
    }

    private func reload() {
        do {
            catalog = try store.catalog()
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
        return "\(status.blocked) need you, \(status.working) working, last seen \(status.seenAt.formatted(.relative(presentation: .named)))"
    }
}

private extension HomeRoute {
    var presentation: SessionPresentation? {
        if case let .session(presentation, _) = self { return presentation }
        return nil
    }
}
