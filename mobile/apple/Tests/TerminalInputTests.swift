// ABOUTME: Exercises the UIKit terminal input method across changes of pane.
// ABOUTME: Checks committed, provisional and cancelled keyboard composition.
import UIKit
import XCTest
@testable import SpaceportMobile

@MainActor
final class TerminalInputTests: XCTestCase {
    func testRetiringAnEditorDropsLateCompositionAndDoesNotRedirectIt() {
        let target = InputTarget(connection: 1, bootId: "one", paneId: "a", popupId: nil)
        let other = InputTarget(connection: 1, bootId: "one", paneId: "b", popupId: nil)
        var submitted: [InputTarget] = []
        let first = TerminalInput.InputView()
        first.attach(to: target) { _, address in submitted.append(address) }
        first.setMarkedText("に", selectedRange: NSRange(location: 1, length: 0))
        XCTAssertTrue(submitted.isEmpty)
        first.invalidate()
        let second = TerminalInput.InputView()
        second.attach(to: other) { _, address in submitted.append(address) }
        first.insertText("日本")
        XCTAssertTrue(submitted.isEmpty)
        second.insertText("a")
        XCTAssertEqual(submitted, [other])
    }

    func testIMECommitReachesOriginalTargetOnce() {
        let target = InputTarget(connection: 2, bootId: "one", paneId: "a", popupId: nil)
        var text: [String] = []
        let input = TerminalInput.InputView()
        input.attach(to: target) { event, _ in
            if case let .text(value) = event { text.append(value) }
        }
        input.setMarkedText("に", selectedRange: NSRange(location: 1, length: 0))
        input.insertText("日本")
        input.unmarkText()
        XCTAssertEqual(text, ["日本"])
    }
}
