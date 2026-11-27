// ABOUTME: Observes saved phone-key sessions over verified, surface-inactive SSH bridges.
// ABOUTME: Shares readers across scenes and rejects late observations after a target changes.
import Combine
import Foundation

@MainActor
final class RosterObservers: ObservableObject {
    static let shared = RosterObservers()
    private var claims = RosterClaims()
    private var queue = RosterQueue()
    private struct Reader {
        let id: UUID
        let task: Task<Void, Never>
    }
    private var readers: [HostProfile: Reader] = [:]
    private var retry: Task<Void, Never>?
    private let store = ProfileStore()
    @Published private var passiveLive: Set<HostProfile> = []
    @Published private var interactiveLive: Set<HostProfile> = []

    func isLive(_ profile: HostProfile) -> Bool { passiveLive.contains(profile) || interactiveLive.contains(profile) }
    func canPause(_ profile: HostProfile) -> Bool { claims.desired.contains(profile) }

    func replaceCatalog(_ profiles: [HostProfile]) {
        claims.replaceCatalog(profiles)
        refresh()
    }

    func update(scene: UUID, owned: Set<HostProfile>, live: Set<HostProfile>) {
        claims.update(scene, owned: owned, live: live)
        refresh()
    }

    func remove(scene: UUID) {
        claims.remove(scene)
        refresh()
    }

    func pause(_ profile: HostProfile) {
        guard !claims.paused.contains(profile) else { return }
        objectWillChange.send()
        claims.pause(profile)
        refresh()
    }

    func resume(_ profile: HostProfile) {
        guard claims.paused.contains(profile) else { return }
        objectWillChange.send()
        claims.resume(profile)
        refresh()
    }

    private func refresh() {
        let desired = claims.desired
        queue.replace(desired)
        if interactiveLive != claims.live { interactiveLive = claims.live }
        for profile in readers.keys where !queue.running.contains(profile) {
            readers.removeValue(forKey: profile)?.task.cancel()
            if passiveLive.contains(profile) { passiveLive.remove(profile) }
        }
        schedule()
    }

    private func schedule() {
        retry?.cancel()
        retry = nil
        for profile in queue.start(now: .now) {
            let id = UUID()
            let task = Task.detached(priority: .utility) { [weak self] in
                guard let self else { return }
                let failed = await self.observe(profile, id: id)
                await self.finished(profile, id: id, failed: failed)
            }
            readers[profile] = Reader(id: id, task: task)
        }
        if let next = queue.nextRetry {
            retry = Task { [weak self] in
                do { try await Task.sleep(until: next, clock: .continuous) } catch { return }
                self?.schedule()
            }
        }
    }

    private nonisolated func observe(_ profile: HostProfile, id: UUID) async -> Bool {
        do {
            try Task.checkCancellation()
            let vault = DeviceVault()
            let host = try vault.host(for: profile.address)
            guard host.fingerprint != nil else { return true }
            let auth = try vault.securePhoneKey().auth
            try Task.checkCancellation()
            let reader = try MobileRoster.open(host: host, auth: auth, session: profile.session)
            defer { reader.disconnect() }
            let started = ContinuousClock.now
            var version: UInt64 = 0
            var saved: MobileView?
            while !Task.isCancelled {
                if let view = reader.poll(after: version) {
                    version = view.version
                    if view.disconnected { return true }
                    if let snapshot = view.observedRoster(after: saved) {
                        try await accept(snapshot, profile: profile, id: id)
                        saved = view
                    }
                }
                if started.duration(to: .now) >= .seconds(20) {
                    if saved == nil { return true }
                    if await queue.hasWaiter(now: .now) { return false }
                }
                try await Task.sleep(for: .milliseconds(500))
            }
        } catch { return true }
        return false
    }

    private func accept(_ snapshot: SessionStatusSnapshot, profile: HostProfile, id: UUID) throws {
        guard readers[profile]?.id == id else { return }
        try store.recordObserved(profile: profile, snapshot: snapshot)
        if !passiveLive.contains(profile) { passiveLive.insert(profile) }
    }

    private func finished(_ profile: HostProfile, id: UUID, failed: Bool) {
        guard readers[profile]?.id == id else { return }
        readers.removeValue(forKey: profile)
        if passiveLive.contains(profile) { passiveLive.remove(profile) }
        queue.finish(profile, failed: failed, now: .now)
        schedule()
    }
}
