// ABOUTME: Exercises pin and snooze against persisted observations from a saved session.
// ABOUTME: Checks the wake deadline, changed activity, and replacement panes.
import XCTest
@testable import SpaceportMobile

@MainActor
final class AgentMarksTests: XCTestCase {
    func testMarksPersistWakeAndStayWithTheObservedAgent() throws {
        let suite = "spaceport.marks.\(UUID())"
        let defaults = try XCTUnwrap(UserDefaults(suiteName: suite))
        defer { defaults.removePersistentDomain(forName: suite) }
        let store = ProfileStore(defaults: defaults)
        let profile = try HostProfile.make(hostname: "test", port: "22", username: "user", session: "main")
        try store.save(profile)
        let start = Date(timeIntervalSince1970: 1000)
        let end = start.addingTimeInterval(3600)
        func observe(_ status: String, at seconds: Double, boot: String = "A", sequence: UInt64 = 1) throws {
            let agent = SeenAgent(bootId: boot, workspaceId: "w", tabId: "t", paneId: "p", name: "Scout",
                                  status: status, location: "Work", stateChangeSeq: sequence)
            let snapshot = SessionStatusSnapshot(seenAt: start.addingTimeInterval(seconds), blocked: 0,
                                                working: 0, hasIdle: true, location: [], agents: [agent])
            try store.recordSeen(profile: profile, snapshot: snapshot)
        }
        func read() throws -> HomeAgent {
            try XCTUnwrap(ProfileStore(defaults: defaults).catalog().observedAgents.first)
        }
        try observe("idle", at: 0)
        let original = try read()
        try store.mark(.pin(true), row: original)
        try store.mark(.snooze(end), row: original)
        try observe("idle", at: 60)
        XCTAssertEqual(try read().displaySection(at: start), .snoozed)
        XCTAssertEqual(try read().displaySection(at: end), .pinned)
        try observe("working", at: 90, sequence: 2)
        XCTAssertEqual(try read().displaySection(at: start.addingTimeInterval(90)), .pinned)
        try store.mark(.pin(false), row: try read())
        XCTAssertEqual(try read().displaySection(at: end), .working)
        try observe("working", at: 100, boot: "B")
        XCTAssertThrowsError(try store.mark(.pin(true), row: original))
        XCTAssertEqual(try read().displaySection(at: end), .working)
    }

    func testNewAttentionSurfacesWhenTheEntireCachedRosterIsPinned() throws {
        let suite = "spaceport.mark-limit.\(UUID())"
        let defaults = try XCTUnwrap(UserDefaults(suiteName: suite))
        defer { defaults.removePersistentDomain(forName: suite) }
        let store = ProfileStore(defaults: defaults)
        let profile = try HostProfile.make(hostname: "test", port: "22", username: "user", session: "main")
        try store.save(profile)
        func agent(_ name: String, status: String) -> SeenAgent {
            SeenAgent(bootId: "boot", workspaceId: "w", tabId: "t", paneId: name,
                      name: name, status: status, location: "Build")
        }
        func record(_ rows: [SeenAgent], at date: Date) throws {
            try store.recordSeen(profile: profile, snapshot: SessionStatusSnapshot(
                seenAt: date, blocked: 0, working: 0, hasIdle: true, location: [], agents: rows))
        }
        let idle = (0..<SeenAgent.cacheLimit).map { agent("Idle \($0)", status: "idle") }
        let start = Date()
        try record(idle, at: start)
        for row in try store.catalog().observedAgents { try store.mark(.pin(true), row: row) }
        try record([agent("Reviewer", status: "blocked")] + idle, at: start.addingTimeInterval(1))
        let saved = try store.catalog().observedAgents
        XCTAssertEqual(saved.filter { $0.displaySection(at: start) == .waiting }.map(\.agent.name), ["Reviewer"])
        XCTAssertEqual(saved.filter { $0.displaySection(at: start) == .pinned }.count, idle.count)
    }
}
