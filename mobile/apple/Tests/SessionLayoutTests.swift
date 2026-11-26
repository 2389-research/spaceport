// ABOUTME: Checks sidebar space against the current session window and terminal width.
// ABOUTME: Covers both Duo displays, narrow Split View, and a collapsed sidebar.
import XCTest
@testable import SpaceportMobile

final class SessionLayoutTests: XCTestCase {
    func testOnlyOffersSidebarWhenBothColumnsFitInARegularWindow() {
        XCTAssertNil(SessionLayout.sidebarWidth(in: 390, regular: false))
        XCTAssertNil(SessionLayout.sidebarWidth(in: 700, regular: false))
        XCTAssertNil(SessionLayout.sidebarWidth(in: 580, regular: true))
        XCTAssertEqual(SessionLayout.sidebarWidth(in: 669, regular: true), 248)
        XCTAssertEqual(SessionLayout.sidebarWidth(in: 951, regular: true), 280)
    }

    func testCompactKeyMenuKeepsConfiguredOrderAndShowsEveryKeyOnce() throws {
        let pages = try XCTUnwrap(TerminalKeyPages(rawValue: "esc,left,alt|left,tab,enter"))
        XCTAssertEqual(pages.compactMenuKeys, [.left, .alt, .tab, .enter])
    }

    func testToolbarFitsFullKeysThenPagesBeforeUsingTheCompactMenu() {
        XCTAssertEqual(SessionLayout.keyRow(width: 200, keyCount: 6), .compact)
        XCTAssertEqual(SessionLayout.keyRow(width: 320, keyCount: 6), .paged)
        XCTAssertEqual(SessionLayout.keyRow(width: 350, keyCount: 6), .flat)
        XCTAssertEqual(SessionLayout.keyRow(width: 400, keyCount: 12), .paged)
        XCTAssertEqual(SessionLayout.keyRow(width: 595, keyCount: 12), .paged)
        XCTAssertEqual(SessionLayout.keyRow(width: 596, keyCount: 12), .flat)
    }

    func testTypingAndComposingOnBothDisplays() {
        for (width, regular, keyboard, composing, collapsed, expected): (CGFloat, Bool, Bool, Bool, Bool, SessionLayout.Sidebar) in [
            (840, true, false, false, false, .full(252)),
            (840, true, true, false, false, .rail),
            (840, true, false, false, true, .rail),
            (840, true, true, true, false, .hidden),
            (390, false, true, false, false, .hidden)
        ] {
            XCTAssertEqual(SessionLayout.sidebar(width: width, regular: regular, keyboard: keyboard,
                                                  composing: composing, userCollapsed: collapsed), expected)
        }
    }

    func testSidebarMotionRespectsReduceMotionAndCompactWindows() {
        let full = SessionLayout.Sidebar.full(248)
        let rail = SessionLayout.Sidebar.rail
        XCTAssertNotNil(full.transitionAnimation(reduceMotion: false))
        XCTAssertEqual(full.transitionAnimation(reduceMotion: false), rail.transitionAnimation(reduceMotion: false))
        XCTAssertNil(full.transitionAnimation(reduceMotion: true))
        XCTAssertNil(rail.transitionAnimation(reduceMotion: true))
        XCTAssertNil(SessionLayout.Sidebar.hidden.transitionAnimation(reduceMotion: false))
    }

    func testWaitingInterruptFollowsNewTransitionsAndHonorsLater() {
        var waiting = WaitingInterrupt()
        func update(_ events: [String: UInt64], typing: Bool = true, boot: String = "a") {
            waiting.update(boot: boot, waiting: events, focused: "here", typing: typing)
        }
        update(["already": 1])
        XCTAssertNil(waiting.pane)
        update(["already": 1, "here": 2, "new": 3])
        XCTAssertEqual(waiting.pane, "new")
        waiting.dismiss()
        update(["already": 1, "here": 2, "new": 3])
        XCTAssertNil(waiting.pane)
        update(["first": 4, "second": 5])
        XCTAssertEqual(waiting.pane, "second")
        update(["first": 4])
        XCTAssertNil(waiting.pane)
        update(["offline": 9], typing: false)
        update(["offline": 9])
        XCTAssertNil(waiting.pane)
        update(["otherBoot": 10], boot: "b")
        XCTAssertNil(waiting.pane)
    }
}
