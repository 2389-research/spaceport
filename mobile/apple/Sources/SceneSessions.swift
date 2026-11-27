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
    private var enrollment: AnyCancellable?
    @Published private var disconnected: Set<UUID> = []
    @Published private var forgottenPins: Set<String> = []
    private var recent: [MobileConnection] = []
    @Published private(set) var current: SessionPresentation?

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
        Task { @MainActor in owned.forEach { $0.leave() } }
    }

    func contains(_ profile: HostProfile) -> Bool { recent.contains { $0.profile == profile } }

    func permitsHomeRefresh(_ profile: HostProfile) -> Bool {
        !disconnected.contains(profile.id) && !forgottenPins.contains(profile.pinAccount)
    }

    func hasParkedSession(for profile: HostProfile) -> Bool {
        recent.contains { $0.profile == profile && $0.presentationID == nil && $0.hasParkedSession }
    }

    func observeRetainedRoster(for profile: HostProfile) async throws -> MobileView? {
        guard current == nil, permitsHomeRefresh(profile) else { throw CancellationError() }
        let roster: MobileView?
        do {
            roster = try await recent.first { $0.profile == profile }?.observeParkedRoster()
        } catch MobileError.Disconnected {
            roster = nil
        }
        try Task.checkCancellation()
        guard current == nil, permitsHomeRefresh(profile) else { throw CancellationError() }
        return roster
    }

    func connection(for profile: HostProfile) -> MobileConnection {
        discard { $0.profile.id == profile.id && $0.profile != profile }
        if let index = recent.firstIndex(where: { $0.profile == profile }) {
            let connection = recent.remove(at: index)
            recent.append(connection)
            return connection
        }
        if recent.count == 2 { discard(recent[0]) }
        let connection = MobileConnection(profile: profile)
        recent.append(connection)
        return connection
    }

    func open(_ profile: HostProfile, agent: SeenAgent?) -> SessionPresentation {
        disconnected.remove(profile.id)
        forgottenPins.remove(profile.pinAccount)
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
        connection.present(presentation.id, agent: agent, alreadyOpen: false)
        return presentation
    }

    func detach(_ presentation: SessionPresentation) {
        guard presentation == current else { return }
        current = nil
        let connection = presentation.connection
        if !connection.keepOnHome() { discard(connection) }
    }

    func disconnect(_ profile: HostProfile) {
        disconnected.insert(profile.id)
        discard { $0.profile == profile }
    }

    func reconcile(_ profiles: [HostProfile]) {
        let saved = Set(profiles)
        disconnected.formIntersection(profiles.map(\.id))
        forgottenPins.formIntersection(profiles.map(\.pinAccount))
        discard { !saved.contains($0.profile) }
    }

    func forget(pinAccount: String) {
        forgottenPins.insert(pinAccount)
        discard { $0.profile.pinAccount == pinAccount }
    }

    func scenePhaseChanged(_ phase: ScenePhase) {
        if phase == .background {
            for connection in recent { connection.background() }
        } else if phase == .inactive {
            current?.connection.sceneInactive()
        } else if phase == .active {
            current?.connection.foreground()
        }
    }

    private func discard(_ matching: (MobileConnection) -> Bool) {
        for connection in recent.filter(matching) { discard(connection) }
    }

    private func discard(_ connection: MobileConnection) {
        objectWillChange.send()
        if current?.connection === connection { current = nil }
        recent.removeAll { $0 === connection }
        connection.leave()
    }
}
