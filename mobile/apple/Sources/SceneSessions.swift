// ABOUTME: Retains the two most recently opened SSH sessions for one window scene.
// ABOUTME: Separates presentation lifetime from transport lifetime and revokes obsolete targets.
import Combine
import SwiftUI

struct SessionPresentation: Identifiable, Hashable {
    let id = UUID()
    let connection: MobileConnection

    static func == (lhs: Self, rhs: Self) -> Bool { lhs.id == rhs.id }
    func hash(into hasher: inout Hasher) { hasher.combine(id) }
}

@MainActor
final class SceneSessions: ObservableObject {
    static let forgotPin = Notification.Name("SpaceportForgotHostPin")
    private let sceneID = UUID()
    private var profiles: [HostProfile] = []
    private var active = false
    private var readiness: AnyCancellable?
    private var currentReady = false
    private var enrollment: AnyCancellable?
    private var recent: [MobileConnection] = []
    @Published private(set) var current: SessionPresentation? {
        didSet {
            readiness = nil
            currentReady = false
            readiness = current?.connection.$phase.map { $0 == .ready }.removeDuplicates().sink { [weak self] ready in
                self?.currentReady = ready
                self?.publishClaims()
            }
        }
    }

    init() {
        enrollment = ProfileStore.verifiedPhoneKeyProfiles.sink { [weak self] previous in
            guard let self else { return }
            for connection in recent where previous.contains(connection.profile) {
                connection.adoptVerifiedPhoneKey()
            }
        }
    }

    deinit {
        let owned = recent
        let sceneID = sceneID
        Task { @MainActor in
            RosterObservers.shared.remove(scene: sceneID)
            owned.forEach { $0.leave() }
        }
    }

    func contains(_ profile: HostProfile) -> Bool { recent.contains { $0.profile == profile } }

    func connection(for profile: HostProfile) -> MobileConnection {
        discard { $0.profile.id == profile.id && $0.profile != profile }
        if let index = recent.firstIndex(where: { $0.profile == profile }) {
            let connection = recent.remove(at: index)
            recent.append(connection)
            return connection
        }
        if recent.count == 2 { discard(recent[0]) }
        let connection = MobileConnection(profile: profile)
        connection.parkedRosterChanged = { [weak self] in self?.publishClaims() }
        recent.append(connection)
        return connection
    }

    func open(_ profile: HostProfile, agent: SeenAgent?) -> SessionPresentation {
        if let current {
            if current.connection.profile == profile {
                current.connection.present(current.id, agent: agent, alreadyOpen: true)
                return current
            }
            detach(current)
        }
        let connection = connection(for: profile)
        let presentation = SessionPresentation(connection: connection)
        current = presentation
        updateObservers()
        RosterObservers.shared.resume(profile)
        connection.present(presentation.id, agent: agent, alreadyOpen: false)
        return presentation
    }

    func detach(_ presentation: SessionPresentation) {
        guard presentation == current else { return }
        current = nil
        let connection = presentation.connection
        if !connection.keepOnHome() { discard(connection) }
        updateObservers()
    }

    func disconnect(_ profile: HostProfile) {
        RosterObservers.shared.pause(profile)
        discard { $0.profile == profile }
        updateObservers()
    }

    func reconcile(_ profiles: [HostProfile]) {
        self.profiles = profiles
        RosterObservers.shared.replaceCatalog(profiles)
        let saved = Set(profiles)
        discard { !saved.contains($0.profile) }
        updateObservers()
    }

    func forget(pinAccount: String) {
        for profile in profiles where profile.pinAccount == pinAccount {
            RosterObservers.shared.pause(profile)
        }
        discard { $0.profile.pinAccount == pinAccount }
        updateObservers()
    }

    func scenePhaseChanged(_ phase: ScenePhase) {
        active = phase == .active
        if !active { updateObservers() }
        if phase == .background {
            for connection in recent { connection.background() }
        } else if phase == .inactive {
            current?.connection.sceneInactive()
        } else if phase == .active {
            current?.connection.foreground()
            updateObservers()
        }
    }

    private func updateObservers() {
        for connection in recent {
            connection.observeParkedRoster(active && current?.connection !== connection)
        }
        publishClaims()
    }

    private func publishClaims() {
        let observers = RosterObservers.shared
        guard active else { observers.remove(scene: sceneID); return }
        var owned = Set(recent.filter(\.hasParkedSession).map(\.profile))
        if let current { owned.insert(current.connection.profile) }
        var live = Set(recent.filter(\.isObservingParkedRoster).map(\.profile))
        if let connection = current?.connection, currentReady { live.insert(connection.profile) }
        observers.update(scene: sceneID, owned: owned, live: live)
    }

    private func discard(_ matching: (MobileConnection) -> Bool) {
        for connection in recent.filter(matching) { discard(connection) }
    }

    private func discard(_ connection: MobileConnection) {
        objectWillChange.send()
        if current?.connection === connection { current = nil }
        recent.removeAll { $0 === connection }
        connection.parkedRosterChanged = nil
        connection.leave()
    }
}
