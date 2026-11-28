// ABOUTME: Sends a real session screen to the app switcher and returns to it.
// ABOUTME: Uses a loopback login form without credentials or a remote account.
import XCTest

extension SessionWorkflowTests {
    @MainActor
    func testReturningFromPrivacyCoverRestoresSessionForm() {
        let app = XCUIApplication()
        app.launch()
        addHost(in: app, hostname: "127.0.0.1", port: "1", user: "flight", session: "cover-check", password: true)
        let password = app.secureTextFields["SSH password"]
        XCTAssertTrue(password.waitForExistence(timeout: 10))
        let alert = app.alerts["Session"]
        XCTAssertTrue(alert.waitForExistence(timeout: 5))
        XCUIDevice.shared.press(.home)
        XCTAssertTrue(app.wait(for: .runningBackground, timeout: 5))
        app.activate()
        XCTAssertTrue(alert.waitForExistence(timeout: 5))
        alert.buttons["OK"].tap()
        XCTAssertTrue(password.waitForExistence(timeout: 5))
        XCTAssertTrue(password.isHittable)
    }
}
