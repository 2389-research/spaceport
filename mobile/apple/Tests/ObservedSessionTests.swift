// ABOUTME: Checks how foreground and passive host observations share the Home catalog.
// ABOUTME: Keeps an unseen terminal out of Resume Session and rejects stale roster updates.
import XCTest
@testable import SpaceportMobile

@MainActor
final class ObservedSessionTests: XCTestCase {
    func testPeerUpdatesHomeWithoutChangingTheResumeDestination() throws {
        let suite = "spaceport.multi-host.\(UUID())"
        let defaults = try XCTUnwrap(UserDefaults(suiteName: suite))
        defer { defaults.removePersistentDomain(forName: suite) }
        let store = ProfileStore(defaults: defaults)
        let active = try HostProfile.make(hostname: "one.invalid", port: "22", username: "qa", session: "main")
        let peer = try HostProfile.make(hostname: "two.invalid", port: "22", username: "qa", session: "main")
        try store.save(active)
        try store.save(peer)
        func snapshot(_ time: Double, _ count: Int) -> SessionStatusSnapshot {
            SessionStatusSnapshot(seenAt: Date(timeIntervalSince1970: time), blocked: count,
                                  working: 0, hasIdle: false, location: [], agents: [])
        }
        try store.recordSeen(profile: active, snapshot: snapshot(1, 0))
        try store.recordObserved(profile: peer, snapshot: snapshot(3, 2))
        try store.recordObserved(profile: peer, snapshot: snapshot(2, 1))
        let catalog = try store.catalog()
        let observed = try XCTUnwrap(catalog.sessions.first { $0.id == peer.id })
        XCTAssertEqual(observed.lastStatus?.blocked, 2)
        XCTAssertNil(observed.lastOpened)
        XCTAssertEqual(catalog.lastOpenedSession?.id, active.id)
    }
}
