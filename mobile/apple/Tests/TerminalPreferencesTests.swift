// ABOUTME: Checks persisted keyboard layouts and the geometry used by terminal rendering.
// ABOUTME: Covers custom slots, corrupt preferences and host resize dimensions.
import SwiftUI
import XCTest
@testable import SpaceportMobile

final class TerminalPreferencesTests: XCTestCase {
    @MainActor
    func testToolbarUsesOneRowAtPhoneAndTabletWidths() {
        for width: CGFloat in [208, 320, 375, 669, 951] {
            let sidebar = SessionLayout.sidebarWidth(in: width, regular: width >= 600)
            let terminalWidth = width - (sidebar ?? 0)
            let toolbar = SessionToolbar(width: terminalWidth, keyboard: true, modifiers: .constant(0),
                                         sendKey: { _, _ in }, toggleKeyboard: {})
            let host = UIHostingController(rootView: toolbar)
            let size = host.sizeThatFits(in: CGSize(width: terminalWidth, height: 500))
            XCTAssertLessThanOrEqual(size.height, 48, "Toolbar height at width \(width)")
            XCTAssertLessThanOrEqual(size.width, terminalWidth)
        }
    }

    @MainActor
    func testOnlyTheDockedToolbarPaintsTheOuterMargin() throws {
        for keyboard in [false, true] {
            let toolbar = SessionToolbar(width: 720, keyboard: keyboard, modifiers: .constant(0),
                                         sendKey: { _, _ in }, toggleKeyboard: {})
            let image = try XCTUnwrap(ImageRenderer(content: toolbar.frame(width: 720)).cgImage)
            let margin = try XCTUnwrap(image.cropping(to: CGRect(x: 0, y: 23, width: 1, height: 1)))
            let context = try XCTUnwrap(CGContext(data: nil, width: 1, height: 1, bitsPerComponent: 8,
                                                  bytesPerRow: 4, space: CGColorSpaceCreateDeviceRGB(),
                                                  bitmapInfo: CGImageAlphaInfo.premultipliedLast.rawValue))
            context.draw(margin, in: CGRect(x: 0, y: 0, width: 1, height: 1))
            let opacity = try XCTUnwrap(context.data).load(fromByteOffset: 3, as: UInt8.self)
            XCTAssertEqual(opacity, keyboard ? 255 : 0)
        }
    }

    @MainActor
    func testOpenTerminalHeaderFitsOneControlRow() {
        let header = TerminalHeader(view: nil, location: "A long session name on the open display",
                                    address: SSHAddress(hostname: "example.test", port: 22, username: "pilot"),
                                    folded: false, composing: false, navigate: {}, select: { _ in },
                                    find: {}, home: {}, newTerminal: {}, compose: {})
        let host = UIHostingController(rootView: header)
        for width: CGFloat in [320, 590] {
            let size = host.sizeThatFits(in: CGSize(width: width, height: 120))
            XCTAssertEqual(size.height, 44, accuracy: 0.5)
            XCTAssertLessThanOrEqual(size.width, width)
        }
    }

    func testCustomPagesSurviveSavingAndDeletingPages() throws {
        var keys = TerminalKeyPages.standard
        keys.addPage()
        keys.replace(page: 2, slot: 1, with: .pageDown)
        keys.removePage(0)
        let suite = "spaceport-tests." + UUID().uuidString
        let defaults = try XCTUnwrap(UserDefaults(suiteName: suite))
        defer { defaults.removePersistentDomain(forName: suite) }
        defaults.set(keys.rawValue, forKey: TerminalPreferences.toolbarKeys)
        let saved = try XCTUnwrap(defaults.string(forKey: TerminalPreferences.toolbarKeys))
        let reloaded = try XCTUnwrap(TerminalKeyPages(rawValue: saved))
        XCTAssertEqual(reloaded.pages, [[.shiftTab, .shift, .alt], [.left, .pageDown, .right]])
        keys.removePage(1)
        keys.removePage(0)
        XCTAssertEqual(keys.pages, [[.shiftTab, .shift, .alt]])
    }

    func testMalformedOrUnknownSlotsDoNotBecomeCommands() {
        for value in ["", "ctrl,esc", "ctrl,esc,tab,alt", "ctrl,rm,tab", "ctrl,esc,tab|"] {
            XCTAssertNil(TerminalKeyPages(rawValue: value))
        }
    }

    @MainActor
    func testTerminalSelectionFollowsVisibleCellsAndIncludesWideGlyphs() {
        func cell(_ text: String, skipped: Bool = false, hidden: Bool = false) -> MobileCell {
            MobileCell(symbol: text, fg: 0, bg: 0, modifier: hidden ? 128 : 0, skip: skipped)
        }
        let surface = MobileTerminal(width: 8, height: 1,
                                     cells: [cell("a"), cell("猫"), cell("", skipped: true), cell("b"),
                                             cell(" "), cell("x", hidden: true), cell("!"), cell(" ")],
                                     cursorX: nil, cursorY: nil)
        let display = TerminalDisplay(terminal: surface, metrics: TerminalMetrics(textSize: 13))
        let size = CGSize(width: 64, height: 17)
        let wide = display.word(at: CGPoint(x: 17, y: 8), in: size)
        XCTAssertEqual(wide?.text, "a猫b")
        XCTAssertEqual(wide?.rect, CGRect(x: 0, y: 0, width: 32, height: 17))
        XCTAssertNil(display.word(at: CGPoint(x: 41, y: 8), in: size))
        XCTAssertEqual(display.word(at: CGPoint(x: 49, y: 8), in: size)?.text, "!")
        XCTAssertNil(display.word(at: CGPoint(x: 65, y: 8), in: size))
    }

    func testTextSizeChangesReportedGridAndFittedCellTogether() {
        let size = CGSize(width: 320, height: 340)
        let regular = TerminalMetrics(textSize: 13)
        let large = TerminalMetrics(textSize: 20)
        let regularGrid = regular.viewport(size: size, scale: 2)
        let largeGrid = large.viewport(size: size, scale: 2)
        XCTAssertEqual(regularGrid.columns, 40)
        XCTAssertEqual(regularGrid.rows, 20)
        XCTAssertLessThan(largeGrid.columns, regularGrid.columns)
        XCTAssertLessThan(largeGrid.rows, regularGrid.rows)
        XCTAssertEqual(large.cell.width * 2, CGFloat(largeGrid.cellWidth))
        XCTAssertEqual(large.cell.height * 2, CGFloat(largeGrid.cellHeight))
        XCTAssertEqual(large.fittedCell(in: size, columns: 80, rows: 40), CGSize(width: 4, height: 8.5))
        XCTAssertEqual(TerminalMetrics(textSize: -100).pointSize, CGFloat(TerminalPreferences.textRange.lowerBound))
    }
}
