// ABOUTME: Runs a Spaceport App Shortcut from Apple's Shortcuts library.
// ABOUTME: Checks warm and cold entry against a real waiting agent on the SSH fixture.
import XCTest

extension SessionWorkflowTests {
    @MainActor
    func testShortcutsOpensNextWaitingAgent() async throws {
        let app = XCUIApplication()
        let (terminal, _) = try connect(in: app, sessionVariable: "SPACEPORT_TEST_SHORTCUT_SESSION")
        let navigation = app.buttons["session.navigation"]
        let shortcuts = XCUIApplication(bundleIdentifier: "com.apple.shortcuts")

        try await focusScout(in: app, navigation: navigation)
        shortcuts.launch()
        runShortcut("Next Waiting Agent", in: shortcuts)
        XCTAssertTrue(app.wait(for: .runningForeground, timeout: 10))
        try await waitForText("Reviewer", in: navigation)
        try await waitForText("Review is ready.", in: terminal)
        attachScreen("Reviewer opened by Shortcuts", in: app)

        try await focusScout(in: app, navigation: navigation)
        shortcuts.activate()
        app.terminate()
        XCTAssertEqual(app.state, .notRunning)
        runShortcut("Next Waiting Agent", in: shortcuts)
        XCTAssertTrue(app.wait(for: .runningForeground, timeout: 10))
        let password = try XCTUnwrap(ProcessInfo.processInfo.environment["SPACEPORT_TEST_SSH_PASSWORD"])
        let field = app.secureTextFields["SSH password"]
        XCTAssertTrue(field.waitForExistence(timeout: 10))
        field.tap()
        field.typeText(password)
        app.buttons["Sign In"].tap()
        try await waitForText("Reviewer", in: navigation)
        try await waitForText("Review is ready.", in: terminal)
        attachScreen("Reviewer after a cold launch from Shortcuts", in: app)
    }

    @MainActor
    func testShortcutsPreservesDraftAndOpensAgentsAfterApproval() async throws {
        let app = XCUIApplication()
        let (terminal, _) = try connect(in: app, sessionVariable: "SPACEPORT_TEST_COMPOSE_SHORTCUT_SESSION")
        let navigation = app.buttons["session.navigation"]
        navigation.tap()
        let reviewer = app.buttons.matching(NSPredicate(format: "identifier BEGINSWITH %@ AND label BEGINSWITH %@",
                                                        "goTo.needsYou.", "Reviewer")).firstMatch
        reviewer.tap()
        try await waitForText("Review is ready.", in: terminal)

        let reply = "approve-" + UUID().uuidString.prefix(8)
        let editor = app.textViews["draft.editor"]
        app.buttons["session.compose"].tap()
        editor.tap()
        editor.typeText(reply)
        app.buttons["draft.done"].tap()
        let shortcuts = XCUIApplication(bundleIdentifier: "com.apple.shortcuts")
        shortcuts.launch()
        runShortcut("Compose Prompt", in: shortcuts)
        XCTAssertTrue(app.wait(for: .runningForeground, timeout: 10))
        XCTAssertTrue(editor.waitForExistence(timeout: 10))
        XCTAssertEqual(editor.value as? String, reply)
        attachScreen("Unsent draft opened by Compose Prompt", in: app)
        app.buttons["draft.done"].tap()
        try await waitForText("Review is ready.", in: terminal)
        XCTAssertFalse(try XCTUnwrap(terminal.value as? String).contains(reply))
        try await waitForText("Needs you", in: navigation)

        app.buttons["session.compose"].tap()
        let pressReturn = app.switches["draft.pressReturn"]
        if pressReturn.value as? String != "1" {
            pressReturn.coordinate(withNormalizedOffset: CGVector(dx: 0.94, dy: 0.5)).tap()
        }
        XCTAssertEqual(app.buttons["draft.send"].label, "Send")
        app.buttons["draft.send"].tap()
        try await waitForText("accepted-" + reply, in: terminal)
        try await waitForText("Idle", in: navigation)

        shortcuts.activate()
        runShortcut("Next Waiting Agent", in: shortcuts)
        XCTAssertTrue(app.wait(for: .runningForeground, timeout: 10))
        let agents = app.segmentedControls["goTo.browse"].buttons["Agents"]
        XCTAssertTrue(agents.waitForExistence(timeout: 10))
        XCTAssertTrue(agents.isSelected)
        XCTAssertFalse(reviewer.exists)
        attachScreen("Agents when nobody is waiting", in: app)
    }

    @MainActor
    private func focusScout(in app: XCUIApplication, navigation: XCUIElement) async throws {
        navigation.tap()
        let scout = app.buttons.matching(NSPredicate(format: "identifier BEGINSWITH %@ AND label BEGINSWITH %@",
                                                     "goTo.agent.", "Scout")).firstMatch
        XCTAssertTrue(scout.waitForExistence(timeout: 5))
        scout.tap()
        try await waitForText("Scout", in: navigation)
    }

    @MainActor
    private func runShortcut(_ title: String, in shortcuts: XCUIApplication) {
        let action = shortcuts.descendants(matching: .any)
            .matching(NSPredicate(format: "label ==[c] %@", title)).firstMatch
        if !action.exists || !action.isHittable {
            let search = shortcuts.searchFields.firstMatch
            XCTAssertTrue(search.waitForExistence(timeout: 8))
            search.tap()
            let clear = search.buttons["Clear text"]
            if clear.exists { clear.tap() }
            search.typeText(title)
        }
        guard action.waitForExistence(timeout: 10) else {
            return XCTFail("Shortcuts did not list \(title).\n\(shortcuts.debugDescription)")
        }
        attachScreen("\(title) in Shortcuts", in: shortcuts)
        action.tap()
    }
}
