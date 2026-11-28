// ABOUTME: Checks that a Home refresh updates the saved roster without changing the resume destination.
// ABOUTME: Exercises stale observations and edits to a saved SSH session.
import XCTest
@testable import SpaceportMobile

@MainActor
final class HomeRosterRefreshTests: XCTestCase {
    func testObservingAnOlderSessionKeepsResumeAndWidgetPointedAtLastOpen() throws {
        let suite = "spaceport.home-refresh.\(UUID())"
        let defaults = try XCTUnwrap(UserDefaults(suiteName: suite))
        defer { defaults.removePersistentDomain(forName: suite) }
        let store = ProfileStore(defaults: defaults)
        let first = try HostProfile.make(hostname: "box", port: "22", username: "qa", session: "first")
        let last = try HostProfile.make(hostname: "box", port: "22", username: "qa", session: "last")
        func observation(_ time: Double) -> SessionStatusSnapshot {
            SessionStatusSnapshot(seenAt: Date(timeIntervalSince1970: time), blocked: 0, working: 0,
                                  hasIdle: false, location: [], agents: [], agentCounts: AgentCounts())
        }
        try store.save(first)
        try store.save(last)
        try store.recordSeen(profile: first, snapshot: observation(1))
        try store.recordSeen(profile: last, snapshot: observation(2))
        try store.recordSeen(profile: first, snapshot: observation(4), source: .home(previouslySeen: observation(1).seenAt))
        try store.recordSeen(profile: first, snapshot: observation(3), source: .home(previouslySeen: observation(4).seenAt))
        let catalog = try store.catalog()
        XCTAssertEqual(catalog.lastOpenedSession?.id, last.id)
        XCTAssertEqual(catalog.agentWidgetSnapshot?.destinationID, last.id)
        XCTAssertEqual(catalog.sessions.first { $0.id == first.id }?.lastStatus, observation(4))
        try store.recordSeen(profile: first, snapshot: observation(3))
        XCTAssertEqual(try store.catalog().lastOpenedSession?.id, first.id)
        XCTAssertEqual(try store.catalog().sessions.first { $0.id == first.id }?.lastStatus, observation(4))

        // Another scene gets a fresh snapshot while Home is waiting for its reply.
        try store.recordSeen(profile: first, snapshot: observation(6))
        try store.recordSeen(profile: first, snapshot: observation(7), source: .home(previouslySeen: observation(4).seenAt))
        XCTAssertEqual(try store.catalog().sessions.first { $0.id == first.id }?.lastStatus, observation(6))
        var changed = first
        changed.session = "renamed"
        try store.save(changed)
        try store.recordSeen(profile: first, snapshot: observation(5), source: .home(previouslySeen: observation(4).seenAt))
        XCTAssertNil(try store.catalog().sessions.first { $0.id == first.id }?.lastStatus)
    }
}
