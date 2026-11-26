// ABOUTME: Opens a shell beside a live agent over an actual Herdr SSH session.
// ABOUTME: Checks that the new tab stays in the agent's workspace and receives input.
import XCTest

extension SessionWorkflowTests {
    @MainActor
    func testNewTerminalBesideAnAgent() async throws {
        let app = XCUIApplication()
        let (terminal, _) = try connect(in: app, sessionVariable: "SPACEPORT_TEST_NEW_TERMINAL_SESSION")
        let location = app.buttons["session.navigation"]
        location.tap()
        app.buttons.matching(NSPredicate(format: "identifier BEGINSWITH %@", "goTo.needsYou.")).firstMatch.tap()
        try await waitForText("Review is ready.", in: terminal)
        try await waitForText("Flight", in: location)

        app.buttons["terminal.keyboard.toggle"].tap()
        XCTAssertTrue(app.keyboards.firstMatch.waitForExistence(timeout: 5))
        let create = app.buttons["session.header.newTerminal"]
        guard create.waitForExistence(timeout: 3) else {
            return XCTFail("The connected terminal needs a New Terminal button beside Home.")
        }
        XCTAssertEqual(create.label, "New Terminal")
        attachScreen("New Terminal beside Home", in: app)
        create.tap()
        let changed = expectation(for: NSPredicate(format: "value CONTAINS 'Flight' AND NOT (value CONTAINS 'Review')"),
                                  evaluatedWith: location)
        await fulfillment(of: [changed], timeout: 10)
        XCTAssertTrue(app.keyboards.firstMatch.waitForExistence(timeout: 5))
        let marker = String(UUID().uuidString.prefix(8))
        app.typeText("printf '%s%s\\n' new-shell- " + marker + "\n")
        try await waitForText("new-shell-" + marker, in: terminal)
        app.buttons["session.header.waiting"].tap()
        try await waitForText("Review is ready.", in: terminal)
        XCTAssertFalse((terminal.value as? String ?? "").contains(marker))
    }
}
