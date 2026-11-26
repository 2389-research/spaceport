// ABOUTME: Adds the Spaceport widget through the iPhone Home Screen gallery.
// ABOUTME: Verifies its saved counts and opens the waiting pane after an SSH sign-in.
import XCTest

extension SessionWorkflowTests {
    @MainActor
    func testHomeWidgetOpensWaitingAgentAfterSignIn() async throws {
        continueAfterFailure = false
        let app = XCUIApplication()
        let (terminal, _) = try connect(in: app, sessionVariable: "SPACEPORT_TEST_HOME_WIDGET_SESSION")
        let navigation = app.buttons["session.navigation"]
        navigation.tap()
        let scout = app.buttons.matching(NSPredicate(format: "identifier BEGINSWITH %@ AND label BEGINSWITH %@",
                                                     "goTo.agent.", "Scout")).firstMatch
        XCTAssertTrue(scout.waitForExistence(timeout: 5))
        scout.tap()
        try await waitForText("Scout", in: navigation)
        XCUIDevice.shared.press(.home)

        let system = XCUIApplication(bundleIdentifier: "com.apple.springboard")
        let widget = system.descendants(matching: .any)
            .matching(NSPredicate(format: "label == %@ AND value BEGINSWITH %@", "Spaceport", "Widget")).firstMatch
        if !widget.exists { addAgentWidget(in: system) }
        XCTAssertTrue(widget.waitForExistence(timeout: 8))
        for count in ["1 waiting", "1 working", "2 idle"] {
            let entry = widget.descendants(matching: .any)
                .matching(NSPredicate(format: "label CONTAINS[c] %@", count)).firstMatch
            XCTAssertTrue(entry.waitForExistence(timeout: 8), "Missing widget count: \(count)")
        }
        let time = widget.descendants(matching: .any)
            .matching(NSPredicate(format: "label MATCHES[c] %@", ".*Last checked.*[0-9]+:[0-9]{2}.*")).firstMatch
        XCTAssertTrue(time.waitForExistence(timeout: 8))
        attachScreen("Home Screen agent widget", in: system)
        app.terminate()
        widget.tap()
        XCTAssertTrue(app.wait(for: .runningForeground, timeout: 10))
        let password = try XCTUnwrap(ProcessInfo.processInfo.environment["SPACEPORT_TEST_SSH_PASSWORD"])
        let field = app.secureTextFields["SSH password"]
        XCTAssertTrue(field.waitForExistence(timeout: 10))
        field.tap()
        field.typeText(password)
        app.buttons["Sign In"].tap()
        try await waitForText("Reviewer", in: navigation)
        try await waitForText("Review is ready.", in: terminal)
        attachScreen("Reviewer opened by Home Screen widget", in: app)
    }

    @MainActor
    private func addAgentWidget(in system: XCUIApplication) {
        let spaceAboveDock = system.coordinate(withNormalizedOffset: CGVector(dx: 0.15, dy: 0.83))
        spaceAboveDock.press(forDuration: 1.2)
        system.buttons["Edit"].tap()
        system.buttons["Add Widget"].tap()
        let search = system.searchFields["Search Widgets"]
        XCTAssertTrue(search.waitForExistence(timeout: 8))
        search.tap()
        search.typeText("Spaceport")
        let result = system.cells["Spaceport"]
        XCTAssertTrue(result.waitForExistence(timeout: 8), "Spaceport is missing from the Home Screen widget gallery.")
        result.tap()
        XCTAssertTrue(system.staticTexts["Agent Status"].waitForExistence(timeout: 5))
        system.buttons.matching(NSPredicate(format: "label ENDSWITH %@", "Add Widget")).firstMatch.tap()
        system.buttons["Done"].tap()
    }
}
