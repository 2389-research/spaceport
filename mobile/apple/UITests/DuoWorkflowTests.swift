// ABOUTME: Verifies that folding keeps Ghostty input bound to the selected remote agent.
// ABOUTME: Runs against a seeded SSH fixture and the iPhone Duo’s actual inner and outer displays.
import XCTest

extension SessionWorkflowTests {
    @MainActor
    func testDuoKeepsReviewerInputWhenFolding() async throws {
        try XCTSkipUnless(ProcessInfo.processInfo.environment["SPACEPORT_TEST_DUO_SESSION"] != nil,
                          "Open the Duo simulator and select a disposable seed-agent-order session.")
        let app = XCUIApplication()
        app.launchArguments += ["-useGhosttyTerminal", "YES"]
        let (surface, _) = try connect(in: app, sessionVariable: "SPACEPORT_TEST_DUO_SESSION",
                                       terminalIdentifier: "terminal.ghostty")
        let sidebar = app.buttons["session.sidebar.hide"]
        guard sidebar.waitForExistence(timeout: 8) else {
            XCTFail("The open display must show the agent sidebar beside the terminal.")
            return
        }
        let reviewer = app.buttons.matching(NSPredicate(format: "identifier BEGINSWITH %@", "goTo.needsYou.")).firstMatch
        XCTAssertTrue(reviewer.waitForExistence(timeout: 5))
        reviewer.tap()
        try await waitForText("Review is ready.", in: surface)
        XCTAssertTrue(sidebar.exists)
        attachScreen("Duo inner display with agent sidebar", in: app)
        app.buttons["terminal.keyboard.toggle"].tap()
        let keyboard = app.keyboards.firstMatch
        XCTAssertTrue(keyboard.waitForExistence(timeout: 5))
        let prefix = "duo-" + String(UUID().uuidString.prefix(6)).lowercased()
        app.typeText(prefix)
        try await waitForText(prefix, in: surface)
        sidebar.tap()
        XCTAssertTrue(sidebar.waitForNonExistence(timeout: 5))
        XCTAssertTrue(keyboard.exists)
        app.buttons["session.navigation"].tap()
        XCTAssertTrue(sidebar.waitForExistence(timeout: 5))
        XCTAssertTrue(keyboard.exists)

        XCTContext.runActivity(named: "Close iPhone Duo in Device Hub") { _ in
            XCTAssertTrue(sidebar.waitForNonExistence(timeout: 45))
        }
        XCTAssertTrue(keyboard.exists)
        XCTAssertTrue((surface.value as? String ?? "").contains(prefix))
        XCTAssertFalse((surface.value as? String ?? "").contains("accepted-" + prefix))
        attachScreen("Duo outer display with pending input", in: app)
        app.typeText("-folded")

        XCTContext.runActivity(named: "Open iPhone Duo in Device Hub") { _ in
            XCTAssertTrue(sidebar.waitForExistence(timeout: 45))
        }
        XCTAssertTrue(keyboard.exists)
        app.typeText("-open\n")
        try await waitForText("accepted-" + prefix + "-folded-open", in: surface)

        app.buttons["session.compose"].tap()
        let editor = app.textViews["draft.editor"]
        let destination = app.buttons["draft.destination"]
        editor.tap()
        let draft = "Unsent " + prefix
        editor.typeText(draft)
        let recipient = destination.label
        let innerWidth = app.frame.width
        for open in [false, true] {
            let activity = open ? "Reopen iPhone Duo with draft" : "Close iPhone Duo with draft"
            XCTContext.runActivity(named: activity) { _ in
                let transition = XCTNSPredicateExpectation(predicate: NSPredicate { _, _ in
                    (app.frame.width == innerWidth) == open
                }, object: app)
                XCTAssertEqual(XCTWaiter.wait(for: [transition], timeout: 45), .completed)
            }
            XCTAssertEqual(editor.value as? String, draft)
            XCTAssertEqual(destination.label, recipient)
        }
        app.buttons["draft.done"].tap()
        XCTAssertTrue(sidebar.waitForExistence(timeout: 5))
        XCTAssertFalse((surface.value as? String ?? "").contains(draft))
    }
}
