// ABOUTME: Exercises native Go To pane actions and the terminal focus trail on iPhone.
// ABOUTME: Verifies the keyboard follows the confirmed pane on Back and Forward.
import XCTest

extension SessionWorkflowTests {
    @MainActor
    func testSplitPaneAndWalkHistoryWithKeyboard() async throws {
        let app = XCUIApplication()
        let (terminal, _) = try connect(in: app, sessionVariable: "SPACEPORT_TEST_PANE_ACTIONS_SESSION")
        let keyboard = app.keyboards.firstMatch
        let toggle = app.buttons["terminal.keyboard.toggle"]
        toggle.tap()
        XCTAssertTrue(keyboard.waitForExistence(timeout: 5))
        let marker = "pane-origin-" + UUID().uuidString.prefix(6).lowercased()
        app.typeText("echo " + marker + "\n")
        try await waitForText(marker, in: terminal)

        app.buttons["session.navigation"].tap()
        app.segmentedControls["goTo.browse"].buttons["Workspaces"].tap()
        let workspace = app.buttons.matching(NSPredicate(format: "identifier BEGINSWITH %@", "goTo.workspace.")).firstMatch
        XCTAssertTrue(workspace.waitForExistence(timeout: 5))
        workspace.tap()
        let origin = app.buttons.matching(NSPredicate(format: "identifier BEGINSWITH %@", "goTo.pane.")).firstMatch
        XCTAssertTrue(origin.waitForExistence(timeout: 5))
        origin.press(forDuration: 0.8)
        let split = app.buttons["Split right"]
        XCTAssertTrue(split.waitForExistence(timeout: 5))
        XCTAssertTrue(app.buttons["Rename pane"].exists)
        XCTAssertTrue(app.buttons["Zoom pane"].exists)
        XCTAssertTrue(app.buttons["Split down"].exists)
        split.tap()
        app.buttons["goTo.done"].tap()

        let back = app.buttons["Previous pane"]
        XCTAssertTrue(back.waitForExistence(timeout: 8))
        if !keyboard.exists { toggle.tap() }
        XCTAssertTrue(keyboard.waitForExistence(timeout: 5))
        back.tap()
        try await waitForText(marker, in: terminal)
        XCTAssertTrue(keyboard.waitForExistence(timeout: 5))
        let forward = app.buttons["Next pane in history"]
        XCTAssertTrue(forward.waitForExistence(timeout: 5))
        forward.tap()
        let other = expectation(for: NSPredicate(format: "value != nil AND NOT (value CONTAINS %@)", marker), evaluatedWith: terminal)
        await fulfillment(of: [other], timeout: 10)
        XCTAssertTrue(keyboard.waitForExistence(timeout: 5))
    }
}
