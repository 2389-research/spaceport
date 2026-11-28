// ABOUTME: Exercises the embedded terminal with a live Metal surface and byte transport.
// ABOUTME: Checks that remote clipboard control sequences leave the device pasteboard alone.
import SwiftUI
import XCTest
@testable import SpaceportMobile

@MainActor
final class GhosttyPaneTests: XCTestCase {
    func testWarmSwitchWaitsForQueuedOutputToReachTheGhosttyGrid() async throws {
        let surface = GhosttyPaneSurface(write: { _ in true }, resize: { _ in })
        let parsed = expectation(description: "Incoming screen is readable")
        surface.receive(Data("Incoming agent".utf8)) {
            XCTAssertTrue(surface.visibleText?.contains("Incoming agent") == true)
            parsed.fulfill()
        }
        let scene = try XCTUnwrap(UIApplication.shared.connectedScenes.compactMap { $0 as? UIWindowScene }.first)
        let host = UIHostingController(rootView: GhosttyPaneView(surface: surface, keyboard: .constant(false), modifiers: .constant(0)))
        let window = UIWindow(windowScene: scene)
        window.rootViewController = host
        window.makeKeyAndVisible()
        defer { window.isHidden = true }
        host.view.layoutIfNeeded()
        await fulfillment(of: [parsed], timeout: 5)
    }

    func testStartingFontSizeOnTheNativeTerminal() throws {
        let scene = try XCTUnwrap(UIApplication.shared.connectedScenes.compactMap { $0 as? UIWindowScene }.first)
        func fontSize(_ surface: GhosttyPaneSurface) throws -> Float {
            let host = UIHostingController(rootView: GhosttyPaneView(surface: surface, keyboard: .constant(false), modifiers: .constant(0)))
            let window = UIWindow(windowScene: scene)
            window.rootViewController = host
            window.makeKeyAndVisible()
            defer { window.isHidden = true }
            host.view.layoutIfNeeded()
            func terminal(in view: UIView) -> PhoneGhosttyView? {
                if let native = view as? PhoneGhosttyView { return native }
                return view.subviews.lazy.compactMap { terminal(in: $0) }.first
            }
            return try XCTUnwrap(terminal(in: host.view)?.configuration.fontSize)
        }
        let standard = GhosttyPaneSurface(write: { _ in true }, resize: { _ in })
        let custom = GhosttyPaneSurface(write: { _ in true }, resize: { _ in }, textSize: 17)
        XCTAssertEqual(try fontSize(standard), 12)
        XCTAssertEqual(try fontSize(custom), 17)
    }

    func testProgramBackgroundAndResetFollowTheGhosttySurface() async throws {
        let controller = GhosttyPaneSurface.controller
        let previous = controller.theme
        controller.setTheme(TerminalAppearance.dark.theme)
        defer { controller.setTheme(previous) }
        let surface = GhosttyPaneSurface(write: { _ in true }, resize: { _ in })
        let host = UIHostingController(rootView: GhosttyPaneView(surface: surface, keyboard: .constant(false), modifiers: .constant(0)))
        let scene = try XCTUnwrap(UIApplication.shared.connectedScenes.compactMap { $0 as? UIWindowScene }.first)
        let window = UIWindow(windowScene: scene)
        window.rootViewController = host
        window.makeKeyAndVisible()
        defer { window.isHidden = true }
        host.view.layoutIfNeeded()
        func check(_ sequence: String, becomes mode: TerminalAppearance) async {
            let changed = expectation(description: "Ghostty reported the terminal background")
            let observation = surface.$reportedAppearance.first(where: { $0 == mode }).sink { _ in changed.fulfill() }
            surface.receive(Data(sequence.utf8))
            await fulfillment(of: [changed], timeout: 5)
            observation.cancel()
        }
        await check("\u{1b}]11;#ffffee\u{7}", becomes: .light)
        await check("\u{1b}]111\u{7}", becomes: .dark)
        controller.setTheme(TerminalAppearance.light.theme)
        XCTAssertEqual(TerminalAppearance(background: controller.backgroundColor), .light)
    }

    func testRemoteScreenAndClipboardPolicy() async throws {
        let surface = GhosttyPaneSurface(write: { _ in true }, resize: { _ in })
        let host = UIHostingController(rootView: GhosttyPaneView(surface: surface, keyboard: .constant(false), modifiers: .constant(0)))
        let scene = try XCTUnwrap(UIApplication.shared.connectedScenes.compactMap { $0 as? UIWindowScene }.first)
        let window = UIWindow(windowScene: scene)
        window.rootViewController = host
        window.makeKeyAndVisible()
        defer { window.isHidden = true }
        let start = UIPasteboard.general.changeCount
        surface.receive(Data("\u{1b}]52;c;Z2hvc3R0eQ==\u{7}Spaceport ready".utf8))
        let deadline = ContinuousClock.now + .seconds(5)
        while surface.visibleText?.contains("Spaceport ready") != true, ContinuousClock.now < deadline {
            try await Task.sleep(for: .milliseconds(50))
        }
        XCTAssertTrue(surface.visibleText?.contains("Spaceport ready") == true)
        XCTAssertEqual(UIPasteboard.general.changeCount, start)
    }
}

extension GhosttyPaneTests {
    func testComposerSendsOneFramedPasteAndKeepsRefusedInput() {
        let wanted = Data("\u{1b}[200~line one\nline two\u{1b}[201~\r".utf8)
        let attached = GhosttyPaneSurface(write: { $0 == wanted }, resize: { _ in })
        XCTAssertTrue(attached.paste("line one\nline two", pressReturn: true))
        XCTAssertFalse(attached.paste("different", pressReturn: true))
        let rejectsControls = GhosttyPaneSurface(write: { _ in XCTFail("unsafe input reached SSH"); return true },
                                                resize: { _ in })
        XCTAssertFalse(rejectsControls.paste("\u{1b}[201~\u{03}"))
        XCTAssertFalse(rejectsControls.paste("\u{9b}201~"))
        XCTAssertFalse(rejectsControls.paste(String(repeating: "x", count: 65_524), pressReturn: true))
        let boundary = GhosttyPaneSurface(write: { $0.count == 65_536 }, resize: { _ in })
        XCTAssertTrue(boundary.paste(String(repeating: "x", count: 65_523), pressReturn: true))
    }
}
