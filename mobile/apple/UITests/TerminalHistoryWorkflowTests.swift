// ABOUTME: Finds old shell output through the iOS UI against a disposable SSH host.
// ABOUTME: Checks wrapped traversal, match copying and return to live Ghostty input.
import XCTest

extension SessionWorkflowTests {
    @MainActor
    func testFindInGhosttyTerminalHistory() async throws {
        let app = XCUIApplication()
        let (terminal, _) = try connect(in: app, sessionVariable: "SPACEPORT_TEST_HERDR_SESSION", suffix: "-find-g")
        app.buttons["terminal.keyboard.toggle"].tap()
        XCTAssertTrue(app.keyboards.firstMatch.waitForExistence(timeout: 5))
        let nonce = String(UUID().uuidString.prefix(5)).lowercased()
        let query = "spf" + nonce
        app.typeText("n=\(nonce)\n")
        app.typeText("echo older-spf$n\n")
        try await waitForText("older-" + query, in: terminal)
        app.typeText("echo newer-spf$n\n")
        try await waitForText("newer-" + query, in: terminal)
        app.typeText("seq 1 120; echo ready-$n\n")
        try await waitForText("ready-" + nonce, in: terminal)
        XCTAssertFalse((terminal.value as? String ?? "").contains(query))
        app.buttons["session.navigation"].tap()
        app.buttons["session.find"].tap()
        let field = app.textFields["history.query"]
        XCTAssertTrue(field.waitForExistence(timeout: 5))
        field.tap()
        field.typeText(query)
        app.buttons["history.submit"].tap()
        let snippet = app.staticTexts["history.excerpt"]
        func waitForSnippet(_ text: String) async {
            let found = expectation(for: NSPredicate(format: "label CONTAINS %@", text), evaluatedWith: snippet)
            await fulfillment(of: [found], timeout: 10)
        }
        await waitForSnippet("newer-" + query)
        app.buttons["history.older"].tap()
        await waitForSnippet("older-" + query)
        app.buttons["history.older"].tap()
        await waitForSnippet("newer-" + query)
        app.buttons["history.newer"].tap()
        await waitForSnippet("older-" + query)
        app.buttons["history.copy"].tap()
        XCTAssertTrue(app.buttons["history.copy"].label.contains("Copied"))
        attachScreen("Ghostty history match", in: app)
        app.buttons["history.done"].tap()
        XCTAssertTrue(app.buttons["session.navigation"].waitForExistence(timeout: 5))
        XCTAssertFalse(app.navigationBars["Go To"].exists)
        XCTAssertTrue(app.keyboards.firstMatch.waitForExistence(timeout: 5))
    }
}
