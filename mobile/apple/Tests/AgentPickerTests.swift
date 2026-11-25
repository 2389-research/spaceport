// ABOUTME: Checks local pane search and bounded, boot-scoped terminal peeks.
// ABOUTME: Includes shell panes, titles, recent visits, and unseen terminal output.
import SwiftUI
import XCTest
@testable import SpaceportMobile

final class AgentPickerTests: XCTestCase {
    @MainActor
    func testAgentSubtitleLeadsWithLocation() {
        func subtitle(_ workspace: String, _ tab: String, _ name: String) -> String {
            let label = AgentSelectorLabel(name: name, terminalTitle: "Fix sidebar", workspace: workspace, tab: tab, font: .body)
            return String(label.subtitle(includingName: true).characters)
        }
        XCTAssertEqual(subtitle("artifact-host", "3", "codex"), "artifact-host › 3 · codex")
        XCTAssertEqual(subtitle("", "", "codex"), "codex")
        XCTAssertEqual(subtitle("spaceport", "", ""), "spaceport")
    }

    @MainActor
    func testLocationSeparatorAndAgentHaveDistinctColors() {
        let label = AgentSelectorLabel(name: "codex", terminalTitle: nil, workspace: "codex", tab: "3", font: .body)
        let subtitle = label.subtitle(includingName: true)
        let runs = subtitle.runs.map { (String(subtitle[$0.range].characters), $0.swiftUI.foregroundColor) }
        XCTAssertEqual(runs.map { $0.0 }, ["codex", " › ", "3", " · ", "codex"])
        XCTAssertEqual(Set(runs.compactMap { $0.1 }).count, 4)
        let location = label.subtitle(includingName: false)
        XCTAssertEqual(String(location.characters), "codex › 3")
        XCTAssertEqual(location.runs.first?.swiftUI.foregroundColor, runs.first?.1)
    }

    @MainActor
    func testHomeContextHasSeparateInksAndPreservesLabelPunctuation() {
        let label = AgentSelectorLabel(name: "Scout", terminalTitle: "Build", workspace: "repo › fork", tab: "3",
                                       host: "pilot@host", session: "dev · main", font: .body)
        let text = label.subtitle(includingName: true)
        let parts = text.runs.filter { $0.swiftUI.foregroundColor != .secondary }
        XCTAssertEqual(parts.map { String(text[$0.range].characters) }, ["repo › fork", "3", "pilot@host", "dev · main", "Scout"])
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
        let legacy = Data(#"{"bootId":"b","workspaceId":"w","tabId":"t","paneId":"p","name":"Scout","status":"working","location":"Flight › Checklist"}"#.utf8)
        let old = try JSONDecoder().decode(SeenAgent.self, from: legacy)
        XCTAssertEqual(old.locationLabels.workspace, "Flight")
        XCTAssertEqual(old.locationLabels.tab, "Checklist")
        XCTAssertNil(old.agentIdentity)
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
