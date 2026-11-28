// ABOUTME: Exercises the workspace picker beside a live Ghostty terminal on Duo.
// ABOUTME: Confirms that browsing leaves Herdr on the origin until an explicit pane tap.
import XCTest

extension SessionWorkflowTests {
    @MainActor
    func testSearchAnotherWorkspaceAndReturnToTyping() async throws {
        let app = XCUIApplication()
        let (terminal, _) = try connect(in: app, sessionVariable: "SPACEPORT_TEST_WORKSPACE_PHONE_SESSION")
        let navigation = app.buttons["session.navigation"]
        let origin = try XCTUnwrap(navigation.value as? String)
        XCTAssertTrue(origin.contains("Mission"))
        app.buttons["terminal.keyboard.toggle"].tap()
        XCTAssertTrue(app.keyboards.firstMatch.waitForExistence(timeout: 5))
        navigation.tap()
        app.segmentedControls["goTo.browse"].buttons["Workspaces"].tap()
        let search = app.searchFields.firstMatch
        search.tap()
        search.typeText("fli")
        let flight = app.buttons.matching(NSPredicate(format: "identifier BEGINSWITH %@ AND label == %@", "goTo.workspace.", "Flight")).firstMatch
        flight.tap()
        search.tap()
        search.typeText("logs")
        let panes = app.buttons.matching(NSPredicate(format: "identifier BEGINSWITH %@", "goTo.pane."))
        XCTAssertEqual(panes.count, 1)
        XCTAssertTrue(panes.firstMatch.label.localizedCaseInsensitiveContains("logs"))
        let cancel = app.navigationBars.buttons.matching(NSPredicate(format: "label IN %@", ["Close", "Cancel"])).firstMatch
        cancel.tap()
        attachScreen("iPhone browsing Flight before returning to Mission", in: app)
        app.buttons["goTo.done"].tap()
        XCTAssertEqual(navigation.value as? String, origin)
        XCTAssertTrue(app.keyboards.firstMatch.waitForExistence(timeout: 5))
        let nonce = UUID().uuidString.prefix(6).lowercased()
        app.typeText("printf '%s%s\\n' origin- " + nonce + "\n")
        try await waitForText("origin-" + nonce, in: terminal)
    }

    @MainActor
    func testBrowseWorkspaceBesideLiveTerminal() async throws {
        let app = XCUIApplication()
        let (terminal, _) = try connect(in: app, sessionVariable: "SPACEPORT_TEST_WORKSPACE_DUO_SESSION")
        let navigation = app.buttons["session.navigation"]
        let origin = try XCTUnwrap(navigation.value as? String)
        XCTAssertTrue(origin.contains("Mission"))
        let switcher = app.segmentedControls["goTo.browse"]
        XCTAssertTrue(switcher.waitForExistence(timeout: 5))
        switcher.buttons["Workspaces"].tap()
        let flight = app.buttons.matching(NSPredicate(format: "identifier BEGINSWITH %@ AND label == %@", "goTo.workspace.", "Flight")).firstMatch
        flight.tap()
        XCTAssertTrue(app.buttons["goTo.workspace.back"].waitForExistence(timeout: 5))
        XCTAssertEqual(navigation.value as? String, origin)
        XCTAssertTrue(terminal.isHittable)
        attachScreen("Duo browsing Flight beside Mission", in: app)
        navigation.tap()
        XCTAssertTrue(switcher.buttons["Agents"].isSelected)
        XCTAssertEqual(navigation.value as? String, origin)
        switcher.buttons["Workspaces"].tap()
        flight.tap()
        let logs = app.buttons.matching(NSPredicate(format: "identifier BEGINSWITH %@ AND label CONTAINS[c] %@", "goTo.pane.", "Logs")).firstMatch
        XCTAssertTrue(logs.waitForExistence(timeout: 5))
        logs.tap()
        try await waitForText("Log stream connected.", in: terminal)
        try await waitForText("Flight", in: navigation)
    }
}
