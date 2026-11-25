// ABOUTME: Verifies the displayed navigation path and Herdr's status vocabulary.
// ABOUTME: Includes an unfocused workspace and a dangling pane reference.
import XCTest
@testable import SpaceportMobile

final class GoToTests: XCTestCase {
    func testAgentRosterResolvesAcrossWorkspacesAndRanksReportedStatus() {
        let workspaces = [
            MobileWorkspace(id: "a", label: "Agents", status: "blocked"),
            MobileWorkspace(id: "i", label: "Infra", status: "blocked")
        ]
        let tabs = [
            MobileTab(id: "build", workspaceId: "a", label: "Build", status: "working"),
            MobileTab(id: "dep", workspaceId: "i", label: "Deploy", status: "blocked")
        ]
        let panes = [
            MobilePane(stateChangeSeq: nil, agentName: "Codex", id: "unknown", tabId: "dep", label: "Investigate", status: "unknown"),
            MobilePane(stateChangeSeq: nil, agentName: "Codex", id: "working", tabId: "build", label: "Builder", status: "working"),
            MobilePane(stateChangeSeq: nil, agentName: nil, id: "shell", tabId: "dep", label: "Shell", status: "blocked"),
            MobilePane(stateChangeSeq: 4, agentName: "Codex", id: "done", tabId: "build", label: "Writer", status: "done"),
            MobilePane(stateChangeSeq: 17, agentName: "Scout", id: "recent-idle", tabId: "dep", label: "Scout", status: "idle"),
            MobilePane(stateChangeSeq: nil, agentName: "Codex", id: "waiting", tabId: "dep", label: "Review", status: "blocked"),
            MobilePane(stateChangeSeq: nil, agentName: "Codex", id: "orphan", tabId: "gone", label: "Detached", status: "blocked")
        ]
        let rows = GoToPane.agents(workspaces: workspaces, tabs: tabs, panes: panes)
        XCTAssertEqual(rows.map(\.id), ["working", "waiting", "recent-idle", "done", "unknown"])
        XCTAssertEqual(rows.first?.location, "Agents › Build")
        XCTAssertEqual(rows.first?.tab.id, "build")
        XCTAssertEqual(rows.first?.pane.agentName, "Codex")
        let groups = AgentRail.groups(rows)
        XCTAssertEqual(groups.flatMap(\.rows).map(\.id), ["waiting", "working", "recent-idle", "done", "unknown"])
        XCTAssertEqual(TerminalHeader.nextWaiting(rows, focused: "working")?.id, "waiting")
        XCTAssertNil(TerminalHeader.nextWaiting(rows, focused: "waiting"))
    }

    func testGoToConfirmationRequiresThePaneToOwnInput() {
        var view = MobileView(version: 8, bootId: "boot", workspaces: [], tabs: [], panes: [],
                              paneMap: nil, focusedWorkspace: "w", focusedTab: "t", focusedPane: "p",
                              terminal: nil, inputTarget: InputTarget(connection: 1, bootId: "boot", paneId: "p", popupId: nil),
                              error: nil, disconnected: false, hostChallenge: nil, goToPending: false, goToArrival: 7)
        XCTAssertTrue(view.confirmsInput(to: "p", revision: 7))
        XCTAssertFalse(view.confirmsInput(to: "p", revision: 6))
        view.inputTarget?.popupId = "overlay"
        XCTAssertFalse(view.confirmsInput(to: "p", revision: 7))
    }

    func testStatusUsesReportedActivityOnly() {
        XCTAssertEqual(PaneActivity("blocked")?.title, "Needs you")
        XCTAssertEqual(PaneActivity("done"), .idle)
        XCTAssertEqual(PaneActivity("idle"), .idle)
        XCTAssertEqual(PaneActivity("working"), .working)
        XCTAssertNil(PaneActivity("unknown"))
        XCTAssertNil(PaneActivity(""))
    }
}
