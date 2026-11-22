// ABOUTME: Exercises UIKit space-bar cursor callbacks against semantic terminal keys.
// ABOUTME: Confirms that retiring the input target stops the gesture mid-drag.
import XCTest
@testable import SpaceportMobile

@MainActor
final class FloatingCursorTests: XCTestCase {
    func testSpaceBarDragSendsArrowsOnlyToOriginalPane() {
        let pane = InputTarget(connection: 1, bootId: "boot", paneId: "pane", popupId: nil)
        var keys: [String] = []
        let input = TerminalInput.InputView()
        input.attach(to: pane) { event, target in
            XCTAssertEqual(target, pane)
            if case let .key(name, 0) = event { keys.append(name) }
        }
        input.beginFloatingCursor(at: .zero)
        input.updateFloatingCursor(at: CGPoint(x: 22, y: 0))
        XCTAssertEqual(keys, ["Right", "Right"])
        input.invalidate()
        input.updateFloatingCursor(at: CGPoint(x: 75, y: 0))
        input.endFloatingCursor()
        XCTAssertEqual(keys, ["Right", "Right"])
    }
}
