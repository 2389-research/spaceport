// ABOUTME: Drives profile persistence and the real SSH workflow through the app.
// ABOUTME: Runs live checks only against an explicitly configured disposable host.
import CryptoKit
import XCTest

final class SessionWorkflowTests: XCTestCase {
    @MainActor
    func testAgentOrderInLiveSessionAndSavedHome() async throws {
        let app = XCUIApplication()
        let (terminal, _) = try connect(in: app, sessionVariable: "SPACEPORT_TEST_ORDER_SESSION")
        let authenticated = try await sampleSSHIfConfigured()
        let keyboard = app.keyboards.firstMatch
        XCTAssertFalse(keyboard.exists)
        app.buttons["session.navigation"].tap()
        let live = app.buttons.matching(NSPredicate(format: "identifier BEGINSWITH %@ OR identifier BEGINSWITH %@",
                                                    "goTo.agent.", "goTo.needsYou."))
        let liveOrder = ["Scout", "Reviewer", "Zephyr", "Atlas"]
        let homeOrder = ["Reviewer", "Scout", "Zephyr", "Atlas"]
        func names(in rows: XCUIElementQuery) -> [String] {
            rows.allElementsBoundByIndex.map { $0.label.components(separatedBy: ", ").first ?? "" }
        }
        XCTAssertEqual(names(in: live), liveOrder)
        let liveReviewer = live.matching(NSPredicate(format: "label BEGINSWITH %@", "Reviewer")).firstMatch
        XCTAssertTrue((liveReviewer.value as? String)?.contains("Review the launch checklist") == true)
        attachScreen("Live agent order", in: app)
        app.buttons["goTo.done"].tap()
        XCTAssertFalse(keyboard.exists)
        app.buttons["session.home"].tap()
        let session = try XCTUnwrap(ProcessInfo.processInfo.environment["SPACEPORT_TEST_ORDER_SESSION"])
        let saved = app.buttons.matching(NSPredicate(format: "identifier BEGINSWITH %@ AND label CONTAINS %@",
                                                     "home.agent.", session))
        XCTAssertEqual(names(in: saved), homeOrder)
        let savedReviewer = saved.matching(NSPredicate(format: "label BEGINSWITH %@", "Reviewer")).firstMatch
        XCTAssertTrue((savedReviewer.value as? String)?.contains("Review the launch checklist") == true)
        XCTAssertTrue(app.staticTexts["Waiting for you"].exists)
        XCTAssertTrue(app.staticTexts["Idle"].exists)
        attachScreen("Saved agent order", in: app)
        let location = app.buttons["session.navigation"]
        let homeStart = ProcessInfo.processInfo.systemUptime
        saved.matching(NSPredicate(format: "label BEGINSWITH %@", "Scout,")).firstMatch.tap()
        try await waitForText("Scout", in: location)
        attachElapsed("Home agent tap to verified location", since: homeStart)
        app.buttons["terminal.keyboard.toggle"].tap()
        XCTAssertTrue(keyboard.waitForExistence(timeout: 5))
        location.tap()
        let bar = app.navigationBars["Go To"]
        let start = bar.coordinate(withNormalizedOffset: CGVector(dx: 0.5, dy: 0.1))
        let end = app.coordinate(withNormalizedOffset: CGVector(dx: 0.5, dy: 0.85))
        start.press(forDuration: 0.1, thenDragTo: end)
        XCTAssertTrue(app.buttons["goTo.done"].waitForNonExistence(timeout: 5))
        XCTAssertTrue(keyboard.waitForExistence(timeout: 5))
        location.tap()
        app.buttons["goTo.done"].tap()
        XCTAssertTrue(keyboard.waitForExistence(timeout: 5))
        location.tap()
        let switchStart = ProcessInfo.processInfo.systemUptime
        live.matching(NSPredicate(format: "label BEGINSWITH %@", "Atlas,")).firstMatch.tap()
        try await waitForText("Atlas", in: location)
        attachElapsed("Live agent tap to verified location", since: switchStart)
        XCTAssertTrue(keyboard.waitForExistence(timeout: 5))
        let input = app.textViews["Terminal input"]
        let marker = "quick-" + String(UUID().uuidString.prefix(5)).lowercased()
        input.typeText(marker)
        try await waitForText(marker, in: terminal)
        app.buttons["terminal.key.ctrl"].tap()
        input.typeText("u")
        let locationFrame = location.frame
        XCTAssertEqual(locationFrame.midY, app.buttons["terminal.key.esc"].frame.midY, accuracy: 2)
        XCTAssertEqual(locationFrame.midY, app.buttons["session.compose"].frame.midY, accuracy: 2)
        attachScreen("Keyboard after switching agents", in: app)
        if let authenticated {
            let after = try await sshWitness("sample")
            XCTAssertEqual(after, authenticated, "Choosing two agents in this session must reuse its SSH login.")
        }
        app.buttons["session.home"].tap()
        app.terminate()
        app.launch()
        XCTAssertEqual(names(in: saved), homeOrder)
        XCTAssertTrue((savedReviewer.value as? String)?.contains("Review the launch checklist") == true)
    }

    @MainActor
    func testLongPressOffersNativeEditingForTerminalText() throws {
        let app = XCUIApplication()
        let (terminal, _) = try connect(in: app, sessionVariable: "SPACEPORT_TEST_HERDR_SESSION", suffix: "-menu")
        terminal.coordinate(withNormalizedOffset: CGVector(dx: 0.08, dy: 0.02)).press(forDuration: 0.8)
        let copy = app.menuItems["Copy"]
        XCTAssertTrue(copy.waitForExistence(timeout: 4))
        XCTAssertTrue(app.menuItems["Paste"].exists)
        let selectAll = app.menuItems["Select All"]
        XCTAssertTrue(selectAll.exists)
        attachScreen("Terminal text edit menu", in: app)
        selectAll.tap()
        XCTAssertTrue(app.navigationBars["Terminal text"].waitForExistence(timeout: 4))
        XCTAssertFalse((app.textViews.firstMatch.value as? String ?? "").isEmpty)
    }

    @MainActor
    func testInstallPhoneKeyOnAnUntrustedHostFromHome() async throws {
        try requireHardwareKey()
        let env = ProcessInfo.processInfo.environment
        guard let host = env["SPACEPORT_TEST_KEY_INSTALL_HOST"],
              let port = env["SPACEPORT_TEST_SSH_PORT"],
              let user = env["SPACEPORT_TEST_SSH_USER"],
              let password = env["SPACEPORT_TEST_SSH_PASSWORD"],
              let fingerprint = env["SPACEPORT_TEST_SSH_FINGERPRINT"] else {
            throw XCTSkip("Configure a disposable, untrusted SSH host for standalone key installation.")
        }
        let app = XCUIApplication()
        app.launch()
        let session = "key-" + UUID().uuidString.prefix(8)
        addHost(in: app, hostname: host, port: port, user: user, session: session)
        let decline = app.buttons["It Doesn't Match"]
        guard decline.waitForExistence(timeout: 10), presentedFingerprint(in: app) == fingerprint else {
            XCTFail("Use a new fixture address with an independently verified fingerprint.")
            return
        }
        XCUIDevice.shared.press(.home)
        app.activate()
        guard decline.waitForExistence(timeout: 5), presentedFingerprint(in: app) == fingerprint else {
            XCTFail("The original trust challenge disappeared while comparing its fingerprint outside the app.")
            return
        }
        decline.tap()
        app.buttons["Settings"].tap()
        attachScreen("Phone key", in: app)
        let installToHost = app.buttons["Install on a Host…"]
        guard installToHost.waitForExistence(timeout: 5) else {
            XCTFail("The Phone key screen has no action to install the key on a saved host.")
            return
        }
        installToHost.tap()
        let destination = "\(user)@\(host)" + (port == "22" ? "" : ":\(port)")
        let hostRow = app.buttons[destination]
        reveal(hostRow, in: app)
        hostRow.tap()
        let passwordField = app.secureTextFields["SSH password"]
        passwordField.tap()
        passwordField.typeText(password)
        app.buttons["Install key"].tap()
        let trust = app.buttons["Trust host"]
        guard trust.waitForExistence(timeout: 8), presentedFingerprint(in: app) == fingerprint else {
            XCTFail("The installer did not ask to trust the fixture's exact SSH host fingerprint.")
            return
        }
        trust.tap()
        XCTAssertTrue(app.staticTexts["Key installed"].waitForExistence(timeout: 15))
        attachScreen("Standalone key installed", in: app)
        app.terminate()
        app.launch()
        let saved = app.buttons.matching(identifier: "\(destination), \(session)").firstMatch
        reveal(saved, in: app)
        saved.tap()
        let terminal = app.descendants(matching: .any)["Terminal display"].firstMatch
        XCTAssertTrue(terminal.waitForExistence(timeout: 20))
        app.buttons["terminal.keyboard.toggle"].tap()
        let input = app.textViews["Terminal input"]
        let writable = expectation(for: NSPredicate(format: "value == 'Ready'"), evaluatedWith: input)
        await fulfillment(of: [writable], timeout: 10)
        let nonce = UUID().uuidString.prefix(8)
        input.typeText("printf '%s%s\\n' enclave- " + nonce + "\n")
        try await waitForText("enclave-" + nonce, in: terminal)
    }

    @MainActor
    func testReplyToBlockedPaneInAnotherWorkspace() async throws {
        let app = XCUIApplication()
        let (terminal, _) = try connect(in: app, sessionVariable: "SPACEPORT_TEST_ATTENTION_SESSION")
        let goTo = app.buttons["session.navigation"]
        let origin = try XCTUnwrap(goTo.value as? String)
        let originalSSH = try await sampleSSHIfConfigured()
        XCTAssertTrue(origin.contains("Mission"))
        XCTAssertEqual(goTo.label, "Agents")
        goTo.tap()
        let attention = app.buttons.matching(NSPredicate(format: "identifier BEGINSWITH %@", "goTo.needsYou.")).firstMatch
        XCTAssertTrue(attention.waitForExistence(timeout: 5))
        XCTAssertTrue((attention.value as? String ?? "").contains("Flight"))
        attachScreen("Agents with a blocked reviewer", in: app)
        app.buttons["goTo.done"].tap()
        XCTAssertEqual(goTo.value as? String, origin)
        app.buttons["session.home"].tap()
        let session = try XCTUnwrap(ProcessInfo.processInfo.environment["SPACEPORT_TEST_ATTENTION_SESSION"])
        let homeAgent = app.buttons.matching(NSPredicate(format: "identifier BEGINSWITH %@ AND label CONTAINS %@ AND label CONTAINS %@",
                                                         "home.agent.", "Review", session)).firstMatch
        XCTAssertTrue(homeAgent.waitForExistence(timeout: 5))
        attachScreen("Home with blocked reviewer", in: app)
        let workspaces = app.buttons["home.workspaces"]
        reveal(workspaces, in: app)
        workspaces.tap()
        let chooseSession = app.buttons.matching(NSPredicate(format: "identifier BEGINSWITH %@ AND label ENDSWITH %@",
                                                             "home.workspaces.", ", " + session)).firstMatch
        if chooseSession.waitForExistence(timeout: 2) {
            attachScreen("Choose a session for Workspaces", in: app)
            chooseSession.tap()
        }
        let mission = app.buttons.matching(NSPredicate(format: "identifier BEGINSWITH %@ AND label == %@", "goTo.workspace.", "Mission")).firstMatch
        guard mission.waitForExistence(timeout: 12) else {
            XCTFail("Reopening Workspaces did not reuse the authenticated session.")
            return
        }
        attachScreen("Workspaces from Home", in: app)
        app.buttons["goTo.done"].tap()
        XCTAssertEqual(goTo.value as? String, origin)
        app.buttons["session.home"].tap()
        reveal(homeAgent, in: app, towardTop: true)
        homeAgent.tap()
        guard terminal.waitForExistence(timeout: 15) else {
            XCTFail("Opening the saved reviewer did not reuse the authenticated session.")
            return
        }
        try await waitForText("Review is ready.", in: terminal)
        XCUIDevice.shared.press(.home)
        XCTAssertTrue(app.state == .runningBackgroundSuspended || app.wait(for: .runningBackground, timeout: 5))
        app.activate()
        try await waitForText("Review is ready.", in: terminal)
        if let originalSSH {
            let currentSSH = try await sshWitness("sample")
            XCTAssertEqual(currentSSH, originalSSH, "Navigation and foregrounding must keep the same authenticated SSH worker.")
        }
        app.buttons["session.navigation"].tap()
        mission.tap()
        app.buttons["goTo.done"].tap()
        try await waitForText("Mission", in: goTo)
        goTo.tap()
        guard attention.waitForExistence(timeout: 5) else {
            XCTFail("Go To omitted Flight’s blocked Review pane.")
            return
        }
        XCTAssertTrue(attention.label.contains("Review"))
        XCTAssertTrue((attention.value as? String ?? "").contains("Flight"))
        attention.tap()
        try await waitForText("Review is ready.", in: terminal)
        XCTAssertTrue((goTo.value as? String ?? "").contains("Review"))
        let keyboard = app.buttons["terminal.keyboard.toggle"]
        keyboard.tap()
        XCTAssertTrue(app.keyboards.firstMatch.waitForExistence(timeout: 5))
        XCTAssertEqual(terminal.frame.minX, app.frame.minX, accuracy: 1)
        XCTAssertEqual(terminal.frame.maxX, app.frame.maxX, accuracy: 1)
        attachScreen("Full screen terminal with keyboard", in: app)
        keyboard.tap()

        let reply = "approve-" + UUID().uuidString.prefix(8)
        app.buttons["session.compose"].tap()
        let editor = app.textViews["draft.editor"]
        editor.tap()
        editor.typeText(reply)
        setPressReturn(true, in: app)
        XCTAssertEqual(app.buttons["draft.send"].label, "Send")
        app.buttons["draft.send"].tap()
        try await waitForText("accepted-" + reply, in: terminal)

        app.buttons["session.navigation"].tap()
        let sibling = app.buttons.matching(NSPredicate(format: "identifier BEGINSWITH %@ AND selected == false", "pane.map.")).firstMatch
        reveal(sibling, in: app)
        XCTAssertEqual(sibling.label, "Logs")
        sibling.tap()
        try await waitForText("Log stream connected.", in: terminal)
        XCTAssertFalse((terminal.value as? String ?? "").contains(reply))
        app.buttons["session.home"].tap()
        let recentlyIdle = app.staticTexts["Idle · under 1 hour"]
        reveal(recentlyIdle, in: app)
        XCTAssertTrue(recentlyIdle.exists)
        reveal(homeAgent, in: app)
        XCTAssertTrue((homeAgent.value as? String ?? "").contains("<1m"))
        attachScreen("Home after a witnessed idle transition", in: app)
    }

    @MainActor
    func waitForText(_ text: String, in element: XCUIElement) async throws {
        let check = expectation(for: NSPredicate(format: "value CONTAINS %@", text), evaluatedWith: element)
        await fulfillment(of: [check], timeout: 10)
    }

    @MainActor
    func testSaveEditAndRemoveSessionAcrossLaunches() throws {
        let app = XCUIApplication()
        app.launch()
        let name = "ui-\(UUID().uuidString.prefix(8)).example"
        addHost(in: app, hostname: name, port: "2202", user: "operator", session: "lab")
        app.terminate()
        app.launch()
        let destination = "operator@\(name):2202"
        let row = app.buttons["\(destination), lab"]
        reveal(row, in: app)
        let detail = app.buttons["Host details for \(destination)"]
        detail.tap()
        app.buttons["Edit"].tap()
        attachScreen("Edit Host", in: app)
        XCTAssertEqual(app.textFields["Address"].value as? String, destination)
        let label = app.textFields["Name"]
        label.tap()
        label.typeText("UI flight")
        app.buttons["Save"].tap()
        app.buttons["Add Session"].tap()
        attachScreen("New Session", in: app)
        let session = app.textFields["Session"]
        session.tap()
        session.typeText("telemetry")
        app.buttons["Add"].tap()
        app.navigationBars.buttons.element(boundBy: 0).tap()
        app.terminate()
        app.launch()
        reveal(row, in: app)
        XCTAssertTrue(app.buttons["\(destination), telemetry"].exists)
        detail.tap()
        let remove = app.buttons["Remove Host"]
        for _ in 0..<4 {
            if remove.exists && remove.isHittable { break }
            app.swipeUp()
        }
        remove.tap()
        let confirmation = app.sheets.firstMatch
        confirmation.buttons["Remove Host"].tap()
        XCTAssertFalse(row.exists)
    }

    @MainActor
    func testInputOverRealPinnedSSH() async throws {
        let app = XCUIApplication()
        try XCTSkipUnless(ProcessInfo.processInfo.environment["SPACEPORT_TEST_HERDR_SESSION"] != nil,
                          "Select a disposable Herdr session.")
        app.launch()
        app.buttons["Settings"].tap()
        attachScreen("Settings", in: app)
        app.buttons["settings.toolbarKeys"].tap()
        let secondPageKey = app.buttons["toolbar.page.1.slot.0"]
        secondPageKey.tap()
        app.buttons["Up Arrow"].tap()
        attachScreen("Toolbar keys", in: app)
        app.terminate()
        let (terminal, row) = try connect(in: app, sessionVariable: "SPACEPORT_TEST_HERDR_SESSION")
        let left = app.buttons["terminal.key.ctrl"].coordinate(withNormalizedOffset: CGVector(dx: 0.1, dy: 0.5))
        let right = app.buttons["terminal.key.tab"].coordinate(withNormalizedOffset: CGVector(dx: 0.9, dy: 0.5))
        right.press(forDuration: 0.08, thenDragTo: left)
        XCTAssertTrue(app.buttons["terminal.key.up"].waitForExistence(timeout: 3))
        let start = app.buttons["terminal.key.up"].coordinate(withNormalizedOffset: CGVector(dx: 0.1, dy: 0.5))
        let end = app.buttons["terminal.key.alt"].coordinate(withNormalizedOffset: CGVector(dx: 0.9, dy: 0.5))
        start.press(forDuration: 0.08, thenDragTo: end)
        try await exercisePaneNavigation(in: app)
        let marker = UUID().uuidString.prefix(8)
        let compose = app.buttons["session.compose"]
        compose.tap()
        let editor = app.textViews["draft.editor"]
        let draft = "IOS_WITNESS=\(marker)\necho pasted-$IOS_WITNESS"
        editor.tap()
        editor.typeText(draft)
        XCTAssertEqual(editor.value as? String, draft)
        app.buttons["draft.done"].tap()
        XCTAssertFalse((terminal.value as? String ?? "").contains(String(marker)))
        compose.tap()
        XCTAssertEqual(editor.value as? String, draft)
        setPressReturn(false, in: app)
        XCTAssertEqual(app.buttons["draft.send"].label, "Paste")
        attachScreen("Multiline terminal draft", in: app)
        app.buttons["draft.send"].tap()
        XCTAssertTrue(compose.waitForExistence(timeout: 5))
        app.buttons["terminal.keyboard.toggle"].tap()
        let input = app.textViews["Terminal input"]
        let ready = expectation(for: NSPredicate(format: "value == 'Ready'"), evaluatedWith: input)
        await fulfillment(of: [ready], timeout: 10)
        // The shell can only join this suffix if Send left the command unsubmitted.
        input.typeText("-done\n")
        let output = NSPredicate(format: "value CONTAINS %@", "pasted-\(marker)-done")
        let observed = expectation(for: output, evaluatedWith: terminal)
        await fulfillment(of: [observed], timeout: 10)
        attachScreen("Connected Herdr terminal", in: app)
        app.buttons["terminal.keyboard.toggle"].tap()
        app.buttons["session.home"].tap()
        reveal(row, in: app)
        attachScreen("Home with last session", in: app)
        app.terminate()
        app.launch()
        reveal(row, in: app)
        row.tap()
        let password = app.secureTextFields["SSH password"]
        XCTAssertTrue(password.waitForExistence(timeout: 10))
        password.tap()
        password.typeText(try XCTUnwrap(ProcessInfo.processInfo.environment["SPACEPORT_TEST_SSH_PASSWORD"]))
        app.buttons["Sign In"].tap()
        XCTAssertTrue(terminal.waitForExistence(timeout: 20))
        XCUIDevice.shared.press(.home)
        app.activate()
        XCTAssertTrue(terminal.waitForExistence(timeout: 20))
        app.buttons["terminal.keyboard.toggle"].tap()
        let writable = expectation(for: NSPredicate(format: "value == 'Ready'"), evaluatedWith: input)
        await fulfillment(of: [writable], timeout: 10)
        input.typeText("printf '%s%s\\n' resumed- \"$IOS_WITNESS\"\n")
        let restored = expectation(for: NSPredicate(format: "value CONTAINS %@", "resumed-\(marker)"), evaluatedWith: terminal)
        await fulfillment(of: [restored], timeout: 10)
        app.buttons["session.home"].tap()
        app.buttons["Settings"].tap()
        app.buttons["settings.toolbarKeys"].tap()
        secondPageKey.tap()
        app.buttons["Shift Tab"].tap()
    }

    private func requireHardwareKey() throws {
        #if targetEnvironment(simulator)
        throw XCTSkip("A physical iPhone or iPad must sign the SSH challenge with its Secure Enclave.")
        #else
        try XCTSkipUnless(SecureEnclave.isAvailable, "This device must support Secure Enclave key generation.")
        #endif
    }

    private enum FixtureError: Error {
        case unexpectedHostKey
        case terminalUnavailable
    }

    @MainActor
    func connect(in app: XCUIApplication, sessionVariable: String, suffix: String = "", launch: Bool = true, terminalIdentifier: String = "Terminal display") throws -> (XCUIElement, XCUIElement) {
        let env = ProcessInfo.processInfo.environment
        guard let host = env["SPACEPORT_TEST_SSH_HOST"],
              let port = env["SPACEPORT_TEST_SSH_PORT"],
              let user = env["SPACEPORT_TEST_SSH_USER"],
              let password = env["SPACEPORT_TEST_SSH_PASSWORD"],
              let fingerprint = env["SPACEPORT_TEST_SSH_FINGERPRINT"],
              let sessionName = env[sessionVariable] else {
            throw XCTSkip("Select a disposable SSH host and configure \(sessionVariable) to run this workflow.")
        }
        let session = sessionName + suffix
        if launch { app.launch() }
        addHost(in: app, hostname: host, port: port, user: user, session: session, password: true)
        let dest = "\(user)@\(host)" + (port == "22" ? "" : ":\(port)")
        let saved = app.buttons.matching(identifier: "\(dest), \(session)")
        let row = saved.element(boundBy: max(0, saved.count - 1))
        let passwordField = app.secureTextFields["SSH password"]
        let trust = app.buttons["Trust and Connect"]
        if !passwordField.exists && trust.waitForExistence(timeout: 5) {
            guard presentedFingerprint(in: app) == fingerprint else { throw FixtureError.unexpectedHostKey }
            attachScreen("Connect verify host", in: app)
            trust.tap()
        }
        XCTAssertTrue(passwordField.waitForExistence(timeout: 8))
        attachScreen("Connect password", in: app)
        passwordField.tap()
        passwordField.typeText(password)
        app.buttons["Sign In"].tap()
        let terminal = app.descendants(matching: .any)[terminalIdentifier].firstMatch
        guard terminal.waitForExistence(timeout: 30) else {
            throw FixtureError.terminalUnavailable
        }
        return (terminal, row)
    }

    @MainActor
    private func presentedFingerprint(in app: XCUIApplication) -> String? {
        let fingerprint = app.descendants(matching: .any)["trust.fingerprint"].firstMatch
        return fingerprint.exists ? fingerprint.label : nil
    }

    @MainActor
    func reveal(_ element: XCUIElement, in app: XCUIApplication, towardTop: Bool = false) {
        for _ in 0..<10 {
            if element.exists && element.isHittable { return }
            if towardTop { app.swipeDown() } else { app.swipeUp() }
        }
        XCTAssertTrue(element.isHittable)
    }

    @MainActor
    private func setPressReturn(_ enabled: Bool, in app: XCUIApplication) {
        let toggle = app.switches["draft.pressReturn"]
        XCTAssertTrue(toggle.waitForExistence(timeout: 5))
        if (toggle.value as? String == "1") != enabled {
            // SwiftUI exposes the full row. The switch thumb is at its trailing edge.
            toggle.coordinate(withNormalizedOffset: CGVector(dx: 0.94, dy: 0.5)).tap()
        }
        XCTAssertEqual(toggle.value as? String, enabled ? "1" : "0")
    }

    @MainActor
    private func exercisePaneNavigation(in app: XCUIApplication) async throws {
        let navigation = app.buttons["session.navigation"]
        let path = try XCTUnwrap(navigation.value as? String)
        XCTAssertFalse(path.isEmpty)
        app.buttons["session.navigation"].tap()
        let tiles = app.buttons.matching(NSPredicate(format: "identifier BEGINSWITH %@", "pane.map."))
        let needsSplit = !tiles.firstMatch.exists
        app.buttons["goTo.done"].tap()
        XCTAssertEqual(navigation.value as? String, path)

        if needsSplit {
            let keyboard = app.buttons["terminal.keyboard.toggle"]
            keyboard.tap()
            let input = app.textViews["Terminal input"]
            let ready = expectation(for: NSPredicate(format: "value == 'Ready'"), evaluatedWith: input)
            await fulfillment(of: [ready], timeout: 10)
            input.typeText("herdr pane split --current --direction down --no-focus\n")
            keyboard.tap()
        }
        app.buttons["session.navigation"].tap()
        let other = tiles.matching(NSPredicate(format: "selected == false")).firstMatch
        reveal(other, in: app)
        XCTAssertTrue(other.waitForExistence(timeout: 8))
        let next = other.identifier
        attachScreen("Workspace and pane map", in: app)
        other.tap()
        XCTAssertTrue(navigation.waitForExistence(timeout: 5))
        app.buttons["session.navigation"].tap()
        let selected = app.buttons[next]
        reveal(selected, in: app)
        let focused = expectation(for: NSPredicate(format: "selected == true"), evaluatedWith: selected)
        await fulfillment(of: [focused], timeout: 8)
        app.buttons["goTo.done"].tap()
    }

    @MainActor
    func attachScreen(_ name: String, in app: XCUIApplication) {
        let image = XCTAttachment(screenshot: app.screenshot())
        image.name = name
        image.lifetime = .keepAlways
        add(image)
    }

    private func attachElapsed(_ name: String, since start: TimeInterval) {
        let elapsed = (ProcessInfo.processInfo.systemUptime - start) * 1_000
        let attachment = XCTAttachment(string: String(format: "%.0f ms including XCTest tap and accessibility polling", elapsed))
        attachment.name = name
        attachment.lifetime = .keepAlways
        add(attachment)
    }

    @MainActor
    private func addHost(in app: XCUIApplication, hostname: String, port: String, user: String, session: String, password: Bool = false) {
        let destination = "\(user)@\(hostname)" + (port == "22" ? "" : ":\(port)")
        app.navigationBars.buttons["Add Host"].tap()
        let address = app.textFields["Address"]
        XCTAssertTrue(address.waitForExistence(timeout: 5))
        attachScreen("Add Host form", in: app)
        address.tap()
        address.typeText(destination)
        let name = app.textFields["Session"]
        name.tap()
        if let current = name.value as? String, current != name.placeholderValue {
            name.typeText(String(repeating: XCUIKeyboardKey.delete.rawValue, count: current.count))
        }
        name.typeText(session)
        if password { app.buttons["Password"].tap() }
        attachScreen("Add Host", in: app)
        app.buttons["Connect"].tap()
    }
}

extension SessionWorkflowTests {
    @MainActor
    func testWarmSessionsKeepTheirOwnUnsentDraft() async throws {
        let app = XCUIApplication()
        let (terminal, firstRow) = try connect(in: app, sessionVariable: "SPACEPORT_TEST_LIFECYCLE_SESSION", suffix: "-a")
        let firstLogin = try await sampleSSHIfConfigured()
        let compose = app.buttons["session.compose"], editor = app.textViews["draft.editor"]
        let destination = app.buttons["draft.destination"], done = app.buttons["draft.done"]
        let text = "Unsent " + UUID().uuidString.prefix(8)
        compose.tap()
        editor.tap()
        editor.typeText(text)
        let originalDestination = destination.label
        done.tap()
        app.buttons["session.home"].tap()
        _ = try connect(in: app, sessionVariable: "SPACEPORT_TEST_LIFECYCLE_SESSION", suffix: "-b", launch: false)
        var bothLogins: SSHWitness?
        if let firstLogin {
            let sample = try await sshWitness("sample")
            XCTAssertEqual(sample.accepted, firstLogin.accepted + 1)
            XCTAssertEqual(sample.workers.count, 2)
            XCTAssertTrue(Set(firstLogin.workers).isSubset(of: Set(sample.workers)))
            bothLogins = sample
        }
        compose.tap()
        XCTAssertEqual(editor.value as? String ?? "", "")
        done.tap()
        app.buttons["session.home"].tap()
        reveal(firstRow, in: app)
        firstRow.tap()
        guard terminal.waitForExistence(timeout: 10) else {
            XCTFail("Opening the first saved session again lost its ready connection.")
            return
        }
        if let bothLogins {
            let returned = try await sshWitness("sample")
            XCTAssertEqual(returned, bothLogins)
        }
        XCTAssertFalse((terminal.value as? String ?? "").contains(text))
        compose.tap()
        XCTAssertEqual(editor.value as? String, text)
        XCTAssertEqual(destination.label, originalDestination)
        done.tap()
        if let firstLogin, let bothLogins {
            app.buttons["session.navigation"].tap()
            let disconnect = app.buttons["session.disconnect"]
            reveal(disconnect, in: app)
            disconnect.tap()
            var closed = try await sshWitness("sample")
            for _ in 0..<10 where closed.workers == bothLogins.workers {
                try await Task.sleep(for: .milliseconds(200))
                closed = try await sshWitness("sample")
            }
            XCTAssertEqual(closed.accepted, bothLogins.accepted)
            XCTAssertEqual(Set(closed.workers), Set(bothLogins.workers).subtracting(firstLogin.workers))
        }
    }

    private struct SSHWitness: Decodable, Equatable {
        let accepted: Int
        let workers: [Int]
    }

    @MainActor
    private func sampleSSHIfConfigured() async throws -> SSHWitness? {
        guard ProcessInfo.processInfo.environment["SPACEPORT_TEST_SSH_WITNESS"] == "1" else { return nil }
        let sample = try await sshWitness("sample")
        XCTAssertEqual(sample.workers.count, 1)
        return sample
    }

    private func sshWitness(_ action: String) async throws -> SSHWitness {
        let directory = FileManager.default.temporaryDirectory.appendingPathComponent("spaceport-ssh-witness")
        try FileManager.default.createDirectory(at: directory, withIntermediateDirectories: true)
        let id = UUID().uuidString
        let request = directory.appendingPathComponent(id + ".request")
        let response = directory.appendingPathComponent(id + ".json")
        try Data(action.utf8).write(to: request, options: .atomic)
        defer {
            try? FileManager.default.removeItem(at: request)
            try? FileManager.default.removeItem(at: response)
        }
        for _ in 0..<80 {
            if FileManager.default.fileExists(atPath: response.path) {
                return try JSONDecoder().decode(SSHWitness.self, from: Data(contentsOf: response))
            }
            try await Task.sleep(for: .milliseconds(100))
        }
        throw NSError(domain: "Fixture SSH witness", code: 1,
                      userInfo: [NSLocalizedDescriptionKey: "Start mobile/test-support/watch-ssh-continuity for this simulator."])
    }

    @MainActor
    func testRecoverFromDroppedSSHWithoutReplayingDraft() async throws {
        try await verifyTransportRecovery(suffix: "-loss", waitInBackground: false)
    }

    @MainActor
    func testRecoverFromDroppedSSHAfterSettlingInBackground() async throws {
        try await verifyTransportRecovery(suffix: "-background", waitInBackground: true)
    }

    @MainActor
    private func verifyTransportRecovery(suffix: String, waitInBackground: Bool) async throws {
        try XCTSkipUnless(ProcessInfo.processInfo.environment["SPACEPORT_TEST_SSH_WITNESS"] == "1",
                          "Start the host-side witness to interrupt the disposable SSH transport.")
        let app = XCUIApplication()
        let (terminal, _) = try connect(in: app, sessionVariable: "SPACEPORT_TEST_LIFECYCLE_SESSION", suffix: suffix)
        let before = try await sshWitness("sample")
        XCTAssertEqual(before.workers.count, 1)
        let draft = "unsent-" + UUID().uuidString.prefix(8)
        let compose = app.buttons["session.compose"]
        compose.tap()
        let editor = app.textViews["draft.editor"]
        editor.tap()
        editor.typeText(draft)
        app.buttons["draft.done"].tap()
        XCUIDevice.shared.press(.home)
        if waitInBackground {
            XCTAssertTrue(app.state == .runningBackgroundSuspended || app.wait(for: .runningBackground, timeout: 5))
            try await Task.sleep(for: .seconds(3))
        }
        let detached = try await sshWitness("drop")
        XCTAssertEqual(detached, before)
        app.activate()
        var after = try await sshWitness("sample")
        for _ in 0..<20 where after.workers.isEmpty || after == before {
            try await Task.sleep(for: .milliseconds(400))
            after = try await sshWitness("sample")
        }
        XCTAssertEqual(after.accepted, before.accepted + 1)
        XCTAssertEqual(after.workers.count, 1)
        XCTAssertNotEqual(after.workers, before.workers)
        guard terminal.waitForExistence(timeout: 10) else {
            attachScreen("Transport recovery failure", in: app)
            let message = app.staticTexts.allElementsBoundByIndex.map(\.label).joined(separator: ", ")
            XCTFail("The selected password-backed terminal did not recover after losing SSH. Visible text: \(message)")
            return
        }
        compose.tap()
        XCTAssertEqual(editor.value as? String, draft)
        XCTAssertFalse(app.buttons["draft.send"].isEnabled)
        XCTAssertTrue(app.buttons["draft.retarget"].exists)
        app.buttons["draft.done"].tap()
        app.buttons["terminal.keyboard.toggle"].tap()
        let input = app.textViews["Terminal input"]
        let ready = expectation(for: NSPredicate(format: "value == 'Ready'"), evaluatedWith: input)
        await fulfillment(of: [ready], timeout: 10)
        let nonce = UUID().uuidString.prefix(8)
        input.typeText("printf '%s%s\\n' restored- " + nonce + "\n")
        try await waitForText("restored-" + nonce, in: terminal)
        XCTAssertFalse((terminal.value as? String ?? "").contains(draft))
    }
}

extension SessionWorkflowTests {
    @MainActor
    func testGhosttySettingsShowTheChosenTextSize() {
        let app = XCUIApplication()
        app.launchArguments += ["-useGhosttyTerminal", "YES", "-terminalTextSize", "17"]
        app.launch()
        app.buttons["Settings"].tap()
        let size = app.descendants(matching: .any).matching(identifier: "settings.textSize").firstMatch
        XCTAssertTrue(size.waitForExistence(timeout: 5))
        XCTAssertTrue(size.label.contains("17 pt") || (size.value as? String)?.contains("17 pt") == true, size.debugDescription)
    }

    @MainActor
    func testHomeSetupAndInlinePhoneKeySettings() throws {
        let app = XCUIApplication()
        app.launch()
        if app.staticTexts["Add your first host"].exists {
            let addHost = app.buttons["home.empty.addHost"]
            XCTAssertTrue(addHost.exists)
            guard addHost.exists else { return }
            addHost.tap()
            XCTAssertTrue(app.textFields["Address"].waitForExistence(timeout: 5))
            app.buttons["Cancel"].tap()
        }
        app.buttons["Settings"].tap()
        XCTAssertTrue(app.buttons["Install on a Host…"].waitForExistence(timeout: 5))
        XCTAssertEqual(app.switches["settings.composeReturn"].label, "Press Return After Pasting")
        attachScreen("Settings and device key", in: app)
    }
}

extension SessionWorkflowTests {
    @MainActor
    func testChangedHostOpensDetailsWithoutReplacingPin() throws {
        let env = ProcessInfo.processInfo.environment
        guard let port = env["SPACEPORT_TEST_ROTATING_SSH_PORT"],
              let trusted = env["SPACEPORT_TEST_ROTATING_SSH_TRUSTED"],
              let presented = env["SPACEPORT_TEST_ROTATING_SSH_PRESENTED"] else {
            throw XCTSkip("Start the rotating loopback SSH fixture on the simulator host.")
        }
        XCTAssertNotEqual(trusted, presented)
        let app = XCUIApplication()
        app.launch()
        let session = "rotation-" + UUID().uuidString.prefix(8)
        addHost(in: app, hostname: "127.0.0.1", port: port, user: "pilot", session: session, password: true)
        let trust = app.buttons["Trust and Connect"]
        guard trust.waitForExistence(timeout: 10), presentedFingerprint(in: app) == trusted else {
            XCTFail("Expected the fixture's independently generated original host key.")
            return
        }
        trust.tap()
        let field = app.secureTextFields["SSH password"]
        guard field.waitForExistence(timeout: 8) else { XCTFail("Expected password sign-in."); return }
        field.tap()
        field.typeText("unused")
        app.buttons["Sign In"].tap()
        let details = app.buttons["Open Host Details"]
        XCTAssertTrue(details.waitForExistence(timeout: 10))
        let saved = app.descendants(matching: .any)["trust.savedFingerprint"].firstMatch
        XCTAssertEqual(saved.label, trusted)
        XCTAssertEqual(presentedFingerprint(in: app), presented)
        XCTAssertFalse(trust.exists)
        attachScreen("Changed host fingerprints", in: app)
        reveal(details, in: app)
        details.tap()
        XCTAssertTrue(app.buttons["Forget Host Key…"].waitForExistence(timeout: 5))
        XCTAssertEqual(presentedFingerprint(in: app), trusted)
        XCTAssertTrue(app.buttons[String(session)].exists)
    }
}

extension SessionWorkflowTests {
    @MainActor
    func testGhosttyRoutesInputToSelectedAgent() async throws {
        try XCTSkipUnless(ProcessInfo.processInfo.environment["SPACEPORT_TEST_GHOSTTY_SESSION"] != nil,
                          "Seed a fresh disposable reviewer with mobile/test-support/seed-attention.")
        let app = XCUIApplication()
        app.launch()
        app.buttons["Settings"].tap()
        let toggle = app.switches["settings.ghostty"]
        guard toggle.waitForExistence(timeout: 5) else {
            XCTFail("Settings must offer the Ghostty terminal experiment.")
            return
        }
        let wasEnabled = toggle.value as? String == "1"
        if !wasEnabled {
            toggle.coordinate(withNormalizedOffset: CGVector(dx: 0.94, dy: 0.5)).tap()
            addTeardownBlock { @MainActor in
                app.terminate()
                app.launch()
                app.buttons["Settings"].tap()
                if toggle.value as? String == "1" {
                    toggle.coordinate(withNormalizedOffset: CGVector(dx: 0.94, dy: 0.5)).tap()
                }
            }
        }
        app.navigationBars.buttons.element(boundBy: 0).tap()
        let (surface, _) = try connect(in: app, sessionVariable: "SPACEPORT_TEST_GHOSTTY_SESSION",
                                       launch: false, terminalIdentifier: "terminal.ghostty")
        let firstFrame = expectation(for: NSPredicate(format: "value MATCHES %@", #"(?s).*\S.*"#), evaluatedWith: surface)
        await fulfillment(of: [firstFrame], timeout: 10)
        let navigation = app.buttons["session.navigation"]
        XCTAssertTrue((navigation.value as? String ?? "").contains("Mission"))
        let keyboard = app.keyboards.firstMatch
        app.buttons["terminal.keyboard.toggle"].tap()
        XCTAssertTrue(keyboard.waitForExistence(timeout: 5))
        let marker = String(UUID().uuidString.prefix(6)).lowercased()
        app.typeText("printf '%s%s\\n' origin- " + marker + "\n")
        try await waitForText("origin-" + marker, in: surface)

        navigation.tap()
        let reviewer = app.buttons.matching(NSPredicate(format: "identifier BEGINSWITH %@", "goTo.needsYou.")).firstMatch
        XCTAssertTrue(reviewer.waitForExistence(timeout: 5))
        let startedSwitch = ProcessInfo.processInfo.systemUptime
        reviewer.tap()
        try await waitForText("Review is ready.", in: surface)
        attachElapsed("Ghostty agent switch", since: startedSwitch)
        XCTAssertFalse((surface.value as? String ?? "").contains("origin-" + marker))
        XCTAssertTrue(keyboard.waitForExistence(timeout: 5))
        navigation.tap()
        app.buttons["goTo.done"].tap()
        XCTAssertTrue(keyboard.waitForExistence(timeout: 5))
        let startedReply = ProcessInfo.processInfo.systemUptime
        app.typeText(marker + "\n")
        try await waitForText("accepted-" + marker, in: surface)
        attachElapsed("Ghostty reviewer reply", since: startedReply)
        attachScreen("Ghostty attached to the reviewer", in: app)
    }
}
