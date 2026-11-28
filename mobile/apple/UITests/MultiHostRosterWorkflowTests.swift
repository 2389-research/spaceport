// ABOUTME: Checks live agents on two distinct SSH endpoints through the iOS Home screen.
// ABOUTME: Both disposable sessions use seed-attention and accept one unique reviewer reply.
import XCTest

extension SessionWorkflowTests {
    @MainActor
    func testTwoHostsStayLiveAndOpeningAnAgentTargetsItsOwnTerminal() async throws {
        let env = ProcessInfo.processInfo.environment
        let primary = "SPACEPORT_TEST_MULTIHOST_SESSION"
        let secondary = "SPACEPORT_TEST_SECOND_MULTIHOST_SESSION"
        guard let a = env[primary], let b = env[secondary],
              let hostA = env["SPACEPORT_TEST_SSH_HOST"], let portA = env["SPACEPORT_TEST_SSH_PORT"],
              let hostB = env["SPACEPORT_TEST_SECOND_SSH_HOST"], let portB = env["SPACEPORT_TEST_SECOND_SSH_PORT"]
        else { throw XCTSkip("Select two disposable SSH hosts, each with a fresh seed-attention session.") }
        guard let witness = env["SPACEPORT_TEST_ROSTER_WITNESS"].flatMap(URL.init(string:)),
              witness.scheme == "http", witness.host == "127.0.0.1" else {
            throw XCTSkip("Run serve-multihost-roster against the second disposable host on the simulator's Mac.")
        }
        try XCTSkipUnless(hostA != hostB || portA != portB, "The two SSH endpoints must be distinct.")
        try XCTSkipUnless(a != b, "Give the two Reviewer sessions distinct fixture names.")

        let app = XCUIApplication()
        let (_, firstSession) = try connect(in: app, sessionVariable: primary)
        openSessionHome(in: app)
        let (terminal, secondSession) = try connect(in: app, sessionVariable: secondary, launch: false, fixture: "SPACEPORT_TEST_SECOND")
        openSessionHome(in: app)

        let live = NSPredicate(format: "value BEGINSWITH %@", "Live")
        // Home virtualizes offscreen rows. Inspect each saved session after scrolling it into view.
        for row in [firstSession, secondSession] {
            reveal(row, in: app)
            let confirmed = expectation(for: live, evaluatedWith: row)
            await fulfillment(of: [confirmed], timeout: 15)
        }
        let firstReviewer = app.buttons.matching(NSPredicate(format: "identifier BEGINSWITH %@ AND label BEGINSWITH %@ AND label CONTAINS %@",
                                                             "home.agent.", "Reviewer,", ", \(a),")).firstMatch
        reveal(firstReviewer, in: app, towardTop: true)
        var request = URLRequest(url: witness)
        request.httpMethod = "POST"
        request.timeoutInterval = 8
        let (data, response) = try await URLSession.shared.data(for: request)
        XCTAssertEqual((response as? HTTPURLResponse)?.statusCode, 200)
        let name = try XCTUnwrap(String(data: data, encoding: .utf8))
        XCTAssertTrue(name.hasPrefix("Signal-"))
        let fresh = app.buttons.matching(NSPredicate(format: "identifier BEGINSWITH %@ AND label BEGINSWITH %@ AND label CONTAINS %@",
                                                      "home.agent.", name + ",", ", \(b),")).firstMatch
        XCTAssertTrue(fresh.waitForExistence(timeout: 15), "The second host's new agent must appear without reopening its terminal.")
        let nonce = UUID().uuidString.prefix(8)
        var receipts: [String] = []
        for (index, session) in [a, b].enumerated() {
            let agent = app.buttons.matching(NSPredicate(format: "identifier BEGINSWITH %@ AND label BEGINSWITH %@ AND label CONTAINS %@",
                                                         "home.agent.", "Reviewer,", ", \(session),")).firstMatch
            reveal(agent, in: app, towardTop: true)
            agent.tap()
            try await waitForText("Review is ready.", in: terminal)
            for receipt in receipts {
                XCTAssertFalse((terminal.value as? String ?? "").contains(receipt))
            }
            let reply = "host-\(index)-\(nonce)"
            app.buttons["terminal.keyboard.toggle"].tap()
            app.typeText(reply + "\n")
            let receipt = "accepted-" + reply
            try await waitForText(receipt, in: terminal)
            receipts.append(receipt)
            if app.keyboards.firstMatch.exists { app.buttons["terminal.keyboard.toggle"].tap() }
            openSessionHome(in: app)
        }
    }
}
