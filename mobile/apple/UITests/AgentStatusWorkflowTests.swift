// ABOUTME: Verifies a real SSH session’s last observed agent counts on the iOS system surface.
// ABOUTME: Uses a seeded disposable session and the installed WidgetKit extension.
import XCTest

extension SessionWorkflowTests {
    @MainActor
    func testLiveActivitySwitchPersistsOnRelaunch() throws {
        let app = XCUIApplication()
        func openSetting() -> XCUIElement {
            app.launch()
            app.buttons["Settings"].tap()
            let toggle = app.switches["settings.liveActivity"]
            reveal(toggle, in: app)
            return toggle
        }
        let toggle = openSetting()
        let original = try XCTUnwrap(toggle.value as? String)
        toggle.coordinate(withNormalizedOffset: CGVector(dx: 0.94, dy: 0.5)).tap()
        app.terminate()
        let saved = openSetting()
        XCTAssertEqual(saved.value as? String, original == "1" ? "0" : "1")
        attachScreen("Live Activity setting", in: app)
        saved.coordinate(withNormalizedOffset: CGVector(dx: 0.94, dy: 0.5)).tap()
        XCTAssertEqual(saved.value as? String, original)
    }

    @MainActor
    func testLastSeenAgentCountsAppearInNotificationCenter() throws {
        let app = XCUIApplication()
        _ = try connect(in: app, sessionVariable: "SPACEPORT_TEST_ACTIVITY_SESSION")
        XCUIDevice.shared.press(.home)
        let system = XCUIApplication(bundleIdentifier: "com.apple.springboard")
        let top = system.coordinate(withNormalizedOffset: CGVector(dx: 0.2, dy: 0.005))
        let bottom = system.coordinate(withNormalizedOffset: CGVector(dx: 0.2, dy: 0.85))
        top.press(forDuration: 0.1, thenDragTo: bottom)
        let allow = system.buttons["Allow"]
        if allow.exists { allow.tap() }
        for text in ["1 waiting", "1 working", "2 idle", "Last seen"] {
            let label = system.descendants(matching: .any).matching(NSPredicate(format: "label CONTAINS[c] %@", text)).firstMatch
            XCTAssertTrue(label.waitForExistence(timeout: 8), "Missing Live Activity label: \(text)")
        }
        attachScreen("Lock Screen agent counts", in: system)
        XCUIDevice.shared.press(.home)
        let compact = system.descendants(matching: .any).matching(NSPredicate(format: "label CONTAINS[c] %@", "1 waiting")).firstMatch
        XCTAssertTrue(compact.waitForExistence(timeout: 5), "The compact Island is missing its waiting count.")
        attachScreen("Compact agent activity", in: system)
        let island = system.coordinate(withNormalizedOffset: CGVector(dx: 0.5, dy: 0.035))
        island.press(forDuration: 1)
        let summary = system.descendants(matching: .any).matching(NSPredicate(format: "label CONTAINS[c] %@", "1 working")).firstMatch
        XCTAssertTrue(summary.waitForExistence(timeout: 5), "The expanded Island is missing its working count.")
        attachScreen("Expanded agent activity", in: system)
        app.activate()
        let navigation = app.buttons["session.navigation"]
        XCTAssertTrue(navigation.waitForExistence(timeout: 8))
        navigation.tap()
        let disconnect = app.buttons["session.disconnect"]
        reveal(disconnect, in: app)
        disconnect.tap()
        XCTAssertTrue(app.buttons["Add Host"].waitForExistence(timeout: 8))
        XCUIDevice.shared.press(.home)
        top.press(forDuration: 0.1, thenDragTo: bottom)
        XCTAssertFalse(summary.exists, "Disconnect should remove the Live Activity.")
        XCUIDevice.shared.press(.home)
    }
}
