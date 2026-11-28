// ABOUTME: Opens a disposable remote web page in Safari through the debug SSH preview.
// ABOUTME: Checks a second browser request before returning to stop the listener.
import XCTest

extension SessionWorkflowTests {
    @MainActor
    func testLoopbackServiceInSafari() throws {
        let environment = ProcessInfo.processInfo.environment
        guard let port = environment["SPACEPORT_TEST_SERVICE_PORT"] else {
            throw XCTSkip("Choose a loopback HTTP trial fixture.")
        }
        let app = XCUIApplication()
        _ = try connect(in: app, sessionVariable: "SPACEPORT_TEST_SERVICE_SESSION")
        app.buttons["session.navigation"].tap()
        app.buttons["Preview loopback service"].tap()
        let prompt = app.alerts["Preview loopback service"]
        prompt.textFields["Remote port"].typeText(port)
        prompt.buttons["Open"].tap()
        let safari = XCUIApplication(bundleIdentifier: "com.apple.mobilesafari")
        XCTAssertTrue(safari.wait(for: .runningForeground, timeout: 15))
        let link = safari.webViews.links["Check again"]
        XCTAssertTrue(link.waitForExistence(timeout: 15))
        link.tap()
        XCTAssertTrue(safari.webViews.staticTexts["Round trip confirmed"].waitForExistence(timeout: 8))
        app.activate()
        let navigation = app.buttons["session.navigation"]
        let ready = XCTNSPredicateExpectation(predicate: NSPredicate { _, _ in
            app.state == .runningForeground && navigation.isHittable && navigation.isEnabled
        }, object: nil)
        XCTAssertEqual(XCTWaiter.wait(for: [ready], timeout: 15), .completed)
        navigation.tap()
        let stop = app.buttons["Stop service preview"]
        XCTAssertTrue(stop.waitForExistence(timeout: 15))
        stop.tap()
        XCTAssertTrue(app.buttons["Preview loopback service"].waitForExistence(timeout: 5))
    }
}
