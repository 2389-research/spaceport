// ABOUTME: Checks that a draft stays local until explicitly queued to its intended target.
// ABOUTME: Covers pane and popup changes, reconnection and queue rejection.
import XCTest
@testable import SpaceportMobile

final class TerminalDraftTests: XCTestCase {
    private let original = InputTarget(connection: 1, bootId: "boot", paneId: "one", popupId: nil)

    func testSendQueuesExactTextOnceAndClearsAcceptedDraft() {
        let content = "  日本語 👨🏽‍💻\n\n\tgit status \r\n"
        var draft = TerminalDraft()
        draft.open(at: DraftDestination(target: original, workspaces: [], tabs: [], panes: []))
        draft.text = content
        var pastes: [String] = []
        let enqueue: (String, InputTarget) -> Bool = { text, target in
            XCTAssertEqual(target, self.original)
            pastes.append(text)
            return true
        }
        XCTAssertTrue(draft.send(to: original, enqueuePaste: enqueue))
        XCTAssertEqual(pastes.map { Array($0.utf8) }, [Array(content.utf8)])
        XCTAssertTrue(draft.text.isEmpty)
        XCTAssertFalse(draft.send(to: original, enqueuePaste: enqueue))
        XCTAssertEqual(pastes.count, 1)
    }

    func testReopeningDraftAfterResumeAdoptsFreshInputEpochForSameTerminal() {
        var draft = TerminalDraft()
        draft.open(at: DraftDestination(target: original, workspaces: [], tabs: [], panes: []))
        draft.text = "unfinished"
        var resumed = original
        resumed.inputEpoch += 1
        XCTAssertFalse(draft.canSend(to: resumed))
        draft.open(at: DraftDestination(target: resumed, workspaces: [], tabs: [], panes: []))
        XCTAssertEqual(draft.text, "unfinished")
        XCTAssertTrue(draft.canSend(to: resumed))
        XCTAssertFalse(draft.canSend(to: original))
    }

    func testReopeningPreservesDestinationUntilExplicitRetarget() {
        let destination = DraftDestination(target: original, workspaces: [], tabs: [], panes: [])
        let targets = [
            InputTarget(connection: 2, bootId: "boot", paneId: "one", popupId: nil),
            InputTarget(connection: 1, bootId: "restart", paneId: "one", popupId: nil),
            InputTarget(connection: 1, bootId: "boot", paneId: "two", popupId: nil),
            InputTarget(connection: 1, bootId: "boot", paneId: "one", popupId: "prompt")
        ]
        for target in targets {
            let current = DraftDestination(target: target, workspaces: [], tabs: [], panes: [])
            var draft = TerminalDraft()
            draft.open(at: destination)
            draft.text = "review"
            draft.open(at: nil)
            draft.open(at: current)
            XCTAssertEqual(draft.destination, destination)
            for stale in [nil, target] {
                XCTAssertFalse(draft.send(to: stale) { _, _ in XCTFail("Stale paste queued"); return true })
            }
            draft.retarget(to: current)
            XCTAssertEqual(draft.text, "review")
            XCTAssertTrue(draft.send(to: target) { text, address in
                XCTAssertEqual(text, "review")
                XCTAssertEqual(address, target)
                return true
            })
        }
    }

    func testQueueRejectionPreservesWhitespaceOnlyDraftForRetry() {
        var draft = TerminalDraft()
        draft.open(at: DraftDestination(target: original, workspaces: [], tabs: [], panes: []))
        draft.text = " \n"
        XCTAssertFalse(draft.send(to: original) { _, _ in false })
        XCTAssertEqual(draft.text, " \n")
        XCTAssertTrue(draft.canSend(to: original))
    }

    func testDestinationNamesWorkspaceTabPaneAndPopupWithDistinctSiblingLabels() {
        let workspace = MobileWorkspace(id: "w", label: "Lab", status: "")
        let tab = MobileTab(id: "t", workspaceId: "w", label: "Build", status: "")
        let first = MobilePane(stateChangeSeq: nil, agentName: nil, id: "one", tabId: "t", label: "Shell", status: "")
        let second = MobilePane(stateChangeSeq: nil, agentName: nil, id: "two", tabId: "t", label: "Shell", status: "")
        let popup = InputTarget(connection: 1, bootId: "boot", paneId: "two", popupId: "help")
        func label(_ target: InputTarget, panes: [MobilePane]) -> String {
            DraftDestination(target: target, workspaces: [workspace], tabs: [tab], panes: panes).label
        }
        XCTAssertEqual(label(original, panes: [first]), "Shell · Lab › Build")
        XCTAssertEqual(label(original, panes: [first, second]), "Shell (one) · Lab › Build")
        XCTAssertEqual(label(popup, panes: [first, second]), "Shell (two) › Popup help · Lab › Build")
    }

    func testUntargetedDraftRequiresExplicitDestination() {
        var draft = TerminalDraft()
        draft.open(at: nil)
        draft.text = "keep editing"
        draft.open(at: DraftDestination(target: original, workspaces: [], tabs: [], panes: []))
        XCTAssertFalse(draft.canSend(to: original))
        XCTAssertEqual(draft.text, "keep editing")
    }
}
