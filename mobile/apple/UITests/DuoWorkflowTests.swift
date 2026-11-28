// ABOUTME: Verifies that folding keeps Ghostty input bound to the selected remote agent.
// ABOUTME: Runs against a seeded SSH fixture and the iPhone Duo’s actual inner and outer displays.
import XCTest

extension SessionWorkflowTests {
    @MainActor
    func testRailSwitchKeepsKeyboardAndRoutesInputToEachAgent() async throws {
        let app = XCUIApplication()
        let (terminal, _) = try connect(in: app, sessionVariable: "SPACEPORT_TEST_DUO_SESSION")
        let sidebar = app.buttons["session.sidebar.hide"]
        let rail = app.buttons["session.rail.expand"]
        let location = app.buttons["session.navigation"]
        let toggle = app.buttons["terminal.keyboard.toggle"]
        let keyboard = app.keyboards.firstMatch
        let agents = app.buttons.matching(NSPredicate(format: "identifier BEGINSWITH %@", "session.rail.agent."))
        func agent(_ name: String) -> XCUIElement {
            agents.matching(NSPredicate(format: "label BEGINSWITH %@", name + ",")).firstMatch
        }
        XCTAssertTrue(sidebar.waitForExistence(timeout: 5), "Run on an unfolded Duo.")
        toggle.tap()
        XCTAssertTrue(keyboard.waitForExistence(timeout: 5))
        let nonce = UUID().uuidString.prefix(6).lowercased()
        let scoutText = "scout-" + nonce
        let reviewText = "review-" + nonce

        for (name, text) in [("Scout", scoutText), ("Reviewer", reviewText)] {
            XCTContext.runActivity(named: "Switch to \(name) with keyboard open") { _ in
                agent(name).tap()
                XCTAssertTrue(keyboard.exists)
                XCTAssertTrue(rail.isHittable)
                XCTAssertFalse(sidebar.exists)
            }
            try await waitForText(name, in: location)
            app.typeText(text)
            try await waitForText(text, in: terminal)
        }
        XCTAssertFalse((terminal.value as? String ?? "").contains(scoutText))
        agent("Scout").tap()
        try await waitForText(scoutText, in: terminal)
        XCTAssertFalse((terminal.value as? String ?? "").contains(reviewText))
        XCTAssertTrue(keyboard.exists)
        app.typeText("-back")
        try await waitForText(scoutText + "-back", in: terminal)

        toggle.tap()
        XCTAssertTrue(keyboard.waitForNonExistence(timeout: 5))
        XCTAssertTrue(sidebar.waitForExistence(timeout: 5))
        sidebar.tap()
        agent("Reviewer").tap()
        try await waitForText(reviewText, in: terminal)
        XCTAssertFalse(keyboard.exists)
        XCTAssertTrue(rail.isHittable)
    }

    @MainActor
    func testKeyboardTransitionKeepsTerminalFocusAndSidebarChoice() async throws {
        let app = XCUIApplication()
        let (surface, _) = try connect(in: app, sessionVariable: "SPACEPORT_TEST_DUO_SESSION")
        let sidebar = app.buttons["session.sidebar.hide"]
        let rail = app.buttons["session.rail.expand"]
        XCTAssertTrue(sidebar.waitForExistence(timeout: 5))
        let originalWidth = surface.frame.width
        let toggle = app.buttons["terminal.keyboard.toggle"]
        let keyboard = app.keyboards.firstMatch
        let pane = app.buttons["session.navigation"].value as? String
        attachScreen("Terminal before keyboard", in: app)

        toggle.tap()
        XCTAssertTrue(keyboard.waitForExistence(timeout: 5))
        XCTAssertTrue(rail.isHittable)
        XCTAssertGreaterThan(surface.frame.width, originalWidth)
        app.buttons["terminal.key.ctrl"].tap()
        app.typeText("u")
        let pending = "motion-" + UUID().uuidString.prefix(6).lowercased()
        app.typeText(pending)
        try await waitForText(pending, in: surface)
        toggle.tap()
        XCTAssertTrue(sidebar.waitForExistence(timeout: 5))
        XCTAssertEqual(surface.frame.width, originalWidth, accuracy: 1)

        sidebar.tap()
        toggle.tap()
        XCTAssertTrue(keyboard.waitForExistence(timeout: 5))
        toggle.tap()
        XCTAssertTrue(keyboard.waitForNonExistence(timeout: 5))
        XCTAssertTrue(rail.isHittable)
        XCTAssertEqual(app.buttons["session.navigation"].value as? String, pane)
        try await waitForText(pending, in: surface)
    }

    @MainActor
    func testHeaderHomeReturnsToSessionList() throws {
        let app = XCUIApplication()
        let (terminal, _) = try connect(in: app, sessionVariable: "SPACEPORT_TEST_DUO_SESSION")
        let home = app.buttons["session.header.home"]
        let compose = app.buttons["session.compose"]
        guard home.waitForExistence(timeout: 5) else {
            XCTFail("The terminal header needs a Home button beside Compose.")
            return
        }
        XCTAssertEqual(home.label, "Home")
        XCTAssertTrue(home.isHittable)
        XCTAssertTrue(compose.isHittable)
        attachScreen("Connected session Home action", in: app)
        app.buttons["terminal.keyboard.toggle"].tap()
        XCTAssertTrue(app.keyboards.firstMatch.waitForExistence(timeout: 5))
        home.tap()
        XCTAssertTrue(app.buttons["Settings"].waitForExistence(timeout: 5))
        XCTAssertFalse(terminal.exists)
        XCTAssertFalse(app.keyboards.firstMatch.exists)
    }

    @MainActor
    func testDuoTerminalControlsUseAvailableHeight() throws {
        let app = XCUIApplication()
        let (surface, _) = try connect(in: app, sessionVariable: "SPACEPORT_TEST_DUO_SESSION")
        let sidebar = app.buttons["session.sidebar.hide"]
        XCTAssertTrue(sidebar.waitForExistence(timeout: 8), "Run on the unfolded Duo.")
        let toggle = app.buttons["terminal.keyboard.toggle"]
        let details = app.buttons["session.hostDetails"]
        let navigation = app.buttons["session.navigation"]
        let keyboard = app.keyboards.firstMatch
        XCTAssertFalse(keyboard.exists)
        // XCTest reports the Duo's app frame in portrait even when its inner display is landscape.
        let displayHeight = min(app.frame.width, app.frame.height)
        XCTAssertGreaterThan(toggle.frame.maxX, displayHeight, "Run on the landscape inner display.")
        let gap = (displayHeight - toggle.frame.maxY).rounded()
        print("Duo spacing app=\(app.frame) terminal=\(surface.frame) toggle=\(toggle.frame) details=\(details.frame) navigation=\(navigation.frame) gap=\(gap)")
        attachScreen("Duo unfolded toolbar spacing", in: app)
        XCTAssertLessThanOrEqual(gap, 12)
        XCTAssertGreaterThanOrEqual(gap, 0)
        XCTAssertEqual(details.frame.midY, toggle.frame.midY, accuracy: 2)
        for button in [navigation, sidebar, details, toggle] {
            XCTAssertGreaterThanOrEqual(button.frame.width, 44)
            XCTAssertGreaterThanOrEqual(button.frame.height, 44)
            XCTAssertTrue(button.isHittable)
        }
        toggle.tap()
        XCTAssertTrue(keyboard.waitForExistence(timeout: 5))
        let dockGap = keyboard.frame.minY - toggle.frame.maxY
        print("Duo spacing keyboard=\(keyboard.frame) dockedToggle=\(toggle.frame) gap=\(dockGap)")
        XCTAssertGreaterThanOrEqual(dockGap, 0)
        XCTAssertLessThanOrEqual(dockGap, 8, "Allow the system keyboard’s own top inset.")
        XCTAssertTrue(app.buttons["session.rail.expand"].isHittable)
    }

    @MainActor
    func testDuoKeepsReviewerInputWhenFolding() async throws {
        try XCTSkipUnless(ProcessInfo.processInfo.environment["SPACEPORT_TEST_DUO_SESSION"] != nil,
                          "Open the Duo simulator and select a disposable seed-agent-order session.")
        let app = XCUIApplication()
        let (surface, _) = try connect(in: app, sessionVariable: "SPACEPORT_TEST_DUO_SESSION")
        let sidebar = app.buttons["session.sidebar.hide"]
        guard sidebar.waitForExistence(timeout: 8) else {
            XCTFail("The open display must show the agent sidebar beside the terminal.")
            return
        }
        let navigation = app.buttons["session.navigation"]
        let expand = app.buttons["session.rail.expand"]
        XCTAssertTrue(app.buttons["session.header.find"].exists)
        XCTAssertTrue(app.descendants(matching: .any)["session.sidebar.search"].firstMatch.exists)
        sidebar.tap()
        XCTAssertTrue(expand.waitForExistence(timeout: 5))
        expand.tap()
        XCTAssertTrue(sidebar.waitForExistence(timeout: 5))
        attachScreen("Duo inner display with agent sidebar", in: app)
        app.buttons["terminal.keyboard.toggle"].tap()
        let keyboard = app.keyboards.firstMatch
        XCTAssertTrue(keyboard.waitForExistence(timeout: 5))
        XCTAssertTrue(expand.waitForExistence(timeout: 5))
        XCTAssertFalse(sidebar.exists)
        let rail = app.buttons.matching(NSPredicate(format: "identifier BEGINSWITH %@", "session.rail.agent."))
        let scout = rail.matching(NSPredicate(format: "label BEGINSWITH %@", "Scout")).firstMatch
        let reviewer = rail.matching(NSPredicate(format: "label BEGINSWITH %@", "Reviewer")).firstMatch
        let later = app.buttons["session.interrupt.later"]
        let jump = app.buttons["session.interrupt.jump"]
        let originalLine = "mission-" + UUID().uuidString.prefix(8)
        app.typeText(originalLine)
        try await waitForText(originalLine, in: surface)
        XCTContext.runActivity(named: "Report Scout blocked with sequence 2 on the fixture") { _ in
            XCTAssertTrue(later.waitForExistence(timeout: 45))
        }
        try await waitForText("Mission", in: navigation)
        attachScreen("Duo typing with a waiting agent", in: app)
        later.tap()
        XCTAssertTrue(later.waitForNonExistence(timeout: 5))
        app.typeText("-stayed")
        try await waitForText(originalLine + "-stayed", in: surface)
        XCTContext.runActivity(named: "Report Scout working with sequence 3 on the fixture") { _ in
            let working = XCTNSPredicateExpectation(predicate: NSPredicate(format: "label CONTAINS[c] %@", "working"), object: scout)
            XCTAssertEqual(XCTWaiter.wait(for: [working], timeout: 45), .completed)
        }
        XCTContext.runActivity(named: "Report Scout blocked with sequence 4 on the fixture") { _ in
            XCTAssertTrue(jump.waitForExistence(timeout: 45))
        }
        jump.tap()
        try await waitForText("Scout", in: navigation)
        XCTAssertFalse((surface.value as? String ?? "").contains(originalLine))
        XCTAssertTrue(keyboard.exists)
        reviewer.tap()
        try await waitForText("Review is ready.", in: surface)
        let prefix = "duo-" + String(UUID().uuidString.prefix(6)).lowercased()
        app.typeText(prefix)
        try await waitForText(prefix, in: surface)
        attachScreen("Duo inner display typing with agent rail", in: app)

        XCTContext.runActivity(named: "Close iPhone Duo in Device Hub") { _ in
            XCTAssertTrue(expand.waitForNonExistence(timeout: 45))
        }
        XCTAssertTrue(keyboard.exists)
        XCTAssertTrue((surface.value as? String ?? "").contains(prefix))
        XCTAssertFalse((surface.value as? String ?? "").contains("accepted-" + prefix))
        attachScreen("Duo outer display with pending input", in: app)
        app.typeText("-folded")
        try await waitForText(prefix + "-folded", in: surface)
        let waiting = app.buttons["session.header.waiting"]
        XCTAssertTrue(waiting.label.contains("Scout"))
        waiting.tap()
        try await waitForText("Scout", in: navigation)
        XCTAssertTrue(keyboard.exists)
        XCTAssertTrue(waiting.label.contains("Reviewer"))
        waiting.tap()
        try await waitForText(prefix + "-folded", in: surface)

        XCTContext.runActivity(named: "Open iPhone Duo in Device Hub") { _ in
            XCTAssertTrue(expand.waitForExistence(timeout: 45))
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
        XCTAssertLessThanOrEqual(editor.frame.maxX, surface.frame.minX)
        attachScreen("Duo inline draft with live terminal", in: app)
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
        XCTAssertTrue(surface.waitForExistence(timeout: 5))
        XCTAssertFalse((surface.value as? String ?? "").contains(draft))
    }
}
