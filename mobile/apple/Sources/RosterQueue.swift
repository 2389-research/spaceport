// ABOUTME: Shares the phone's saved roster targets across active window scenes.
// ABOUTME: Rotates a bounded set of readers and backs off hosts that fail.
import Foundation

struct RosterClaims {
    private struct Scene {
        var owned: Set<HostProfile>
        var live: Set<HostProfile>
    }
    private var profiles: [HostProfile] = []
    private var scenes: [UUID: Scene] = [:]
    private(set) var paused: Set<HostProfile> = []

    mutating func replaceCatalog(_ profiles: [HostProfile]) {
        self.profiles = profiles
        paused.formIntersection(profiles)
    }

    mutating func update(_ scene: UUID, owned: Set<HostProfile>, live: Set<HostProfile> = []) {
        scenes[scene] = Scene(owned: owned, live: live)
    }

    mutating func remove(_ scene: UUID) { scenes.removeValue(forKey: scene) }
    mutating func pause(_ profile: HostProfile) { paused.insert(profile) }
    mutating func resume(_ profile: HostProfile) { paused.remove(profile) }

    var live: Set<HostProfile> { scenes.values.reduce(into: []) { $0.formUnion($1.live) } }

    var desired: [HostProfile] {
        guard !scenes.isEmpty else { return [] }
        let excluded = scenes.values.reduce(into: paused) { $0.formUnion($1.owned) }
        return profiles.filter { $0.login == .phoneKey && !excluded.contains($0) }
    }
}

struct RosterQueue {
    private static let capacity = 4
    private var waiting: [HostProfile] = []
    private var retryAfter: [HostProfile: ContinuousClock.Instant] = [:]
    private(set) var running: Set<HostProfile> = []

    mutating func replace(_ profiles: [HostProfile]) {
        let wanted = Set(profiles)
        waiting.removeAll { !wanted.contains($0) }
        running.formIntersection(wanted)
        retryAfter = retryAfter.filter { wanted.contains($0.key) }
        var known = Set(waiting).union(running)
        waiting.append(contentsOf: profiles.filter { known.insert($0).inserted })
    }

    func hasWaiter(now: ContinuousClock.Instant) -> Bool {
        waiting.contains { retryAfter[$0].map { $0 <= now } ?? true }
    }

    var nextRetry: ContinuousClock.Instant? {
        running.count < Self.capacity ? waiting.compactMap { retryAfter[$0] }.min() : nil
    }

    mutating func start(now: ContinuousClock.Instant) -> [HostProfile] {
        let eligible = waiting.filter { retryAfter[$0].map { $0 <= now } ?? true }
        let started = Array(eligible.prefix(Self.capacity - running.count))
        running.formUnion(started)
        waiting.removeAll { running.contains($0) }
        for profile in started { retryAfter.removeValue(forKey: profile) }
        return started
    }

    mutating func finish(_ profile: HostProfile, failed: Bool, now: ContinuousClock.Instant) {
        guard running.remove(profile) != nil else { return }
        if failed { retryAfter[profile] = now.advanced(by: .seconds(30)) }
        waiting.append(profile)
    }
}
