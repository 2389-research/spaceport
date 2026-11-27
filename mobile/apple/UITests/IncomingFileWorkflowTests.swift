// ABOUTME: Sends an actual document from Files to the installed Share extension.
// ABOUTME: Verifies the main app receives the local attachment for an explicitly chosen terminal.
import XCTest

extension SessionWorkflowTests {
    @MainActor
    func testFilesShareStagesAttachmentForCompose() throws {
        let app = XCUIApplication()
        _ = try connect(in: app, sessionVariable: "SPACEPORT_UI_SHARE_SESSION")
        let files = XCUIApplication(bundleIdentifier: "com.apple.DocumentsApp")
        files.launch()
        let done = files.buttons["Done"].firstMatch
        if done.exists && done.isHittable { done.tap() }
        let browse = files.buttons["Browse"].firstMatch
        if browse.waitForExistence(timeout: 3) { browse.tap() }
        let local = files.descendants(matching: .any).matching(NSPredicate(format: "label == %@", "On My iPhone")).firstMatch
        if local.waitForExistence(timeout: 3) { local.tap() }
        let document = files.descendants(matching: .any).matching(NSPredicate(format: "label BEGINSWITH %@", "SpaceportFixture")).firstMatch
        guard document.waitForExistence(timeout: 8) else { XCTFail("Fixture document unavailable in Files.\n" + files.debugDescription); return }
        files.buttons["OverflowBarButtonItem"].tap()
        files.buttons["Select"].firstMatch.tap()
        document.tap()
        files.buttons["Share"].firstMatch.tap()
        let destination = files.cells.matching(NSPredicate(format: "label == %@ OR label == %@", "Spaceport", "Send to Spaceport")).firstMatch
        if !destination.waitForExistence(timeout: 3) {
            let more = files.cells.matching(NSPredicate(format: "label == %@", "More")).firstMatch
            if more.waitForExistence(timeout: 3) { more.tap() }
        }
        guard destination.waitForExistence(timeout: 5) else { XCTFail("Spaceport missing in share sheet.\n" + files.debugDescription); return }
        destination.tap()
        let save = files.buttons["Save to Spaceport"].firstMatch
        guard save.waitForExistence(timeout: 8) else { XCTFail("Extension did not open.\n" + files.debugDescription); return }
        save.tap()
        let saved = expectation(for: NSPredicate(format: "exists == false"), evaluatedWith: save)
        wait(for: [saved], timeout: 8)
        app.activate()
        XCTAssertTrue(app.wait(for: .runningForeground, timeout: 10))
        let compose = app.buttons["session.compose"]
        let ready = expectation(for: NSPredicate(format: "hittable == true AND enabled == true"), evaluatedWith: compose)
        wait(for: [ready], timeout: 15)
        compose.tap()
        let attach = app.buttons["Attach shared file"]
        XCTAssertTrue(attach.waitForExistence(timeout: 6))
        attach.tap()
        XCTAssertTrue(app.buttons["draft.removeFile"].waitForExistence(timeout: 6))
    }
}
