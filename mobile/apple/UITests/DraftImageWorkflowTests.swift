// ABOUTME: Pastes a real screenshot and note into an isolated SSH-backed Herdr terminal.
// ABOUTME: Verifies the remote shell can open the image and read its PNG signature.
import UIKit
import XCTest

extension SessionWorkflowTests {
    @MainActor
    func testScreenshotAndNoteReachGhosttyOverSSH() async throws {
        try await pasteScreenshot(usingGhostty: true)
    }

    @MainActor
    func testScreenshotAndNoteReachHerdrSurfaceOverSSH() async throws {
        try await pasteScreenshot(usingGhostty: false)
    }

    @MainActor
    private func pasteScreenshot(usingGhostty: Bool) async throws {
        guard ProcessInfo.processInfo.environment["SPACEPORT_UI_IMAGE_SESSION"] != nil else {
            throw XCTSkip("Select a disposable Herdr session with a shell pane.")
        }
        let app = XCUIApplication()
        app.launchArguments += ["-useGhosttyTerminal", usingGhostty ? "YES" : "NO"]
        let (terminal, _) = try connect(in: app, sessionVariable: "SPACEPORT_UI_IMAGE_SESSION",
                                         suffix: usingGhostty ? "" : "-surface",
                                         terminalIdentifier: usingGhostty ? "terminal.ghostty" : "Terminal display")
        let firstFrame = expectation(for: NSPredicate(format: "value MATCHES %@", #"(?s).*\S.*"#), evaluatedWith: terminal)
        await fulfillment(of: [firstFrame], timeout: 10)
        app.buttons["terminal.keyboard.toggle"].tap()
        XCTAssertTrue(app.keyboards.firstMatch.waitForExistence(timeout: 5))
        app.typeText("printf '%s%s\\n' image- ready; IFS=' ' read -r f note; " +
                     "if [ \"$(od -An -tx1 -N8 \"$f\" | tr -d ' \\n')\" = 89504e470d0a1a0a ]; " +
                     "then printf 'verified-%s\\n' \"$note\"; fi\n")
        try await waitForText("image-ready", in: terminal)
        let data = UIGraphicsImageRenderer(size: CGSize(width: 16, height: 12)).pngData { context in
            UIColor.systemTeal.setFill()
            context.fill(CGRect(x: 0, y: 0, width: 16, height: 12))
        }
        UIPasteboard.general.setItems([["public.png": data]], options: [.localOnly: true])
        app.buttons["session.compose"].tap()
        let paste = app.buttons["draft.pasteImage"]
        XCTAssertTrue(paste.waitForExistence(timeout: 5))
        paste.tap()
        XCTAssertTrue(app.buttons["draft.removeImage"].waitForExistence(timeout: 10))
        // The system paste banner overlays the navigation bar outside the app's accessibility tree.
        let bannerCleared = ContinuousClock.now + .seconds(4)
        let note = "note" + UUID().uuidString.prefix(6).lowercased()
        let editor = app.textViews["draft.editor"]
        editor.tap()
        editor.typeText(note)
        let toggle = app.switches["draft.pressReturn"]
        if toggle.value as? String != "1" { toggle.tap() }
        attachScreen("Screenshot preview with note and confirmed destination", in: app)
        try await Task.sleep(until: bannerCleared, clock: .continuous)
        app.buttons["draft.send"].tap()
        try await waitForText("verified-" + note, in: terminal)
    }
}
