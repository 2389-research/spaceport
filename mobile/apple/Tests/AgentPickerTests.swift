// ABOUTME: Checks local pane search and bounded, boot-scoped terminal peeks.
// ABOUTME: Includes shell panes, titles, recent visits, and unseen terminal output.
import SwiftUI
import XCTest
@testable import SpaceportMobile

final class AgentPickerTests: XCTestCase {
    @MainActor
    func testSidebarSummaryUsesTheNonemptyRosterGroups() {
        let workspace = MobileWorkspace(id: "w", label: "Mission", status: "idle")
        let tab = MobileTab(id: "t", workspaceId: "w", label: "1", status: "idle")
        let agents = ["idle", "working", "idle"].enumerated().map { index, status in
            GoToPane(workspace: workspace, tab: tab, pane: MobilePane(stateChangeSeq: nil, agentName: "Scout",
                     id: "p\(index)", tabId: tab.id, label: "1", status: status))
        }
        XCTAssertEqual(AgentRail.summary(AgentRail.groups(agents)), "1 working · 2 idle")
        XCTAssertEqual(AgentRail.summary([]), "No agents")
    }

    @MainActor
    func testAgentRowShowsOnlyTheHostForShellTitles() {
        func title(_ value: String) -> String? {
            var label = AgentSelectorLabel(name: "codex", terminalTitle: value, workspace: "app", tab: "3", font: .body)
            label.address = SSHAddress(hostname: "alias", port: 22, username: "pilot")
            return label.title
        }
        XCTAssertEqual(title("pilot@box: ~/src"), "box")
        XCTAssertEqual(title("pilot@box"), "box")
        XCTAssertEqual(title("pilot@alias"), "alias")
        XCTAssertEqual(title("Review pilot@alias"), "Review pilot@alias")
        XCTAssertEqual(title("pilot@example.com"), "pilot@example.com")
    }

    @MainActor
    func testHeadingStartsWithPaneTitleOrTabLabel() {
        func label(_ tab: String, pane: String? = nil) -> AgentSelectorLabel {
            AgentSelectorLabel(name: "codex", terminalTitle: "Fix sidebar", workspace: "app",
                               tab: tab, font: .body, paneTitle: pane)
        }
        let named = label("3", pane: "Review")
        XCTAssertEqual(String(named.heading.characters), "Review · Fix sidebar")
        XCTAssertEqual(String(named.subtitle(includingName: true).characters), "app › 3 · codex")
        XCTAssertEqual(String(label("3").heading.characters), "3 · Fix sidebar")
        XCTAssertEqual(String(label("Build").heading.characters), "Build · Fix sidebar")
        XCTAssertEqual(String(label("Fix sidebar").heading.characters), "Fix sidebar")
        XCTAssertEqual(String(label("").heading.characters), "Fix sidebar")
    }

    @MainActor
    func testAgentSubtitleShowsWorkspaceAndAgentBelowTheHeading() {
        func subtitle(_ workspace: String, _ tab: String, _ name: String) -> String {
            let label = AgentSelectorLabel(name: name, terminalTitle: "Fix sidebar", workspace: workspace, tab: tab, font: .body)
            return String(label.subtitle(includingName: true).characters)
        }
        XCTAssertEqual(subtitle("artifact-host", "3", "codex"), "artifact-host · codex")
        XCTAssertEqual(subtitle("", "", "codex"), "codex")
        XCTAssertEqual(subtitle("spaceport", "", ""), "spaceport")
    }

    @MainActor
    func testLocationSeparatorAndAgentHaveDistinctColors() {
        let label = AgentSelectorLabel(name: "codex", terminalTitle: nil, workspace: "codex", tab: "3", font: .body)
        let subtitle = label.subtitle(includingName: true)
        let runs = subtitle.runs.map { (String(subtitle[$0.range].characters), $0.swiftUI.foregroundColor) }
        XCTAssertEqual(runs.map { $0.0 }, ["codex", " · ", "codex"])
        XCTAssertEqual(Set(runs.compactMap { $0.1 }).count, 3)
        let location = label.subtitle(includingName: false)
        XCTAssertEqual(String(location.characters), "codex")
        XCTAssertEqual(location.runs.first?.swiftUI.foregroundColor, runs.first?.1)
    }

    @MainActor
    func testHomeContextHasSeparateInksAndPreservesLabelPunctuation() {
        let label = AgentSelectorLabel(name: "Scout", terminalTitle: "Build", workspace: "repo › fork", tab: "3",
                                       host: "pilot@host", session: "dev · main", font: .body)
        let text = label.subtitle(includingName: true)
        let parts = text.runs.filter { $0.swiftUI.foregroundColor != .secondary }
        XCTAssertEqual(parts.map { String(text[$0.range].characters) }, ["repo › fork", "pilot@host", "dev · main", "Scout"])
        XCTAssertEqual(Set(parts.compactMap { $0.swiftUI.foregroundColor }).count, parts.count)
    }

    func testSavedLabelsSurviveRelaunchAndOlderCachesKeepTheirText() throws {
        let workspace = "🚀 › flight", tab = "\u{0301}docs › build"
        let view = MobileView(version: 1, bootId: "boot", workspaces: [.init(id: "w", label: workspace, status: "idle")],
                              tabs: [.init(id: "t", workspaceId: "w", label: tab, status: "idle")],
                              panes: [.init(stateChangeSeq: nil, agentName: "Reviewer", id: "p", tabId: "t", label: "Build",
                                            status: "idle", agentIdentity: "claude")], paneMap: nil,
                              focusedWorkspace: "w", focusedTab: "t", focusedPane: "p", terminal: nil, inputTarget: nil,
                              error: nil, disconnected: false, hostChallenge: nil, goToPending: false, goToArrival: nil)
        let seen = try XCTUnwrap(SessionStatusSnapshot(view: view).agents?.first)
        let restored = try JSONDecoder().decode(SeenAgent.self, from: JSONEncoder().encode(seen))
        XCTAssertEqual(restored.locationLabels.workspace, workspace)
        XCTAssertEqual(restored.locationLabels.tab, tab)
        XCTAssertEqual(restored.agentIdentity, "claude")
        XCTAssertEqual(restored.paneTitle, "Build")
        let legacy = Data(#"{"bootId":"b","workspaceId":"w","tabId":"t","paneId":"p","name":"Scout","status":"working","location":"Flight › Checklist"}"#.utf8)
        let old = try JSONDecoder().decode(SeenAgent.self, from: legacy)
        XCTAssertEqual(old.locationLabels.workspace, "Flight")
        XCTAssertEqual(old.locationLabels.tab, "Checklist")
        XCTAssertNil(old.agentIdentity)
        XCTAssertNil(old.paneTitle)
    }

    func testSearchIncludesShellsAndMatchesWordsAcrossTitlesAndPaths() {
        let workspace = MobileWorkspace(id: "w", label: "Mission", status: "idle")
        let tab = MobileTab(id: "t", workspaceId: "w", label: "Rélease", status: "idle")
        let pane = MobilePane(stateChangeSeq: nil, agentName: "Scout", id: "a", tabId: "t",
                              label: "Review", status: "working", terminalTitle: "Launch checklist")
        let shell = MobilePane(stateChangeSeq: nil, agentName: nil, id: "b", tabId: "t",
                               label: "Mission shell", status: "unknown")
        func ids(_ query: String, recent: [String] = []) -> [String] {
            GoToPane.search(query, workspaces: [workspace], tabs: [tab], panes: [pane, shell], recent: recent).map(\.id)
        }
        XCTAssertEqual(ids("RELEASE lauNCh"), ["a"])
        XCTAssertEqual(ids("mission shell"), ["b"])
        XCTAssertEqual(ids("missing"), [])
        XCTAssertEqual(ids("release", recent: ["b"]), ["b", "a"])
    }

    func testSwitchBackKeepsTwoWholeViewportsWithinOneBoot() {
        var cache = PanePreviewCache()
        let first = (1...30).map { "row \($0)" }.joined(separator: "\n")
        cache.record(first, pane: "a", boot: "boot")
        cache.record("second", pane: "b", boot: "boot")
        XCTAssertEqual(cache.viewport(pane: "a", boot: "boot"), first)
        XCTAssertNil(cache.viewport(pane: "c", boot: "boot"))
        XCTAssertNil(cache.viewport(pane: "a", boot: "other"))

        cache.record("first updated", pane: "a", boot: "boot")
        cache.record("third", pane: "c", boot: "boot")
        XCTAssertNil(cache.viewport(pane: "b", boot: "boot"))
        XCTAssertEqual(cache.viewport(pane: "a", boot: "boot"), "first updated")
        cache.record(String(repeating: "🚀", count: 65_536), pane: "a", boot: "boot")
        XCTAssertNil(cache.viewport(pane: "a", boot: "boot"))
        cache.record("Claude's output", pane: "reused", boot: "boot", agent: "Claude")
        XCTAssertEqual(cache.viewport(pane: "reused", boot: "boot", agent: "Claude"), "Claude's output")
        XCTAssertNil(cache.viewport(pane: "reused", boot: "boot", agent: "Codex"))
        XCTAssertNil(cache.viewport(pane: "reused", boot: "boot"))
        cache.visit("c", boot: "restarted")
        XCTAssertNil(cache.viewport(pane: "c", boot: "boot"))
    }

    func testPreviewClipsOutputAndEvictsOlderPanes() throws {
        var cache = PanePreviewCache(capacity: 2)
        let seen = Date(timeIntervalSince1970: 123)
        cache.record("first", pane: "a", boot: "old", at: seen)
        XCTAssertNil(cache.lastSeen(pane: "a", boot: "new"))
        cache.record("first", pane: "a", boot: "new", at: seen)
        cache.record("second", pane: "b", boot: "new", at: seen)
        cache.record((1...12).map { "line \($0)" }.joined(separator: "\n") + "\n\n", pane: "c", boot: "new", at: seen)
        XCTAssertNil(cache.lastSeen(pane: "a", boot: "new"))
        let last = try XCTUnwrap(cache.lastSeen(pane: "c", boot: "new"))
        XCTAssertEqual(last.seenAt, seen)
        XCTAssertFalse(last.text.contains("line 1\n"))
        XCTAssertTrue(last.text.hasSuffix("line 12"))
        cache.record("  \n ", pane: "c", boot: "new", at: seen)
        XCTAssertNil(cache.lastSeen(pane: "c", boot: "new"))
        let long = try XCTUnwrap(PanePreviewCache.excerpt(String(repeating: "🚀", count: 10_000)))
        XCTAssertLessThanOrEqual(long.utf8.count, 4096)
    }
}
