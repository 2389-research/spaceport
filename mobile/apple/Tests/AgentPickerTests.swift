// ABOUTME: Checks local pane search and bounded, boot-scoped terminal peeks.
// ABOUTME: Includes shell panes, titles, recent visits, and unseen terminal output.
import XCTest
@testable import SpaceportMobile

final class AgentPickerTests: XCTestCase {
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
