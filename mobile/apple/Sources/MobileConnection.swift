// ABOUTME: Owns a foreground SSH connection and its current Herdr projection.
// ABOUTME: Retires old connections before accepting their asynchronous results.
import Foundation
import SwiftUI

struct Viewport: Equatable {
    var columns: UInt16
    var rows: UInt16
    var cellWidth: UInt32
    var cellHeight: UInt32
    static let initial = Viewport(columns: 80, rows: 24, cellWidth: 8, cellHeight: 16)
}

@MainActor
final class MobileConnection: ObservableObject {
    #if DEBUG
    @Published private(set) var previewPort: UInt16?

    func startServicePreview(_ port: UInt16) async throws -> UInt16 {
        guard phase == .ready, let session else { throw MobileError.Disconnected }
        let token = generation
        let local = try await Task.detached { try session.startServicePreview(remotePort: port) }.value
        guard generation == token else {
            try? await Task.detached { try session.stopServicePreview() }.value
            throw MobileError.Disconnected
        }
        previewPort = local
        return local
    }

    func stopServicePreview() async throws {
        guard let session else { return }
        let token = generation
        try await Task.detached { try session.stopServicePreview() }.value
        if generation == token { previewPort = nil }
    }
    #endif

    private enum DiagnosticStage: String {
        case verifyHost = "SSH host verification", open = "SSH login / Herdr attach"
        case roster = "Herdr handshake / roster", connected = "Connected", resume = "Verify retained session"
    }
    private var stage: DiagnosticStage = .verifyHost
    private var lastFailure: DiagnosticStage?
    private var lastCapabilities: String?
    private var observedRoster: Date?

    private var connectionStage: DiagnosticStage {
        if stage == .open, session?.diagnosticCapabilities() != nil { return .roster }
        return stage
    }

    func diagnosticReport() -> String {
        let build = Bundle.main.object(forInfoDictionaryKey: "CFBundleVersion") as? String ?? "unknown"
        let caps = session?.diagnosticCapabilities() ?? lastCapabilities ?? "unavailable"
        let catalog = try? ProfileStore().catalog()
        let cached = catalog?.sessions.first { catalog?.profile($0) == profile }?.lastStatus?.seenAt
        let observed = (observedRoster ?? cached)?.ISO8601Format() ?? "unknown"
        return ["Spaceport iOS build " + build, "Stage: " + connectionStage.rawValue,
                "Last failure: " + (lastFailure?.rawValue ?? "none"),
                "Last negotiated capabilities: " + caps, "Last roster observation: " + observed].joined(separator: "\n")
    }

    enum Phase { case credentials, connecting, trust, locatingAgent, ready, closed }
    private(set) var profile: HostProfile
    @Published var authorizeThisPhone = SecurePhoneKey.isAvailable
    @Published var enrollmentNotice: String?
    @Published var method: LoginMethod
    @Published var draft = TerminalDraft()
    @Published var password = ""
    @Published private(set) var phase: Phase = .credentials
    @Published private(set) var view: MobileView?
    @Published private(set) var challenge: HostChallenge?
    @Published var message: String?
    @Published private(set) var inputStatus: String?
    private(set) var reconnectFrame: MobileTerminal?
    private(set) var editorTarget: InputTarget?
    private(set) var panePreviews = PanePreviewCache()
    private var lastRosterSave: RosterSaveAttempt?
    @Published private(set) var directPane: DirectPaneAttachment?
    private var session: MobileSession?
    private var paneTrail: MobilePaneTrail?
    private var opening: Task<Void, Never>?
    private var polling: Task<Void, Never>?
    private(set) var generation = UUID()
    private var viewport = Viewport.initial
    private var appliedViewport: Viewport?
    private var resumeRequested = false
    private var recovery = TransportRecovery()
    private var parkedRoster: Task<Void, Never>?
    private(set) var isObservingParkedRoster = false
    var parkedRosterChanged: (() -> Void)?
    private var parkedBoot: String?
    private var verification: Task<Void, Never>?

    var hasParkedSession: Bool { parkedBoot != nil }

    func observeParkedRoster() async throws -> MobileView? {
        try Task.checkCancellation()
        guard presentationID == nil, let boot = parkedBoot, let session else { return nil }
        let token = generation
        let request = try session.beginParkedRoster(bootId: boot)
        let roster = try await withTaskCancellationHandler {
            try await Task.detached(priority: .utility) {
                try session.waitParkedRoster(bootId: boot, request: request)
            }.value
        } onCancel: {
            session.cancelParkedRoster(request: request)
        }
        try Task.checkCancellation()
        guard generation == token, presentationID == nil, parkedBoot == boot else { throw CancellationError() }
        return roster
    }

    private enum AgentArrival {
        case requested(SeenAgent)
        case moving(SeenAgent, revision: UInt64)
    }
    private var arrival: AgentArrival?
    @Published private(set) var pendingShortcut: SessionShortcutRequest?
    private(set) var presentationID: UUID? {
        didSet {
            if oldValue != presentationID { pendingShortcut = nil }
        }
    }

    func requestShortcut(_ action: SessionShortcutAction, presentation: UUID) {
        guard presentationID == presentation, pendingShortcut == nil else { return }
        // A warm agent jump can already have failed its identity check in present().
        if case .revealTerminal = action, phase == .closed { return }
        // A connected screen may already hold a roster from before the shortcut.
        // A cold open or parked session will receive its own SSH proof on entry.
        let needsRefresh: Bool
        if case .nextWaiting = action { needsRefresh = view?.bootId.isEmpty == false }
        else { needsRefresh = false }
        pendingShortcut = SessionShortcutRequest(action: action, destination: phase == .ready ? draftDestination : nil,
                                                needsRosterRefresh: needsRefresh)
    }

    var readyShortcutID: UUID? {
        guard phase == .ready, let request = pendingShortcut, let view, !view.goToPending else { return nil }
        // The first roster can precede the focused pane's input target.
        if case .compose = request.action, request.destination == nil, view.focusedPane != nil, view.inputTarget == nil {
            return nil
        }
        return request.id
    }

    func takeShortcut(from presentation: UUID) -> SessionShortcutAction? {
        guard presentationID == presentation, readyShortcutID != nil, let request = pendingShortcut else { return nil }
        if request.needsRosterRefresh {
            guard let session else { return nil }
            pendingShortcut?.needsRosterRefresh = false
            endDirectPane()
            park(session)
            foreground()
            return nil
        }
        pendingShortcut = nil
        if case let .compose(text) = request.action {
            draft.compose(text, at: request.destination ?? draftDestination)
        }
        return request.action
    }

    var openingAgent: SeenAgent? {
        switch arrival {
        case let .requested(agent), let .moving(agent, _): agent
        case nil: nil
        }
    }

    func present(_ id: UUID, agent: SeenAgent?, alreadyOpen: Bool) {
        AgentLiveActivity.shared.select(id, view: view, observedAt: view?.rosterObservationDate)
        if alreadyOpen, presentationID == id, agent == nil || agent == openingAgent { return }
        if presentationID != id || agent != nil { reconnectFrame = nil }
        recovery = TransportRecovery()
        presentationID = id
        observeParkedRoster(false)
        arrival = agent.map(AgentArrival.requested)
        if hasParkedSession {
            foreground()
        } else if phase == .ready {
            if let arrival, let view { _ = locate(arrival, in: view) }
        } else if !alreadyOpen {
            firstOpen()
        }
    }

    func keepOnHome() -> Bool {
        AgentLiveActivity.shared.end(presentationID)
        if method == profile.login, phase == .ready || hasParkedSession {
            presentationID = nil
            arrival = nil
            session?.cancelGoTo()
            background()
            password = ""
            return true
        }
        guard !draft.isEmpty else { return false }
        closePresentation()
        method = profile.login
        return true
    }

    func acceptsInput(from id: UUID) -> Bool {
        presentationID == id && phase == .ready
    }

    init(profile: HostProfile) {
        self.profile = profile
        method = profile.login
    }

    var draftDestination: DraftDestination? {
        guard let view, let target = view.inputTarget else { return nil }
        return DraftDestination(target: target, workspaces: view.workspaces, tabs: view.tabs, panes: view.panes)
    }

    func chooseMethod(_ next: LoginMethod, from id: UUID) {
        guard presentationID == id, method != next, phase == .credentials || phase == .closed else { return }
        retire()
        password = ""
        method = next
        phase = .credentials
        if next == .password { firstOpen() }
    }

    func firstOpen(checkHost: @escaping @Sendable (SSHAddress) throws -> Void = { address in
        try checkSshHost(host: DeviceVault().host(for: address))
    }) {
        guard method == .password else { startConnection(); return }
        stage = .verifyHost
        phase = .connecting
        let token = generation
        let profile = profile
        opening = Task {
            guard !Task.isCancelled else { return }
            do {
                try await Task.detached {
                    try checkHost(profile.address)
                }.value
                guard token == generation else { return }
                phase = .credentials
            } catch {
                guard token == generation else { return }
                lastFailure = connectionStage
                if let issue = HostChallenge(sshError: error) {
                    challenge = issue
                    phase = .trust
                } else {
                    phase = .closed
                    message = error.localizedDescription
                }
            }
        }
    }

    func connect() {
        recovery.allowRetry()
        startConnection(authorizePhone: method == .password && authorizeThisPhone)
    }

    private func startConnection(authorizePhone: Bool = false) {
        retire()
        guard method == .phoneKey || !password.isEmpty else { phase = .credentials; return }
        stage = .open
        phase = .connecting
        message = nil
        let token = generation
        let profile = profile
        let size = viewport
        let method = method
        let password = password
        opening = Task {
            guard !Task.isCancelled else { return }
            do {
                let handle = try await Task.detached(priority: .userInitiated) {
                    let vault = DeviceVault()
                    let host = try vault.host(for: profile.address)
                    let auth: SshAuth = try method == .phoneKey ? vault.securePhoneKey().auth : .password(password: password)
                    return try MobileSession.open(host: host, auth: auth, session: profile.session, cols: size.columns,
                                                  rows: size.rows, cellWidth: size.cellWidth, cellHeight: size.cellHeight)
                }.value
                guard token == generation else { handle.disconnect(); return }
                if let paneTrail { handle.restorePaneTrail(saved: paneTrail) }
                session = handle
                appliedViewport = size
                pollUpdates(handle, token: token)
                if authorizePhone { enroll(on: handle) }
            } catch {
                guard token == generation else { return }
                lastFailure = connectionStage
                phase = .closed
                message = error.localizedDescription
            }
        }
    }

    private func enroll(on handle: MobileSession) {
        Task {
            do {
                try await Task.detached {
                    let key = try DeviceVault().securePhoneKey()
                    try handle.authorizePhone(publicKey: key.publicPoint, signer: key)
                }.value
                guard session === handle else { return }
                do { try saveVerifiedPhoneKey(using: ProfileStore()) }
                catch { enrollmentNotice = "Phone Key verified. Spaceport could not save this host’s sign-in preference." }
            } catch {
                guard session === handle else { return }
                switch error {
                case SshError.Cancelled:
                    return
                case SshError.ChangedHost, SshError.UnknownHost:
                    enrollmentNotice = "The verification connection presented an unexpected host key. Check the host fingerprint before using Phone Key."
                case SshError.Channel, SshError.Enrollment:
                    enrollmentNotice = "The host refused to install Phone Key. Your password session is still open."
                default:
                    enrollmentNotice = "Spaceport couldn’t verify Phone Key for this account. You can keep using this session and retry from Settings → Phone Key."
                }
            }
        }
    }

    func saveVerifiedPhoneKey(using store: ProfileStore) throws {
        let catalog = try store.catalog()
        guard let entry = catalog.sessions.first(where: { $0.id == profile.id }),
              let host = catalog.hosts.first(where: { $0.id == entry.hostID }),
              host.address == profile.address, entry.name == profile.session else { throw ProfileError.missingHost }
        guard try store.usePhoneKey(for: host) else {
            enrollmentNotice = "Phone Key verified. Another saved host already uses this account with Phone Key. Both sign-in settings were kept."
            return
        }
        adoptVerifiedPhoneKey()
    }

    func adoptVerifiedPhoneKey() {
        profile.login = .phoneKey
        method = .phoneKey
        password = ""
    }

    private func pollUpdates(_ handle: MobileSession, token: UUID, after initialVersion: UInt64 = 0) {
        polling = Task.detached(priority: .userInitiated) { [weak self] in
            var version = initialVersion
            while !Task.isCancelled {
                if let update = handle.pollForAttach(after: version) {
                    version = update.version
                    await self?.receive(update, token: token)
                    if update.disconnected { break }
                }
                do { try await Task.sleep(for: .milliseconds(35)) } catch { break }
            }
        }
    }

    private func receive(_ update: MobileView, token: UUID) {
        guard generation == token else { return }
        if update.disconnected {
            let established = phase == .ready || phase == .locatingAgent
            recordLastSeen()
            if let issue = update.hostChallenge {
                stage = .verifyHost
                lastFailure = connectionStage
                retire()
                challenge = issue
                phase = .trust
            } else {
                finishLoss(eligible: established && session?.transportLost() == true,
                           message: update.error ?? "Connection closed.")
            }
            return
        }
        let previousError = view?.error
        if let target = update.inputTarget { editorTarget = target }
        let changedPane = view?.focusedPaneTarget?.paneId != update.focusedPaneTarget?.paneId
        if changedPane, let outgoing = view, let target = outgoing.focusedPaneTarget,
           let terminal = outgoing.terminal, outgoing.bootId == update.bootId {
            let agent = outgoing.panes.first { $0.id == target.paneId }?.agentName
            panePreviews.record(terminal.visibleText, pane: target.paneId, boot: target.bootId, agent: agent)
        }
        if view?.bootId != update.bootId { panePreviews = PanePreviewCache() }
        let previous = view
        view = update
        if !update.bootId.isEmpty {
            stage = .connected
            if let date = update.rosterObservationDate {
                observedRoster = date
                AgentLiveActivity.shared.observe(update, previous: previous, at: date, from: presentationID)
            }
            reconnectFrame = nil
            if let arrival, !locate(arrival, in: update) { return }
            if phase != .ready { lastRosterSave = nil }
            phase = .ready
            if let status = update.changedAgentStatus(since: lastRosterSave) {
                saveObservation(status, view: update)
            }
            syncDirectPane()
            if changedPane, let target = update.focusedPaneTarget { panePreviews.visit(target.paneId, boot: target.bootId) }
            if appliedViewport != viewport { resize(viewport) }
        }
        if update.error != previousError, let failure = update.error { message = failure }
    }

    // Keep the keyboard and terminal hidden until Herdr confirms the selected agent's pane.
    private func locate(_ request: AgentArrival, in update: MobileView) -> Bool {
        phase = .locatingAgent
        switch request {
        case let .requested(agent):
            if update.bootId == agent.bootId, let session,
               let revision = try? session.goToPane(bootId: agent.bootId, workspaceId: agent.workspaceId,
                                                    tabId: agent.tabId, paneId: agent.paneId, expectedAgent: agent.name) {
                arrival = .moving(agent, revision: revision)
                return false
            }
        case let .moving(agent, revision):
            guard update.version >= revision, !update.goToPending else { return false }
            if update.confirmsInput(to: agent.paneId, revision: revision), update.bootId == agent.bootId, update.focusedWorkspace == agent.workspaceId,
               update.focusedTab == agent.tabId,
               update.panes.contains(where: { $0.id == agent.paneId && $0.tabId == agent.tabId && $0.agentName == agent.name }) {
                arrival = nil
                return true
            }
        }
        arrival = nil
        pendingShortcut = nil
        retire()
        phase = .closed
        message = "Spaceport couldn’t open the last-seen agent. Connect to the session to choose a live agent."
        return false
    }

    func trust() { _ = trust(using: DeviceVault()) }

    func trust(using vault: DeviceVault) -> Task<Void, Never>? {
        guard phase == .trust, let challenge, !challenge.changed else { return nil }
        phase = .connecting
        let token = generation
        let account = profile.pinAccount
        return Task {
            do {
                try await Task.detached { try vault.trust(challenge.fingerprint, account: account) }.value
                if generation == token { connect() }
            } catch {
                guard generation == token else { return }
                phase = .trust
                message = error.localizedDescription
            }
        }
    }

    func resize(_ size: Viewport) {
        viewport = size
        guard let session, let view, !view.bootId.isEmpty, appliedViewport != size else { return }
        perform {
            try session.resize(bootId: view.bootId, cols: size.columns, rows: size.rows,
                               cellWidth: size.cellWidth, cellHeight: size.cellHeight)
            appliedViewport = size
        }
    }

    func syncDirectPane() {
        guard phase == .ready, presentationID != nil, let view, let session,
              !view.goToPending, view.terminal == nil else { endDirectPane(); return }
        if let attached = directPane,
           attached.target.bootId != view.bootId || attached.target.paneId != view.focusedPane {
            endDirectPane()
        }
        if let attached = directPane, attached.cachedViewport != nil,
           !view.panes.contains(where: { $0.id == attached.target.paneId && $0.agentName == attached.agent }) {
            attached.dismissCachedViewport()
        }
        // A partial projection can briefly lack a target. Rust checks the latest
        // focus before every write, so a short gap needn't tear down the renderer.
        guard let target = view.inputTarget else { return }
        guard target.popupId == nil else { endDirectPane(); return }
        guard directPane?.target != target else { return }
        endDirectPane()
        let agent = view.panes.first { $0.id == target.paneId }?.agentName
        let cached = panePreviews.viewport(pane: target.paneId, boot: target.bootId, agent: agent)
        directPane = DirectPaneAttachment(session: session, target: target, size: viewport, agent: agent, cachedViewport: cached)
    }

    func retryDirectPane() {
        endDirectPane()
        syncDirectPane()
    }

    private func endDirectPane() {
        guard let directPane else { return }
        if directPane.hasPaneOutput, let view, view.bootId == directPane.target.bootId,
           let pane = view.panes.first(where: { $0.id == directPane.target.paneId }), pane.agentName == directPane.agent,
           let text = directPane.surface?.visibleText {
            panePreviews.record(text, pane: pane.id, boot: view.bootId, agent: pane.agentName)
        }
        directPane.close()
        self.directPane = nil
    }

    func claimSize() {
        guard let session, let view, !view.bootId.isEmpty else { return }
        perform { try session.claimSize(bootId: view.bootId) }
    }

    func createTab(workspace: String, sourceTab: String, boot: String, from presentation: UUID) {
        guard acceptsInput(from: presentation), let session else { return }
        if perform({ try session.createTab(bootId: boot, workspace: workspace, sourceTab: sourceTab) }) {
            endDirectPane()
        }
    }

    func editPane(_ pane: GoToPane, boot: String, generation token: UUID, action: MobilePaneAction) {
        guard token == generation, phase == .ready, let session else { return }
        if perform({ try session.applyPaneAction(bootId: boot, workspaceId: pane.workspace.id, tabId: pane.tab.id, paneId: pane.id, action: action) }) {
            endDirectPane()
        }
    }

    func goTo(_ pane: GoToPane, boot: String, generation token: UUID, expectedAgent: String? = nil, historyForward: Bool? = nil) -> UInt64? {
        guard token == generation, phase == .ready, let session else { return nil }
        do {
            let revision: UInt64
            if let historyForward {
                revision = try session.travelToPane(bootId: boot, paneId: pane.id, forward: historyForward)
            } else {
                revision = try session.goToPane(bootId: boot, workspaceId: pane.workspace.id, tabId: pane.tab.id, paneId: pane.id, expectedAgent: expectedAgent)
            }
            endDirectPane()
            return revision
        } catch {
            message = error.localizedDescription
            return nil
        }
    }

    func cancelGoTo(generation token: UUID) {
        if token == generation { session?.cancelGoTo() }
    }

    func inputUnavailable() {
        claimSize()
        inputStatus = "Input paused — the pane changed. Check it and type again."
    }

    func findHistory(_ query: String, newer: Bool, previous: HistoryCursor?, target: InputTarget,
                     from presentation: UUID) async throws -> HistoryMatch? {
        guard acceptsInput(from: presentation), let session, view?.inputTarget == target else { throw MobileError.StaleView }
        let search = try session.historySearch(target: target)
        let result = try await withTaskCancellationHandler {
            try await Task.detached(priority: .userInitiated) {
                try search.find(query: query, newer: newer, previous: previous)
            }.value
        } onCancel: {
            search.cancel()
        }
        try Task.checkCancellation()
        guard acceptsInput(from: presentation), self.session === session, view?.inputTarget == target else { throw MobileError.StaleView }
        return result
    }

    func scroll(target: InputTarget?, lines: Int16, column: UInt16 = 0, row: UInt16 = 0) {
        guard let session, let target, view?.inputTarget == target else { inputUnavailable(); return }
        perform { try session.scroll(target: target, lines: lines, column: column, row: row) }
    }

    func sendDraftAttachment(from presentation: UUID, pressReturn: Bool, transfer: MobileFileTransfer) async throws {
        let image = draft.image
        let file = draft.file
        guard image != nil || file != nil, let target = draft.destination?.target,
              let session, acceptsInput(from: presentation), view?.inputTarget == target else { throw MobileError.StaleView }
        let direct = directPane
        if target.popupId == nil && (direct?.target != target || direct?.failure != nil) { throw MobileError.TerminalUnavailable }
        let note = draft.text
        let staged = try await withTaskCancellationHandler {
            try await Task.detached(priority: .userInitiated) {
                if let file { return try session.stageDraftFile(target: target, name: file.name, bytes: file.data, transfer: transfer) }
                guard let image else { throw MobileError.StaleView }
                return try session.stageDraftImage(target: target, png: image.png)
            }.value
        } onCancel: { transfer.cancel() }
        try Task.checkCancellation()
        guard self.session === session, directPane === direct,
              draft.file?.id == file?.id, draft.image?.id == image?.id, draft.text == note, draft.destination?.target == target,
              UIApplication.shared.applicationState == .active else { throw MobileError.StaleView }
        let path = staged.pastePath()
        let paste = note.isEmpty ? path : "\(path) \(note)"
        guard send(.paste(paste, pressReturn: pressReturn), to: target, from: presentation) else {
            throw MobileError.TerminalUnavailable
        }
        staged.keep()
        draft = TerminalDraft()
    }

    enum Input { case text(String), key(String, UInt8), paste(String, pressReturn: Bool = false) }
    @discardableResult
    func send(_ input: Input, to target: InputTarget, from presentation: UUID) -> Bool {
        guard acceptsInput(from: presentation) else { return false }
        guard let session, target == view?.inputTarget else { inputUnavailable(); return false }
        inputStatus = nil
        if target.popupId == nil {
            guard let directPane, directPane.target == target,
                  directPane.send(input) else { return false }
            recovery.allowRetry()
            return true
        }
        let sent = perform {
            switch input {
            case let .text(text): try session.sendText(target: target, text: text)
            case let .key(key, modifiers): try session.sendKey(target: target, key: key, modifiers: modifiers)
            case let .paste(text, pressReturn):
                try session.sendPaste(target: target, text: text, pressReturn: pressReturn)
            }
        }
        if sent { recovery.allowRetry() }
        return sent
    }

    @discardableResult
    private func perform(_ body: () throws -> Void) -> Bool {
        do {
            try body()
            return true
        } catch {
            message = error.localizedDescription
            return false
        }
    }

    private func recordLastSeen() {
        guard phase == .ready, let view, !view.bootId.isEmpty, let date = view.rosterObservationDate else { return }
        saveObservation(SessionStatusSnapshot(view: view, seenAt: date), view: view)
    }

    private func saveObservation(_ snapshot: SessionStatusSnapshot, view: MobileView) {
        do {
            let saved = try ProfileStore().recordSeen(profile: profile, snapshot: snapshot)
            lastRosterSave = RosterSaveAttempt(view, accepted: saved)
        } catch { message = error.localizedDescription }
    }

    func sceneInactive() {
        recovery.sceneInactive(established: phase == .ready || phase == .locatingAgent || hasParkedSession)
    }

    func background() {
        AgentLiveActivity.shared.pause(presentationID)
        endDirectPane()
        sceneInactive()
        // An unanswered fingerprint remains available for comparison in another app.
        if phase == .trust || phase == .credentials || phase == .closed { return }
        if verification == nil && hasParkedSession { return }
        recordLastSeen()
        if let session, phase == .ready || phase == .locatingAgent || hasParkedSession {
            park(session)
            return
        }
        resumeRequested = phase == .connecting
        retire()
        phase = .closed
    }

    private func park(_ session: MobileSession) {
        if case let .moving(agent, _) = arrival { arrival = .requested(agent) }
        resumeRequested = false
        if let view { parkedBoot = view.bootId }
        generation = UUID()
        polling?.cancel()
        polling = nil
        verification?.cancel()
        verification = nil
        session.park()
        let lastFrame = presentationID != nil && arrival == nil ? view?.terminal : nil
        clearView()
        reconnectFrame = lastFrame
        phase = .connecting
    }

    /// A parked terminal still receives snapshots without holding focus or a surface.
    func observeParkedRoster(_ enabled: Bool) {
        guard enabled, hasParkedSession, presentationID == nil, let session else {
            parkedRoster?.cancel()
            parkedRoster = nil
            if isObservingParkedRoster {
                isObservingParkedRoster = false
                parkedRosterChanged?()
            }
            return
        }
        guard parkedRoster == nil else { return }
        let token = generation
        parkedRoster = Task.detached(priority: .utility) { [weak self] in
            var version: UInt64 = 0
            var saved: MobileView?
            while !Task.isCancelled {
                if let update = session.pollForAttach(after: version) {
                    version = update.version
                    guard !Task.isCancelled else { return }
                    let snapshot = update.observedRoster(after: saved)
                    await self?.receiveParkedRoster(update, snapshot: snapshot, token: token)
                    if update.disconnected { return }
                    if snapshot != nil { saved = update }
                }
                do { try await Task.sleep(for: .milliseconds(500)) } catch { return }
            }
        }
    }

    private func receiveParkedRoster(_ update: MobileView, snapshot: SessionStatusSnapshot?, token: UUID) {
        guard !Task.isCancelled, generation == token, presentationID == nil, hasParkedSession else { return }
        if update.disconnected {
            let wasLive = isObservingParkedRoster
            finishLoss(eligible: false, message: update.error ?? "Connection closed.")
            if !wasLive { parkedRosterChanged?() }
            return
        }
        guard let snapshot else { return }
        do {
            guard try ProfileStore().recordObserved(profile: profile, snapshot: snapshot) else { return }
        } catch { message = error.localizedDescription; return }
        if !isObservingParkedRoster {
            isObservingParkedRoster = true
            parkedRosterChanged?()
        }
    }

    func foreground() {
        if let presentationID {
            AgentLiveActivity.shared.select(presentationID, view: view, observedAt: view?.rosterObservationDate)
        }
        if recovery.sceneActive() {
            resumeRequested = false
            if presentationID != nil { reopen() }
            return
        }
        if let session, let parkedBoot {
            guard verification == nil else { return }
            let token = generation
            do {
                stage = .resume
                let request = try session.beginResume(bootId: parkedBoot)
                verification = Task.detached(priority: .userInitiated) { [weak self] in
                    do {
                        while !Task.isCancelled {
                            if try session.checkResume(bootId: parkedBoot, request: request) {
                                let update = session.pollForAttach(after: 0)
                                if let latest = update, !latest.disconnected, latest.bootId == parkedBoot {
                                    await self?.resumeVerified(session, latest: latest, token: token)
                                } else {
                                    await self?.failedResume(session, error: MobileError.Disconnected, token: token)
                                }
                                return
                            }
                            try await Task.sleep(for: .milliseconds(35))
                        }
                    } catch {
                        if !Task.isCancelled { await self?.failedResume(session, error: error, token: token) }
                    }
                }
            } catch { failedResume(session, error: error, token: token) }
            return
        }
        guard resumeRequested else { return }
        resumeRequested = false
        reopen()
    }

    private func resumeVerified(_ session: MobileSession, latest: MobileView, token: UUID) {
        guard generation == token, hasParkedSession else { return }
        verification = nil
        receive(latest, token: token)
        parkedBoot = nil
        if generation == token { pollUpdates(session, token: token, after: latest.version) }
    }

    private func failedResume(_ session: MobileSession, error: Error, token: UUID) {
        guard generation == token, hasParkedSession else { return }
        let eligible = session.transportLost() || (error as? MobileError) == .ResumeTimeout
        finishLoss(eligible: eligible, message: error.localizedDescription)
    }

    private func finishLoss(eligible: Bool, message diagnostic: String) {
        lastFailure = connectionStage
        let established = phase == .ready || phase == .locatingAgent || hasParkedSession
        let interrupted = (established && arrival != nil) || session?.focusInterrupted() == true
        let action = recovery.loss(eligible: eligible, presented: presentationID != nil, focusPending: interrupted)
        let lastFrame = view?.terminal ?? reconnectFrame
        if established { arrival = nil }
        retire()
        switch action {
        case .reconnect:
            reopen()
            if phase == .connecting { reconnectFrame = lastFrame }
        case .deferUntilActive:
            phase = .closed
            message = nil
        case .stop:
            phase = .closed
            message = interrupted ? "The connection closed while switching panes. Select the agent or pane again." : diagnostic
        }
    }

    private func reopen() {
        if method == .password && password.isEmpty {
            retire()
            firstOpen()
        } else { startConnection() }
    }

    func leave() {
        closePresentation()
        draft = TerminalDraft()
    }

    private func closePresentation() {
        AgentLiveActivity.shared.end(presentationID)
        recordLastSeen()
        resumeRequested = false
        presentationID = nil
        arrival = nil
        password = ""
        message = nil
        retire()
        phase = .closed
    }

    private func retire() {
        AgentLiveActivity.shared.pause(presentationID)
        closeResources()
        clearView()
    }

    private func closeResources() {
        #if DEBUG
        previewPort = nil
        #endif
        endDirectPane()
        enrollmentNotice = nil
        generation = UUID()
        verification?.cancel()
        verification = nil
        parkedBoot = nil
        observeParkedRoster(false)
        opening?.cancel()
        opening = nil
        polling?.cancel()
        polling = nil
        if let capabilities = session?.diagnosticCapabilities() { lastCapabilities = capabilities }
        if let saved = session?.savedPaneTrail() { paneTrail = saved }
        session?.disconnect()
        session = nil
        appliedViewport = nil
    }

    private func clearView() {
        panePreviews = PanePreviewCache()
        view = nil
        lastRosterSave = nil
        reconnectFrame = nil
        challenge = nil
        inputStatus = nil
        editorTarget = nil
    }
}
