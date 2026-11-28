// ABOUTME: Verifies the private payload and lifecycle of the selected session’s Live Activity.
// ABOUTME: Advances observation time directly to test refreshes and suspension.
import ActivityKit
import XCTest
@testable import SpaceportMobile

final class AgentLiveActivityTests: XCTestCase {
    private let start = Date(timeIntervalSince1970: 200)
    private let counts = AgentCounts(waiting: 1, working: 0, idle: 3)

    func testPayloadCountsOnlyReachableNamedAgentsAndContainsNoSessionDetails() throws {
        let statuses: [(String?, String, String)] = [("a", "blocked", "t"), ("b", "working", "t"),
            ("c", "idle", "t"), ("d", "done", "t"), ("e", "unknown", "t"),
            (nil, "working", "t"), ("f", "working", "missing")]
        let view = MobileView(version: 1, bootId: "boot", workspaces: [.init(id: "w", label: "private", status: "")],
            tabs: [.init(id: "t", workspaceId: "w", label: "secret", status: "")],
            panes: statuses.enumerated().map { index, item in
                MobilePane(stateChangeSeq: nil, agentName: item.0, id: String(index), tabId: item.2,
                           label: "private pane", status: item.1, terminalTitle: "secret title")
            }, paneMap: nil, focusedWorkspace: "w", focusedTab: "t", focusedPane: "0",
            terminal: nil, inputTarget: nil, error: nil, disconnected: false, hostChallenge: nil,
            goToPending: false, goToArrival: nil)
        let counts = AgentCounts(view: view)
        XCTAssertEqual(counts, AgentCounts(waiting: 1, working: 1, idle: 2))
        let state = AgentStatusAttributes.ContentState(counts: counts, observedAt: start, paused: false)
        let data = try JSONEncoder().encode(state)
        let payload = try XCTUnwrap(JSONSerialization.jsonObject(with: data) as? [String: Any])
        XCTAssertEqual(Set(payload.keys), ["counts", "observedAt", "paused"])
        let aggregate = try XCTUnwrap(payload["counts"] as? [String: Int])
        XCTAssertEqual(aggregate, ["waiting": 1, "working": 1, "idle": 2])
    }

    @MainActor
    func testTurningOffEndsTheActivityAndTurningOnUsesCurrentCounts() async throws {
        let suite = "live-activity-tests.\(UUID())"
        let defaults = try XCTUnwrap(UserDefaults(suiteName: suite))
        defer { defaults.removePersistentDomain(forName: suite) }
        let manager = AgentLiveActivity(defaults: defaults)
        XCTAssertTrue(manager.isEnabled)
        await manager.setEnabled(false)?.value
        let id = UUID()
        let view = MobileView(version: 1, bootId: "boot", workspaces: [.init(id: "w", label: "", status: "")],
            tabs: [.init(id: "t", workspaceId: "w", label: "", status: "")],
            panes: [.init(stateChangeSeq: nil, agentName: "worker", id: "p", tabId: "t", label: "", status: "working")],
            paneMap: nil, focusedWorkspace: "w", focusedTab: "t", focusedPane: "p", terminal: nil, inputTarget: nil,
            error: nil, disconnected: false, hostChallenge: nil, goToPending: false, goToArrival: nil)
        manager.select(id, view: view, observedAt: Date())
        await manager.setEnabled(false)?.value
        typealias Status = Activity<AgentStatusAttributes>
        XCTAssertTrue(Status.activities.isEmpty)
        try XCTSkipUnless(ActivityAuthorizationInfo().areActivitiesEnabled, "Live Activities are disabled in iOS.")
        await manager.setEnabled(true)?.value
        let live = try XCTUnwrap(Status.activities.first)
        XCTAssertEqual(live.content.state.counts.working, 1)
        let paused = expectation(description: "The same Live Activity receives the paused observation")
        let updates = live.contentUpdates
        let observer = Task {
            for await content in updates where content.state.paused {
                paused.fulfill()
                return
            }
        }
        defer { observer.cancel() }
        manager.pause(id)
        let received = await XCTWaiter.fulfillment(of: [paused], timeout: 5)
        XCTAssertEqual(received, .completed)
        await manager.setEnabled(false)?.value
        XCTAssertFalse(Status.activities.contains { $0.id == live.id })
        XCTAssertFalse(AgentLiveActivity(defaults: defaults).isEnabled)
    }

    func testParkingPublishesLatestRealObservationAndResumeRequiresAnotherObservation() throws {
        var feed = AgentActivityFeed()
        let id = UUID()
        feed.select(id)
        feed.observe(counts, at: start, from: id)
        let first = try XCTUnwrap(feed.content)
        XCTAssertGreaterThan(first.staleDate, start)
        let next = start.addingTimeInterval(1)
        feed.observe(counts, at: next, from: id)
        XCTAssertEqual(feed.content, first)
        feed.pause(id)
        XCTAssertEqual(feed.content?.observedAt, next)
        XCTAssertEqual(feed.content?.staleDate, next)
        feed.observe(counts, at: next.addingTimeInterval(1), from: id)
        XCTAssertEqual(feed.content?.paused, false)
        let later = first.staleDate.addingTimeInterval(1)
        feed.observe(counts, at: later, from: id)
        XCTAssertEqual(feed.content?.observedAt, later)
    }

    func testSwitchIgnoresPreviousPresentationAndDisconnectClearsCounts() {
        var feed = AgentActivityFeed()
        let previous = UUID(), selected = UUID()
        feed.select(previous)
        feed.observe(counts, at: start, from: previous)
        feed.select(selected)
        XCTAssertNil(feed.content)
        feed.observe(counts, at: start, from: previous)
        XCTAssertNil(feed.content)
        feed.observe(counts, at: start, from: selected)
        feed.pause(previous)
        feed.end(previous)
        XCTAssertEqual(feed.content?.paused, false)
        let changed = AgentCounts(waiting: 0, working: 1, idle: 3)
        feed.observe(changed, at: start.addingTimeInterval(1), from: selected)
        XCTAssertEqual(feed.content?.counts, changed)
        feed.end(selected)
        XCTAssertNil(feed.content)
        feed.observe(counts, at: start, from: selected)
        XCTAssertNil(feed.content)
    }
}
