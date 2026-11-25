// ABOUTME: Exercises Herdr popup cell replay, including the rendered native Ghostty grid.
// ABOUTME: Checks control isolation, cell clipping, wide characters, colors and cursor placement.
import GhosttyTerminal
import SwiftUI
import XCTest
@testable import SpaceportMobile

final class TerminalSnapshotTests: XCTestCase {
    private func cell(_ text: String, fg: UInt32 = 0, bg: UInt32 = 0, flags: UInt16 = 0, skip: Bool = false) -> MobileCell {
        MobileCell(symbol: text, fg: fg, bg: bg, modifier: flags, skip: skip)
    }

    func testPopupReplayOnlyInterpretsLocalStylesAndPositions() {
        let cursor = "\u{1b}[2;1H\u{1b}[?25h"
        let unsafe = ["\u{1b}]52;c;AAAA\u{7}", "\u{9b}2J", "\n", "\u{202e}", "abc"]
        for symbol in unsafe {
            let frame = MobileTerminal(width: 2, height: 2,
                                       cells: [cell(symbol), cell("!"), cell("X", fg: 0x02010203, bg: 0x01000004,
                                                                           flags: 1 | 8), cell("x", flags: 128)],
                                       cursorX: 0, cursorY: 1)
            let replay = String(decoding: frame.replay(columns: 1, rows: 2), as: UTF8.self)
            XCTAssertEqual(frame.visibleText, "�!\nX")
            XCTAssertTrue(replay.contains("\u{1b}[1;1H�"))
            XCTAssertTrue(replay.contains("\u{1b}[0;1;4;38;2;1;2;3;48;5;4m\u{1b}[2;1HX"))
            XCTAssertTrue(replay.hasSuffix(cursor))
            XCTAssertFalse(replay.contains(symbol))
            XCTAssertFalse(replay.contains("!"))
            XCTAssertFalse(String(decoding: frame.replay(columns: 1, rows: 1), as: UTF8.self).hasSuffix(cursor))
        }
    }

    @MainActor
    func testPopupReachesTheMountedGhosttyGrid() async throws {
        let frame = MobileTerminal(width: 5, height: 2,
                                   cells: [cell("猫"), cell("", bg: 0x02000000), cell("K"), cell("", bg: 0x02000000), cell("!"),
                                           cell("Y"), cell("e"), cell("s"), cell(" "), cell("X", flags: 128)],
                                   cursorX: 2, cursorY: 1)
        let host = UIHostingController(rootView: TerminalDisplay(terminal: frame, metrics: TerminalMetrics(textSize: 12)))
        let scene = try XCTUnwrap(UIApplication.shared.connectedScenes.compactMap { $0 as? UIWindowScene }.first)
        let window = UIWindow(windowScene: scene)
        window.rootViewController = host
        window.makeKeyAndVisible()
        defer { window.isHidden = true }
        host.view.layoutIfNeeded()
        func findTerminal(_ view: UIView) -> UITerminalView? {
            (view as? UITerminalView) ?? view.subviews.lazy.compactMap(findTerminal).first
        }
        let view = try XCTUnwrap(findTerminal(host.view))
        guard case let .inMemory(session) = view.configuration.backend else {
            XCTFail("The popup requires an isolated in-memory session.")
            return
        }
        let replay = expectation(description: "Ghostty receives the full popup")
        replay.assertForOverFulfill = false
        session.setOutputBacklogHandler(highWater: 1, lowWater: 0) { queued in
            if queued { replay.fulfill() }
        }
        await fulfillment(of: [replay], timeout: 5)
        session.setOutputBacklogHandler(highWater: 1, lowWater: 0, nil)
        XCTAssertTrue(session.waitForPendingOutput())
        let rows = try XCTUnwrap(session.readViewportText()).split(separator: "\n").map { $0.trimmingCharacters(in: .whitespaces) }
        XCTAssertEqual(Array(rows.prefix(2)), ["猫K !", "Yes"])
    }

    @MainActor
    func testWideContinuationSharesItsLeadGlyphForReadingAndWordSelection() {
        for symbol in ["こ", "猫", "ｶﾞ", "🇺🇸", "❤️"] {
            let frame = MobileTerminal(width: 5, height: 1,
                                       cells: [cell(symbol), cell("", bg: 0x02000000), cell("x"),
                                               cell("", bg: 0x02123456), cell("y")],
                                       cursorX: nil, cursorY: nil)
            XCTAssertEqual(frame.visibleText, symbol + "x y")
            let replay = String(decoding: frame.replay(columns: 5, rows: 1), as: UTF8.self)
            XCTAssertFalse(replay.contains("\u{1b}[1;2H "))
            XCTAssertTrue(replay.contains("\u{1b}[1;4H "))
            let display = TerminalDisplay(terminal: frame, metrics: TerminalMetrics(textSize: 13))
            let size = CGSize(width: 40, height: 17)
            XCTAssertEqual(display.word(at: CGPoint(x: 9, y: 1), in: size)?.text, symbol + "x")
            XCTAssertNil(display.word(at: CGPoint(x: 25, y: 1), in: size))
        }
    }
}
