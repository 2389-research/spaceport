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

    @MainActor
    func testFloatingToolbarKeepsArmedKeysWhenDocked() throws {
        for mode in ["light", "dark"] {
            try checkFloatingToolbar(appearance: mode)
        }
    }

    @MainActor
    private func checkFloatingToolbar(appearance: String) throws {
        let app = XCUIApplication()
        defer { app.terminate() }
        app.launchArguments += ["-terminalAppearance", appearance]
        // Nine keys page on a portrait phone and fit in one strip in landscape.
        app.launchArguments += ["-terminalToolbarKeys", "ctrl,esc,tab|shiftTab,shift,alt|left,enter,right"]
        XCUIDevice.shared.orientation = .portrait
        defer { XCUIDevice.shared.orientation = .portrait }
        _ = try connect(in: app, sessionVariable: "SPACEPORT_TEST_APPEARANCE_SESSION")
        let control = app.buttons["terminal.key.ctrl"]
        let toggle = app.buttons["terminal.keyboard.toggle"]
        control.tap()
        XCTAssertTrue(control.isSelected)
        XCTAssertGreaterThanOrEqual(try keycapContrast(control), 4.5)
        attachScreen("\(appearance) floating toolbar portrait", in: app)

        let alt = app.buttons["terminal.key.alt"]
        if !alt.isHittable {
            let tab = app.buttons["terminal.key.tab"]
            tab.coordinate(withNormalizedOffset: CGVector(dx: 0.9, dy: 0.5))
                .press(forDuration: 0.1, thenDragTo: control.coordinate(withNormalizedOffset: CGVector(dx: 0.1, dy: 0.5)))
            XCTAssertTrue(alt.isHittable)
        }
        alt.tap()
        XCTAssertTrue(alt.isSelected)
        toggle.tap()
        let keyboard = app.keyboards.firstMatch
        XCTAssertTrue(keyboard.waitForExistence(timeout: 5))
        let keyboardTutorial = keyboard.buttons["Continue"]
        if keyboardTutorial.exists { keyboardTutorial.tap() }
        XCTAssertTrue(alt.isSelected)
        attachScreen("\(appearance) docked toolbar portrait", in: app)
        toggle.tap()
        XCTAssertTrue(app.keyboards.firstMatch.waitForNonExistence(timeout: 5))
        XCTAssertTrue(alt.isSelected)

        XCUIDevice.shared.orientation = .landscapeLeft
        XCTAssertTrue(control.waitForExistence(timeout: 5))
        XCTAssertTrue(control.isHittable && control.isSelected)
        XCTAssertTrue(alt.isHittable && alt.isSelected)
        let landscape = XCTAttachment(screenshot: XCUIScreen.main.screenshot())
        landscape.name = "\(appearance) floating toolbar landscape"
        landscape.lifetime = .keepAlways
        add(landscape)
    }

    @MainActor
    private func keycapContrast(_ key: XCUIElement) throws -> Double {
        let image = try XCTUnwrap(key.screenshot().image.cgImage)
        // The center contains the glyph and its selected fill, clear of the circle's edge.
        let bounds = CGRect(x: 0, y: 0, width: image.width, height: image.height)
        let center = bounds.insetBy(dx: bounds.width * 0.30, dy: bounds.height * 0.38).integral
        let sample = try XCTUnwrap(image.cropping(to: center))
        let gray = try XCTUnwrap(CGColorSpace(name: CGColorSpace.linearGray))
        let bitmap = try XCTUnwrap(CGContext(data: nil, width: sample.width, height: sample.height,
                                             bitsPerComponent: 8, bytesPerRow: sample.width, space: gray,
                                             bitmapInfo: CGImageAlphaInfo.none.rawValue))
        bitmap.draw(sample, in: CGRect(origin: .zero, size: center.size))
        let data = try XCTUnwrap(bitmap.data).assumingMemoryBound(to: UInt8.self)
        let pixels = UnsafeBufferPointer(start: data, count: sample.width * sample.height)
        let light = Double(try XCTUnwrap(pixels.max())) / 255
        let dark = Double(try XCTUnwrap(pixels.min())) / 255
        return (light + 0.05) / (dark + 0.05)
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
