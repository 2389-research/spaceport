// ABOUTME: Exercises searchable Go To and read-only previews against a disposable Herdr host.
// ABOUTME: Checks the local Ghostty screen before and after visiting another agent.
import XCTest

extension SessionWorkflowTests {
    @MainActor
    func testSearchAndPeekWithoutMovingTerminalFocus() async throws {
        let app = XCUIApplication()
        let (surface, _) = try connect(in: app, sessionVariable: "SPACEPORT_TEST_PICKER_SESSION")
        let navigation = app.buttons["session.navigation"]
        let origin = try XCTUnwrap(navigation.value as? String)
        navigation.tap()
        let search = app.searchFields.firstMatch
        XCTAssertTrue(search.waitForExistence(timeout: 5))
        search.tap()
        search.typeText("flight launch")
        let reviewer = app.buttons.matching(NSPredicate(format: "identifier BEGINSWITH %@", "goTo.needsYou.")).firstMatch
        XCTAssertTrue(reviewer.waitForExistence(timeout: 5))
        XCTAssertTrue(reviewer.label.contains("Reviewer"))
        XCTAssertFalse(app.buttons.matching(NSPredicate(format: "identifier BEGINSWITH %@", "goTo.agent.")).firstMatch.exists)
        reviewer.press(forDuration: 0.8)
        app.buttons["Peek"].tap()
        XCTAssertTrue(app.staticTexts["No screen preview yet."].waitForExistence(timeout: 5))
        app.navigationBars["Pane preview"].buttons["Done"].tap()
        let dismissSearch = app.navigationBars["Go To"].buttons.matching(
            NSPredicate(format: "label IN %@", ["Close", "Cancel"])
        ).firstMatch
        XCTAssertTrue(dismissSearch.waitForExistence(timeout: 5))
        dismissSearch.tap()
        app.buttons["goTo.done"].tap()
        XCTAssertEqual(navigation.value as? String, origin)

        navigation.tap()
        reviewer.tap()
        try await waitForText("Review is ready.", in: surface)
        navigation.tap()
        let scout = app.buttons.matching(NSPredicate(format: "identifier BEGINSWITH %@ AND label BEGINSWITH %@",
                                                   "goTo.agent.", "Scout")).firstMatch
        scout.tap()
        try await waitForText("Scout", in: navigation)
        navigation.tap()
        reviewer.press(forDuration: 0.8)
        app.buttons["Peek"].tap()
        XCTAssertTrue(app.staticTexts.matching(NSPredicate(format: "label CONTAINS %@", "Review is ready.")).firstMatch.waitForExistence(timeout: 5))
        XCTAssertTrue(app.staticTexts.matching(NSPredicate(format: "label BEGINSWITH %@", "Last seen")).firstMatch.exists)
        attachScreen("Cached pane preview", in: app)
        app.navigationBars["Pane preview"].buttons["Done"].tap()
        app.buttons["goTo.done"].tap()
        XCTAssertTrue((navigation.value as? String ?? "").contains("Scout"))
    }
}
