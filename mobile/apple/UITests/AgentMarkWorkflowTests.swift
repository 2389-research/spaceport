// ABOUTME: Marks an agent from a real session through Home's swipe and menu actions.
// ABOUTME: Reopens the app to check that pin and snooze persist on the phone.
import XCTest

extension SessionWorkflowTests {
    @MainActor
    func testPinSnoozeAndWakeOnHome() throws {
        let app = XCUIApplication()
        _ = try connect(in: app, sessionVariable: "SPACEPORT_TEST_MARKS_SESSION")
        let session = try XCTUnwrap(ProcessInfo.processInfo.environment["SPACEPORT_TEST_MARKS_SESSION"])
        app.buttons["session.home"].tap()
        let scout = app.buttons.matching(NSPredicate(format: "identifier BEGINSWITH %@ AND label BEGINSWITH %@ AND label CONTAINS %@",
                                                    "home.agent.", "Scout,", session)).firstMatch
        reveal(scout, in: app)
        scout.swipeRight()
        app.buttons["Pin"].tap()
        reveal(scout, in: app, towardTop: true)
        XCTAssertTrue((scout.value as? String)?.contains("Pinned") == true)
        scout.swipeLeft()
        app.buttons["Snooze 1h"].tap()
        reveal(scout, in: app)
        XCTAssertTrue((scout.value as? String)?.contains("Snoozed until") == true)
        attachScreen("Home snoozed agent", in: app)
        app.terminate()
        app.launch()
        reveal(scout, in: app)
        XCTAssertTrue((scout.value as? String)?.contains("Snoozed until") == true)
        scout.press(forDuration: 0.8)
        app.buttons["Wake now"].tap()
        reveal(scout, in: app, towardTop: true)
        XCTAssertTrue((scout.value as? String)?.contains("Pinned") == true)
        scout.press(forDuration: 0.8)
        app.buttons["Unpin"].tap()
        reveal(scout, in: app)
        XCTAssertFalse((scout.value as? String ?? "").contains("Pinned"))
    }
}
