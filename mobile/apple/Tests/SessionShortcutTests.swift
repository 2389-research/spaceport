// ABOUTME: Checks shortcut ownership across session presentations and repeated deliveries.
// ABOUTME: Exercises dictated drafts with existing text, photos and a different focused pane.
import UIKit
import XCTest
@testable import SpaceportMobile

@MainActor
final class SessionShortcutTests: XCTestCase {
    func testDictationKeepsExistingPhotoAndTextBoundToTheirOriginalPane() throws {
        let original = InputTarget(connection: 1, bootId: "boot", paneId: "one", popupId: nil)
        var other = original
        other.paneId = "two"
        let destination = DraftDestination(target: original, workspaces: [], tabs: [], panes: [])
        var draft = TerminalDraft()
        draft.open(at: destination)
        draft.text = "unsent"
        let data = UIGraphicsImageRenderer(size: .init(width: 2, height: 2)).pngData { _ in }
        draft.image = try DraftImage(data: data)
        let image = draft.image?.id
        draft.compose(" dictated\ntext ", at: DraftDestination(target: other, workspaces: [], tabs: [], panes: []))
        XCTAssertEqual(draft.text, "unsent\n dictated\ntext ")
        XCTAssertEqual(draft.image?.id, image)
        XCTAssertEqual(draft.destination, destination)
        XCTAssertFalse(draft.canSend(to: other))
    }

    func testDictationPreservesWhitespaceAndAnEmptyRequestOnlyOpensTheDraft() {
        let target = InputTarget(connection: 1, bootId: "boot", paneId: "one", popupId: nil)
        let destination = DraftDestination(target: target, workspaces: [], tabs: [], panes: [])
        var draft = TerminalDraft()
        draft.compose("  日本語\n", at: destination)
        draft.compose("", at: destination)
        XCTAssertEqual(draft.text, "  日本語\n")
        XCTAssertTrue(draft.canSend(to: target))
    }

    func testObsoletePresentationCannotQueueOrReceiveAShortcut() throws {
        let profile = try HostProfile.make(hostname: "test.invalid", port: "22", username: "pilot", session: "work", login: .password)
        let session = MobileConnection(profile: profile)
        let old = UUID(), current = UUID()
        session.present(old, agent: nil, alreadyOpen: true)
        session.requestShortcut(.compose("private"), presentation: old)
        let first = try XCTUnwrap(session.pendingShortcut)
        session.requestShortcut(.compose("second"), presentation: old)
        XCTAssertEqual(session.pendingShortcut?.id, first.id)
        guard case .compose("private") = session.pendingShortcut?.action else {
            return XCTFail("Another delivery replaced the unsent prompt")
        }
        XCTAssertNil(session.takeShortcut(from: old))
        session.present(current, agent: nil, alreadyOpen: true)
        session.requestShortcut(.nextWaiting, presentation: old)
        XCTAssertNil(session.pendingShortcut)
        XCTAssertNil(session.takeShortcut(from: current))
        XCTAssertTrue(session.draft.isEmpty)
    }
}
