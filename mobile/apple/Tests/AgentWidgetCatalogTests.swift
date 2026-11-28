// ABOUTME: Checks the Home Screen widget's counts against the full Herdr roster.
// ABOUTME: Exercises saved session selection through catalog edits and deletions.
import XCTest
@testable import SpaceportMobile

@MainActor
final class AgentWidgetCatalogTests: XCTestCase {
    func testRosterChangesAreSavedWithoutRewritingQuietViews() throws {
        var view = MobileView(version: 1, bootId: "boot", workspaces: [.init(id: "w", label: "", status: "")],
                              tabs: [.init(id: "t", workspaceId: "w", label: "", status: "")], panes: [],
                              paneMap: nil, focusedWorkspace: "w", focusedTab: "t", focusedPane: nil, terminal: nil,
                              inputTarget: nil, error: nil, disconnected: false, hostChallenge: nil,
                              goToPending: false, goToArrival: nil)
        XCTAssertNil(view.changedAgentStatus(since: nil))
        view.rosterObservedAtMs = 2_500
        let first = try XCTUnwrap(view.changedAgentStatus(since: nil))
        XCTAssertEqual(first.seenAt, Date(timeIntervalSince1970: 2.5))
        XCTAssertEqual(first.agentCounts, AgentCounts())
        let empty = view
        view.version += 1
        view.rosterObservedAtMs = 3_000
        XCTAssertNil(view.changedAgentStatus(since: RosterSaveAttempt(empty, accepted: true)))
        view.rosterObservedAtMs = 62_500
        XCTAssertNotNil(view.changedAgentStatus(since: RosterSaveAttempt(empty, accepted: true)))
        view.rosterObservedAtMs = 3_000
        view.tabs[0].zoomed.toggle()
        XCTAssertNil(view.changedAgentStatus(since: RosterSaveAttempt(empty, accepted: true)))
        view.panes = [.init(stateChangeSeq: nil, agentName: "Builder", id: "p", tabId: "t", label: "", status: "working")]
        let busy = try XCTUnwrap(view.changedAgentStatus(since: RosterSaveAttempt(empty, accepted: true)))
        XCTAssertEqual(busy.agentCounts, AgentCounts(working: 1))
        XCTAssertEqual(busy.seenAt, Date(timeIntervalSince1970: 3))
        let named = view
        view.panes[0].label = "Compile"
        XCTAssertEqual(view.changedAgentStatus(since: RosterSaveAttempt(named, accepted: true))?.agents?.first?.paneTitle, "Compile")
        view.bootId = "replacement"
        XCTAssertNotNil(view.changedAgentStatus(since: RosterSaveAttempt(named, accepted: true)))
    }

    func testSavedWaitingAgentFollowsASwapWithUnchangedCounts() throws {
        let suite = "spaceport.waiting-swap." + UUID().uuidString
        let defaults = try XCTUnwrap(UserDefaults(suiteName: suite))
        defer { defaults.removePersistentDomain(forName: suite) }
        let store = ProfileStore(defaults: defaults)
        let profile = try HostProfile.make(hostname: "box", port: "22", username: "pilot", session: "test")
        try store.save(profile)
        var view = MobileView(version: 1, bootId: "boot", workspaces: [.init(id: "w", label: "Lab", status: "")],
                              tabs: [.init(id: "t", workspaceId: "w", label: "Console", status: "")],
                              panes: [.init(stateChangeSeq: 1, agentName: "Claude", id: "a", tabId: "t", label: "", status: "blocked"),
                                      .init(stateChangeSeq: 2, agentName: "Codex", id: "b", tabId: "t", label: "", status: "idle")],
                              paneMap: nil, focusedWorkspace: "w", focusedTab: "t", focusedPane: "a", terminal: nil,
                              inputTarget: nil, error: nil, disconnected: false, hostChallenge: nil,
                              goToPending: false, goToArrival: nil)
        view.rosterObservedAtMs = 1_000
        let first = try XCTUnwrap(view.changedAgentStatus(since: nil))
        XCTAssertTrue(try store.recordSeen(profile: profile, snapshot: first))
        let previous = view
        view.panes[0].status = "idle"
        view.panes[1].status = "blocked"
        view.rosterObservedAtMs = 2_000
        if let next = view.changedAgentStatus(since: RosterSaveAttempt(previous, accepted: true)) {
            try store.recordSeen(profile: profile, snapshot: next)
        }
        let accepted = try store.recordSeen(profile: profile, snapshot: first)
        XCTAssertFalse(accepted)
        let lostRace = RosterSaveAttempt(previous, accepted: accepted)
        XCTAssertNil(previous.changedAgentStatus(since: lostRace))
        var fresh = previous
        fresh.rosterObservedAtMs = 3_000
        XCTAssertNotNil(fresh.changedAgentStatus(since: lostRace))
        let catalog = try ProfileStore(defaults: defaults).catalog()
        XCTAssertEqual(catalog.observedAgents.filter { $0.section == .waiting }.map(\.agent.name), ["Codex"])
        XCTAssertEqual(catalog.agentWidgetSnapshot?.counts, first.agentCounts)
    }

    func testWidgetUsesTheFullNamedRosterBeforeSavingTheBoundedCache() throws {
        let suite = "spaceport.widget-catalog.\(UUID())"
        let defaults = try XCTUnwrap(UserDefaults(suiteName: suite))
        defer { defaults.removePersistentDomain(forName: suite) }
        let store = ProfileStore(defaults: defaults)
        let main = try HostProfile.make(hostname: "box", port: "22", username: "biz", session: "main")
        try store.save(main)
        var panes = (0..<SeenAgent.cacheLimit).map {
            MobilePane(stateChangeSeq: nil, agentName: "Agent", id: String($0), tabId: "t", label: "", status: "idle")
        }
        panes.append(contentsOf: [
            .init(stateChangeSeq: nil, agentName: "Done", id: "d", tabId: "t", label: "", status: "done"),
            .init(stateChangeSeq: nil, agentName: "Busy", id: "b", tabId: "t", label: "", status: "working"),
            .init(stateChangeSeq: nil, agentName: "Waiting", id: "w", tabId: "t", label: "", status: "blocked"),
            .init(stateChangeSeq: nil, agentName: nil, id: "shell", tabId: "t", label: "", status: "working"),
            .init(stateChangeSeq: nil, agentName: "Orphan", id: "orphan", tabId: "gone", label: "", status: "blocked"),
            .init(stateChangeSeq: nil, agentName: "Unknown", id: "u", tabId: "t", label: "", status: "unknown")
        ])
        let view = MobileView(version: 1, bootId: "boot", workspaces: [.init(id: "s", label: "", status: "")],
                              tabs: [.init(id: "t", workspaceId: "s", label: "", status: "")], panes: panes,
                              paneMap: nil, focusedWorkspace: "s", focusedTab: "t", focusedPane: "b", terminal: nil,
                              inputTarget: nil, error: nil, disconnected: false, hostChallenge: nil,
                              goToPending: false, goToArrival: nil)
        let date = Date(timeIntervalSince1970: 850)
        try store.recordSeen(profile: main, snapshot: SessionStatusSnapshot(view: view, seenAt: date))
        let reloaded = try ProfileStore(defaults: defaults).catalog()
        XCTAssertEqual(reloaded.sessions.first?.lastStatus?.agents?.count, SeenAgent.cacheLimit)
        let expected = AgentWidgetSnapshot(destinationID: main.id,
            counts: AgentCounts(waiting: 1, working: 1, idle: SeenAgent.cacheLimit + 1), observedAt: date)
        XCTAssertEqual(reloaded.agentWidgetSnapshot, expected)
        let row = try XCTUnwrap(reloaded.observedAgents.first)
        try store.mark(.pin(true), row: row)
        XCTAssertEqual(try store.catalog().agentWidgetSnapshot, expected)
    }

    func testLatestObservationSurvivesStaleWritesAndFallsBackAfterEditsAndDeletion() throws {
        let suite = "spaceport.widget-catalog.\(UUID())"
        let defaults = try XCTUnwrap(UserDefaults(suiteName: suite))
        defer { defaults.removePersistentDomain(forName: suite) }
        let store = ProfileStore(defaults: defaults)
        let a = try HostProfile.make(hostname: "a", port: "22", username: "biz", session: "work")
        var b = try HostProfile.make(hostname: "b", port: "22", username: "biz", session: "work")
        try store.save(a)
        try store.save(b)
        let time = Date(timeIntervalSinceReferenceDate: 1200)
        let legacy = try JSONDecoder().decode(SessionStatusSnapshot.self,
            from: Data(#"{"seenAt":0,"blocked":1,"working":2,"hasIdle":true,"location":[]}"#.utf8))
        try store.recordSeen(profile: a, snapshot: legacy)
        XCTAssertNil(try store.catalog().agentWidgetSnapshot)

        func observe(_ profile: HostProfile, at date: Date, _ counts: AgentCounts) throws {
            try store.recordSeen(profile: profile, snapshot: SessionStatusSnapshot(seenAt: date, blocked: 0,
                working: 0, hasIdle: false, location: [], agentCounts: counts))
        }
        let empty = AgentCounts()
        let busy = AgentCounts(working: 2)
        try observe(a, at: time, empty)
        let first = AgentWidgetSnapshot(destinationID: a.id, counts: empty, observedAt: time)
        XCTAssertEqual(try store.catalog().agentWidgetSnapshot, first)
        let later = time.addingTimeInterval(10)
        try observe(b, at: later, busy)
        try observe(b, at: time, empty)
        let originalLink = try XCTUnwrap(store.catalog().agentWidgetSnapshot?.destinationURL)
        XCTAssertEqual(try store.catalog().agentWidgetSnapshot,
                       AgentWidgetSnapshot(destinationID: b.id, counts: busy, observedAt: later))
        b.session = "renamed"
        try store.save(b)
        XCTAssertEqual(try store.catalog().agentWidgetSnapshot, first)
        try observe(b, at: later, busy)
        let renamedLink = try XCTUnwrap(store.catalog().agentWidgetSnapshot?.destinationURL)
        XCTAssertNotEqual(renamedLink, originalLink)
        b.hostname = "replacement"
        try store.save(b)
        try observe(b, at: later, busy)
        let edited = try store.catalog()
        let replacementLink = try XCTUnwrap(edited.agentWidgetSnapshot?.destinationURL)
        XCTAssertNotEqual(replacementLink, renamedLink)
        func openWidget(_ link: URL) throws -> HostProfile {
            let id = try XCTUnwrap(AgentWidgetSnapshot.destinationID(from: link))
            return try SpaceportShortcut.widgetNextWaiting(destination: id).destination(in: edited, active: a).profile
        }
        for oldLink in [originalLink, renamedLink] {
            XCTAssertThrowsError(try openWidget(oldLink)) { XCTAssertEqual($0 as? ShortcutFailure, .changedWidgetSession) }
        }
        XCTAssertEqual(try openWidget(replacementLink), b)
        try store.remove(b)
        XCTAssertEqual(try store.catalog().agentWidgetSnapshot, first)
        try store.remove(a)
        XCTAssertNil(try store.catalog().agentWidgetSnapshot)
    }
}
