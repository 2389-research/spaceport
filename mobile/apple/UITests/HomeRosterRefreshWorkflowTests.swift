// ABOUTME: Changes a real Herdr agent while its iOS session is parked on Home.
// ABOUTME: Verifies the new status arrives without reopening the agent or entering its password.
import XCTest

extension SessionWorkflowTests {
    @MainActor
    func testHomeRefreshesAgentChangedInParkedSession() async throws {
        let app = XCUIApplication()
        let variable = "SPACEPORT_TEST_HOME_ROSTER_SESSION"
        _ = try connect(in: app, sessionVariable: variable)
        let name = try XCTUnwrap(ProcessInfo.processInfo.environment[variable])
        let quoted = "'" + name.replacingOccurrences(of: "'", with: "'\\''") + "'"
        openSessionHome(in: app)
        let reviewer = app.buttons.matching(NSPredicate(
            format: "identifier BEGINSWITH %@ AND label BEGINSWITH %@ AND label CONTAINS %@",
            "home.agent.", "Reviewer", name)).firstMatch
        try await waitForText("Needs you", in: reviewer)
        let (shell, _) = try connect(in: app, sessionVariable: variable, suffix: "-control", launch: false)
        app.buttons["terminal.keyboard.toggle"].tap()
        XCTAssertTrue(app.keyboards.firstMatch.waitForExistence(timeout: 5))
        let prefix = "herdr --session " + quoted
        app.typeText("p=$(" + prefix + " agent list | jq -er '.result.agents[] | select(.agent==\"Reviewer\") | .pane_id')\n")
        app.typeText(prefix + " pane report-agent \"$p\" --source mobile-qa --agent Reviewer --state working --seq 2 && printf '%s%s\\n' roster- changed\n")
        try await waitForText("roster-changed", in: shell)
        openSessionHome(in: app)
        try await waitForText("Working", in: reviewer)
        XCTAssertFalse(app.secureTextFields["SSH password"].exists)
    }
}
