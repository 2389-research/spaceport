// ABOUTME: Verifies the visible app palette on Home and in a presented sheet.
// ABOUTME: Checks the actual background of a live SSH terminal against its surrounding app.
import UIKit
import XCTest

extension SessionWorkflowTests {
    @MainActor
    func testTerminalAppearanceAppliesToHomeAndSheets() throws {
        let app = XCUIApplication()
        app.launch()
        XCTAssertTrue(app.buttons["Settings"].waitForExistence(timeout: 5))
        func choose(_ mode: String) {
            app.buttons["Settings"].tap()
            let option = app.segmentedControls["settings.appearance"].buttons[mode]
            XCTAssertTrue(option.waitForExistence(timeout: 5))
            option.tap()
            app.navigationBars.buttons.element(boundBy: 0).tap()
        }
        for mode in ["Light", "Dark"] {
            choose(mode)
            app.terminate()
            app.launch()
            XCTAssertEqual(try usesLightBackground(app), mode == "Light")
            let welcome = app.staticTexts["home.welcome.title"].exists
            attachScreen(mode + (welcome ? " Spaceport welcome" : " Agents"), in: app)
            app.buttons[welcome ? "home.empty.addHost" : "Add Host"].tap()
            XCTAssertTrue(app.textFields["Address"].waitForExistence(timeout: 5))
            XCTAssertEqual(try usesLightBackground(app), mode == "Light")
            attachScreen(mode + " Add Host", in: app)
            app.buttons["Cancel"].tap()
            app.buttons["Settings"].tap()
            XCTAssertTrue(app.segmentedControls["settings.appearance"].buttons[mode].isSelected)
            app.navigationBars.buttons.element(boundBy: 0).tap()
        }
    }

    @MainActor
    func testLightTerminalAndHomeAgreeOverSSH() throws {
        let app = XCUIApplication()
        app.launchArguments += ["-terminalAppearance", "light"]
        _ = try connect(in: app, sessionVariable: "SPACEPORT_TEST_APPEARANCE_SESSION")
        XCTAssertTrue(try usesLightBackground(app))
        attachScreen("Light SSH terminal", in: app)
        app.buttons["session.header.home"].tap()
        XCTAssertTrue(app.buttons["Settings"].waitForExistence(timeout: 5))
        XCTAssertTrue(try usesLightBackground(app))
    }

    // Sample the clear margin beside the content, away from text and buttons.
    @MainActor
    private func usesLightBackground(_ app: XCUIApplication) throws -> Bool {
        let image = try XCTUnwrap(app.screenshot().image.cgImage)
        let rect = CGRect(x: CGFloat(image.width) * 0.03, y: CGFloat(image.height) * 0.55, width: 1, height: 1)
        let sample = try XCTUnwrap(image.cropping(to: rect.integral))
        let context = try XCTUnwrap(CGContext(data: nil, width: 1, height: 1, bitsPerComponent: 8, bytesPerRow: 1,
                                              space: CGColorSpaceCreateDeviceGray(), bitmapInfo: CGImageAlphaInfo.none.rawValue))
        context.draw(sample, in: CGRect(x: 0, y: 0, width: 1, height: 1))
        return try XCTUnwrap(context.data).load(as: UInt8.self) > 128
    }
}
