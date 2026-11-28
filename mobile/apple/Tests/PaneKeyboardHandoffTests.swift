// ABOUTME: Exercises terminal keyboard ownership across a pane switch in a real UIKit window.
// ABOUTME: Verifies that unconfirmed input has no destination and a closed keyboard stays closed.
import UIKit
import XCTest
@testable import SpaceportMobile

@MainActor
final class PaneKeyboardHandoffTests: XCTestCase {
    private func window() throws -> UIWindow {
        let scene = try XCTUnwrap(UIApplication.shared.connectedScenes.compactMap { $0 as? UIWindowScene }.first)
        let window = UIWindow(windowScene: scene)
        window.rootViewController = UIViewController()
        window.makeKeyAndVisible()
        return window
    }

    func testConfirmedPaneTakesTheKeyboardDirectlyFromHandoff() throws {
        let window = try window()
        defer { window.isHidden = true }
        let source = PhoneGhosttyView(frame: window.bounds)
        let destination = PhoneGhosttyView(frame: window.bounds)
        window.addSubview(source)
        window.addSubview(destination)
        source.wantsKeyboard = true
        XCTAssertTrue(source.becomeFirstResponder())
        let handoff = PaneKeyboardHandoff()
        XCTAssertTrue(handoff.hold(from: source))
        XCTAssertTrue(handoff.isFirstResponder)
        XCTAssertFalse(source.isFirstResponder)
        XCTAssertFalse(source.wantsKeyboard)
        source.removeFromSuperview()
        handoff.insertText("discard during switch")
        handoff.deleteBackward()
        XCTAssertFalse(handoff.hasText)
        XCTAssertTrue(destination.becomeFirstResponder())
        XCTAssertFalse(handoff.isFirstResponder)
        handoff.cancel()
        XCTAssertTrue(destination.isFirstResponder)
    }

    func testSystemDismissalDuringSwitchIsRespectedOnArrival() throws {
        let window = try window()
        defer { window.isHidden = true }
        let source = PhoneGhosttyView(frame: window.bounds)
        window.addSubview(source)
        source.wantsKeyboard = true
        XCTAssertTrue(source.becomeFirstResponder())
        let handoff = PaneKeyboardHandoff()
        XCTAssertTrue(handoff.hold(from: source))
        let intent = GoToSheet.Jump.Keyboard.held
        XCTAssertTrue(intent.showOnArrival(requested: true, handoff: handoff, hasSurface: true))
        XCTAssertFalse(intent.showOnArrival(requested: false, handoff: handoff, hasSurface: true))
        XCTAssertFalse(intent.showOnArrival(requested: true, handoff: handoff, hasSurface: false))

        XCTAssertTrue(window.endEditing(true))
        XCTAssertFalse(intent.showOnArrival(requested: true, handoff: handoff, hasSurface: true))
    }

    func testClosedKeyboardAndCancelledSwitchLeaveNoResponder() throws {
        let window = try window()
        defer { window.isHidden = true }
        let source = PhoneGhosttyView(frame: window.bounds)
        window.addSubview(source)
        let handoff = PaneKeyboardHandoff()
        XCTAssertFalse(handoff.hold(from: source))
        XCTAssertNil(handoff.superview)
        source.wantsKeyboard = true
        XCTAssertTrue(source.becomeFirstResponder())
        XCTAssertTrue(handoff.hold(from: source))
        handoff.cancel()
        XCTAssertFalse(handoff.isFirstResponder)
        XCTAssertNil(handoff.superview)
    }
}
