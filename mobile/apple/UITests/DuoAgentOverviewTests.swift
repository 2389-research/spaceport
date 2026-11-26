// ABOUTME: Checks the expanded Duo header and sidebar against a live Herdr roster.
// ABOUTME: Switches from a shell to an agent with a reported terminal title.
import XCTest

extension SessionWorkflowTests {
    @MainActor
    func testDuoShowsAgentTitleAndSessionCounts() async throws {
        let app = XCUIApplication()
        let (terminal, _) = try connect(in: app, sessionVariable: "SPACEPORT_TEST_OVERVIEW_SESSION")
        let header = app.buttons["session.navigation"]
        try await waitForText("Mission", in: header)
        let reviewer = app.buttons.matching(NSPredicate(format: "identifier BEGINSWITH %@ AND label == %@",
                                                        "goTo.needsYou.", "Reviewer")).firstMatch
        reviewer.tap()
        try await waitForText("Review is ready.", in: terminal)
        let title = "Review the launch checklist"
        let titled = expectation(for: NSPredicate(format: "value BEGINSWITH %@", title), evaluatedWith: header)
        await fulfillment(of: [titled], timeout: 3)
        let counts = app.staticTexts["session.agentCounts"]
        XCTAssertEqual(counts.label, "1 waiting · 1 working · 2 idle")
        XCTAssertTrue(app.buttons["session.hostDetails"].isHittable)
        attachScreen("Duo agent title and counts", in: app)

        app.buttons["terminal.keyboard.toggle"].tap()
        app.typeText("approved\n")
        try await waitForText("accepted-approved", in: terminal)
        app.buttons["terminal.keyboard.toggle"].tap()
        let updated = expectation(for: NSPredicate(format: "label == %@", "1 working · 3 idle"), evaluatedWith: counts)
        await fulfillment(of: [updated], timeout: 5)
    }
}
