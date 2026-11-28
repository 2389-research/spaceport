// ABOUTME: Navigates from a live SSH terminal to the phone's agent list with one swipe.
// ABOUTME: Checks the pane and keyboard after the user dismisses the list.
import XCTest

extension SessionWorkflowTests {
    @MainActor
    func testSwipeRightFromTerminalReturnsToAgents() async throws {
        let app = XCUIApplication()
        let (terminal, _) = try connect(in: app, sessionVariable: "SPACEPORT_TEST_ORDER_SESSION")
        let location = app.buttons["session.navigation"]
        location.tap()
        let reviewer = app.buttons.matching(NSPredicate(format: "identifier BEGINSWITH %@ AND label BEGINSWITH %@",
                                                        "goTo.needsYou.", "Reviewer")).firstMatch
        reviewer.tap()
        try await waitForText("Review is ready.", in: terminal)
        app.buttons["terminal.keyboard.toggle"].tap()
        let keyboard = app.keyboards.firstMatch
        XCTAssertTrue(keyboard.waitForExistence(timeout: 5))
        let picker = app.segmentedControls["goTo.browse"]
        terminal.swipeUp()
        terminal.swipeLeft()
        XCTAssertFalse(picker.exists)
        terminal.swipeRight()
        guard picker.waitForExistence(timeout: 3) else { return XCTFail("A right swipe should open Agents.") }
        XCTAssertTrue(picker.buttons["Agents"].isSelected)
        XCTAssertTrue(reviewer.exists)
        app.buttons["goTo.done"].tap()
        XCTAssertTrue(keyboard.waitForExistence(timeout: 5))
        try await waitForText("Reviewer", in: location)
        try await waitForText("Review is ready.", in: terminal)
    }

    @MainActor
    func testSwipeRightExpandsDuoAgentRail() throws {
        let app = XCUIApplication()
        let (terminal, _) = try connect(in: app, sessionVariable: "SPACEPORT_TEST_DUO_SESSION")
        let sidebar = app.buttons["session.sidebar.hide"]
        XCTAssertTrue(sidebar.waitForExistence(timeout: 5))
        sidebar.tap()
        XCTAssertTrue(app.buttons["session.rail.expand"].waitForExistence(timeout: 5))
        terminal.swipeRight()
        XCTAssertTrue(sidebar.waitForExistence(timeout: 3))
        XCTAssertTrue(app.segmentedControls["goTo.browse"].buttons["Agents"].isSelected)
        XCTAssertFalse(app.buttons["goTo.done"].exists)
        XCTAssertTrue(terminal.exists)
    }
}
