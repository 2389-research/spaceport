// ABOUTME: Exercises workspace discovery and scoped pane lookup from a Herdr roster.
// ABOUTME: Checks that the terminal location stays independent from the workspace being browsed.
import XCTest
@testable import SpaceportMobile

final class WorkspacePickerTests: XCTestCase {
    let current = MobileWorkspace(id: "a", label: "Atlas", status: "")
    let other = MobileWorkspace(id: "b", label: "Relay", status: "")
    let empty = MobileWorkspace(id: "c", label: "Empty workspace", status: "")

    private var picker: WorkspacePicker {
        let tabs = [
            MobileTab(id: "a1", workspaceId: current.id, label: "Shell", status: ""),
            MobileTab(id: "b1", workspaceId: other.id, label: "Build", status: ""),
            MobileTab(id: "b2", workspaceId: other.id, label: "Shell", status: "")
        ]
        let panes = [
            MobilePane(stateChangeSeq: nil, agentName: nil, id: "p1", tabId: "a1", label: "Shell", status: ""),
            MobilePane(stateChangeSeq: nil, agentName: "Codex", id: "p2", tabId: "b1", label: "1", status: "working", terminalTitle: "Release checks")
        ]
        return WorkspacePicker(workspaces: [current, other, empty], tabs: tabs, panes: panes)
    }

    func testFindsEmptyWorkspaceByNameAndKeepsRosterOrder() {
        XCTAssertEqual(picker.workspaces(matching: "").map(\.id), [current.id, other.id, empty.id])
        XCTAssertEqual(picker.workspaces(matching: "EMPTY").map(\.id), [empty.id])
        XCTAssertTrue(picker.groups(in: empty, matching: "").isEmpty)
    }

    func testScopesPanesAndEmptyTabsToBrowsedWorkspace() {
        let groups = picker.groups(in: other, matching: "")
        XCTAssertEqual(groups.map(\.tab.id), ["b1", "b2"])
        XCTAssertEqual(groups.map { $0.panes.map(\.id) }, [["p2"], []])
        for name in ["release", "BUILD", "codex"] {
            XCTAssertEqual(picker.groups(in: other, matching: name).flatMap(\.panes).map(\.id), ["p2"])
            XCTAssertTrue(picker.groups(in: current, matching: name).isEmpty)
        }
        XCTAssertEqual(picker.current(workspaceID: current.id, tabID: "a1", paneID: "p1")?.id, "p1")
        XCTAssertNil(picker.current(workspaceID: other.id, tabID: "a1", paneID: "p1"))
    }
}
