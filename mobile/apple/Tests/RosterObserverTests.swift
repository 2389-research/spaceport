// ABOUTME: Exercises fair observer scheduling and foreground ownership across scenes.
// ABOUTME: Uses saved SSH profiles and an explicit clock without opening network connections.
import XCTest
@testable import SpaceportMobile

final class RosterObserverTests: XCTestCase {
    private func host(_ index: Int, login: LoginMethod = .phoneKey) throws -> HostProfile {
        try HostProfile.make(hostname: "host-\(index).invalid", port: "22", username: "test", session: "default", login: login)
    }

    func testDistinctSavedHostsShareSlotsAndFailuresLeaveOtherHostsEligible() throws {
        let profiles = try (1...5).map { try host($0) }
        var queue = RosterQueue()
        let now = ContinuousClock.now
        queue.replace(profiles)
        XCTAssertEqual(queue.start(now: now), Array(profiles.prefix(4)))
        XCTAssertTrue(queue.hasWaiter(now: now))
        queue.finish(profiles[0], failed: true, now: now)
        XCTAssertEqual(queue.start(now: now), [profiles[4]])
        XCTAssertFalse(queue.hasWaiter(now: now))
        queue.finish(profiles[1], failed: false, now: now)
        XCTAssertEqual(queue.start(now: now), [profiles[1]])
        XCTAssertTrue(queue.hasWaiter(now: now.advanced(by: .seconds(90))))
        queue.finish(profiles[2], failed: false, now: now.advanced(by: .seconds(90)))
        XCTAssertEqual(queue.start(now: now.advanced(by: .seconds(90))), [profiles[0]])
    }

    func testSceneClaimsExcludePasswordAndInteractiveSessionsUntilOwnershipChanges() throws {
        let a = try host(1), b = try host(2), password = try host(3, login: .password)
        var claims = RosterClaims()
        let first = UUID(), second = UUID()
        claims.replaceCatalog([a, b, password])
        XCTAssertTrue(claims.desired.isEmpty)
        claims.update(first, owned: [])
        claims.update(second, owned: [b])
        XCTAssertEqual(Set(claims.desired), [a])
        claims.remove(second)
        XCTAssertEqual(Set(claims.desired), [a, b])
        claims.pause(a)
        // A readiness change in another window cannot overwrite a catalog decision.
        claims.update(second, owned: [], live: [b])
        XCTAssertEqual(Set(claims.desired), [b])
        claims.remove(second)
        claims.update(first, owned: [])
        XCTAssertEqual(Set(claims.desired), [b])
        claims.remove(first)
        XCTAssertTrue(claims.desired.isEmpty)
        claims.update(second, owned: [])
        XCTAssertEqual(Set(claims.desired), [b])
        claims.resume(a)
        XCTAssertEqual(Set(claims.desired), [a, b])
    }
    func testUnchangedRosterRefreshesPeriodicallyAndAgentRenamesRefreshImmediately() throws {
        let pane = MobilePane(stateChangeSeq: nil, agentName: "First", id: "p", tabId: "t", label: "", status: "working")
        var view = MobileView(version: 1, bootId: "boot", workspaces: [.init(id: "w", label: "", status: "")],
                              tabs: [.init(id: "t", workspaceId: "w", label: "", status: "")], panes: [pane],
                              paneMap: nil, focusedWorkspace: "w", focusedTab: "t", focusedPane: "p", terminal: nil,
                              inputTarget: nil, error: nil, disconnected: false, hostChallenge: nil,
                              goToPending: false, goToArrival: nil, rosterObservedAtMs: 1_000)
        let first = view
        XCTAssertNotNil(view.observedRoster(after: nil))
        view.rosterObservedAtMs = 2_000
        XCTAssertNil(view.observedRoster(after: first))
        view.rosterObservedAtMs = 61_000
        XCTAssertEqual(view.observedRoster(after: first)?.seenAt, Date(timeIntervalSince1970: 61))
        view.rosterObservedAtMs = 3_000
        view.panes[0].agentName = "Second"
        XCTAssertEqual(view.observedRoster(after: first)?.agents?.first?.name, "Second")
    }

}
