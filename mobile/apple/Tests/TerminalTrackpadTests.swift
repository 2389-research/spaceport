// ABOUTME: Checks cursor travel for a held terminal or system keyboard space bar.
// ABOUTME: Covers small motions, direction changes and bounded key delivery.
import XCTest
#if os(iOS)
@testable import SpaceportMobile
#endif

final class TerminalTrackpadTests: XCTestCase {
    func testTravelAccumulatesAndChangesDirection() {
        var pad = TerminalTrackpad(origin: .zero)
        XCTAssertEqual(pad.move(to: CGPoint(x: 5, y: 0)), [])
        XCTAssertFalse(pad.moved)
        XCTAssertEqual(pad.move(to: CGPoint(x: 25, y: 2)), ["Right", "Right"])
        XCTAssertTrue(pad.moved)
        XCTAssertEqual(pad.move(to: CGPoint(x: 25, y: 2)), [])
        XCTAssertEqual(pad.move(to: CGPoint(x: -5, y: 2)), ["Left", "Left"])
        XCTAssertEqual(pad.move(to: CGPoint(x: -5, y: -21)), ["Up"])
    }

    func testLargeJumpDropsExcessKeys() {
        var pad = TerminalTrackpad(origin: .zero)
        XCTAssertEqual(pad.move(to: CGPoint(x: 0, y: 3000)), Array(repeating: "Down", count: 8))
        XCTAssertEqual(pad.move(to: CGPoint(x: 0, y: 3000)), [])
    }
}
