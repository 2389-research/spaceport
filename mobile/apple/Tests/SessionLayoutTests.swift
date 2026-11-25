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

    func testCompactTerminalPrioritizesEscapeButWideTerminalsKeepTheirKeyPages() {
        XCTAssertTrue(SessionLayout.usesCompactKeys(width: 320))
        XCTAssertFalse(SessionLayout.usesCompactKeys(width: 390))
    }
}
