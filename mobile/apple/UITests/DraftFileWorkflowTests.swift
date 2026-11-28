// ABOUTME: Delivers a real Files document through the iPhone composer to an SSH shell.
// ABOUTME: Requires the fixture document in the simulator's On My iPhone storage.
import XCTest

extension SessionWorkflowTests {
    @MainActor
    func testDocumentPickerDeliversFileToSelectedTerminal() async throws {
        let app = XCUIApplication()
        let (terminal, _) = try connect(in: app, sessionVariable: "SPACEPORT_UI_FILE_SESSION")
        app.buttons["terminal.keyboard.toggle"].tap()
        XCTAssertTrue(app.keyboards.firstMatch.waitForExistence(timeout: 5))
        app.typeText("printf '%s%s\\n' file- ready; IFS= read -r f; cat -- \"$f\"\n")
        try await waitForText("file-ready", in: terminal)
        app.buttons["session.compose"].tap()
        app.buttons["draft.chooseFile"].tap()
        let browse = app.buttons["Browse"].firstMatch
        if browse.waitForExistence(timeout: 3) { browse.tap() }
        let local = app.descendants(matching: .any).matching(NSPredicate(format: "label == %@", "On My iPhone")).firstMatch
        if local.waitForExistence(timeout: 3) { local.tap() }
        let file = app.descendants(matching: .any).matching(NSPredicate(format: "label BEGINSWITH %@", "SpaceportFixture")).firstMatch
        guard file.waitForExistence(timeout: 8) else { XCTFail("Missing SpaceportFixture.txt in On My iPhone.\n" + app.debugDescription); return }
        file.tap()
        let open = app.buttons["Open"].firstMatch
        if open.exists && open.isHittable { open.tap() }
        XCTAssertTrue(app.buttons["draft.removeFile"].waitForExistence(timeout: 8))
        let toggle = app.switches["draft.pressReturn"]
        if toggle.value as? String != "1" { toggle.tap() }
        app.buttons["draft.send"].tap()
        try await waitForText("verified-native-file-content", in: terminal)
    }
}
