// ABOUTME: Exercises the connected session actions on the Duo side bar and ordinary iPhone header.
// ABOUTME: Checks native control positions, keyboard input and the remote shell created by a tap.
import XCTest

extension SessionWorkflowTests {
    @MainActor
    func testDuoSideBarRunsConnectedSessionActions() async throws {
        continueAfterFailure = false
        let app = XCUIApplication()
        let (terminal, _) = try connect(in: app, sessionVariable: "SPACEPORT_TEST_GUTTER_SESSION")
        let title = app.buttons["session.navigation"]
        XCTAssertTrue(app.buttons["session.sidebar.hide"].waitForExistence(timeout: 5), "Unfold the Duo.")
        let home = app.buttons["session.header.home"]
        let create = app.buttons["session.header.newTerminal"]
        let find = app.buttons["session.header.find"]
        let compose = app.buttons["session.compose"]
        attachScreen("Duo connected session actions", in: app)
        for button in [home, create, find, compose] {
            XCTAssertTrue(button.isHittable)
            XCTAssertGreaterThanOrEqual(button.frame.minX, terminal.frame.maxX - 1,
                                        "\(button.label) should sit in the system side bar beyond the terminal.")
            XCTAssertGreaterThan(button.frame.minY, title.frame.maxY)
            XCTAssertEqual(button.frame.midX, home.frame.midX, accuracy: 2)
        }
        app.buttons["session.sidebar.hide"].tap()
        title.tap()
        XCTAssertTrue(app.buttons["session.sidebar.hide"].waitForExistence(timeout: 5))
        find.tap()
        XCTAssertTrue(app.textFields["history.query"].waitForExistence(timeout: 5))
        app.buttons["history.done"].tap()
        compose.tap()
        XCTAssertTrue(app.textViews["draft.editor"].waitForExistence(timeout: 5))
        XCTAssertFalse(home.exists)
        app.buttons["draft.done"].tap()
        let originalTab = try XCTUnwrap(title.value as? String)
        create.tap()
        let changedTab = expectation(for: NSPredicate(format: "value != %@", originalTab), evaluatedWith: title)
        await fulfillment(of: [changedTab], timeout: 8)
        app.buttons["terminal.keyboard.toggle"].tap()
        XCTAssertTrue(app.keyboards.firstMatch.waitForExistence(timeout: 5))
        let marker = UUID().uuidString.prefix(8)
        app.typeText("printf '%s%s\\n' gutter- " + marker + "\n")
        try await waitForText("gutter-" + marker, in: terminal)
        home.tap()
        XCTAssertTrue(app.buttons["Add Host"].waitForExistence(timeout: 5))
        XCTAssertFalse(terminal.exists)
    }

    @MainActor
    func testPhoneKeepsSessionActionsInTheHeader() async throws {
        continueAfterFailure = false
        let app = XCUIApplication()
        let (terminal, _) = try connect(in: app, sessionVariable: "SPACEPORT_TEST_PHONE_HEADER_SESSION")
        let title = app.buttons["session.navigation"]
        let compose = app.buttons["session.compose"]
        let actions = [app.buttons["session.header.home"], app.buttons["session.header.newTerminal"], compose]
        for button in actions {
            XCTAssertTrue(button.isHittable)
            XCTAssertGreaterThanOrEqual(button.frame.width, 44)
            XCTAssertGreaterThanOrEqual(button.frame.height, 44)
            XCTAssertEqual(button.frame.midY, title.frame.midY, accuracy: 2)
            XCTAssertLessThanOrEqual(button.frame.maxX, terminal.frame.maxX + 1)
        }
        XCTAssertFalse(app.buttons["session.header.find"].exists)
        attachScreen("iPhone connected session header", in: app)
        title.tap()
        app.buttons["session.find"].tap()
        XCTAssertTrue(app.textFields["history.query"].waitForExistence(timeout: 5))
        app.buttons["history.done"].tap()
        compose.tap()
        XCTAssertTrue(app.textViews["draft.editor"].waitForExistence(timeout: 5))
        app.buttons["draft.done"].tap()
        app.buttons["terminal.keyboard.toggle"].tap()
        XCTAssertTrue(app.keyboards.firstMatch.waitForExistence(timeout: 5))
        XCTAssertTrue(app.buttons["terminal.key.ctrl"].isHittable)
    }
}
