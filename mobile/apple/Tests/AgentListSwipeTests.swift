// ABOUTME: Checks which single-finger drags can open the agent list.
// ABOUTME: Protects short movements, vertical scrolling and leftward drags.
import XCTest
@testable import SpaceportMobile

final class AgentListSwipeTests: XCTestCase {
    func testOnlyADeliberateRightwardSwipeOpensAgents() {
        XCTAssertTrue(AgentListSwipe.completes(CGPoint(x: 80, y: -20)))
        XCTAssertFalse(AgentListSwipe.completes(CGPoint(x: 16, y: 0)))
        XCTAssertFalse(AgentListSwipe.completes(CGPoint(x: -90, y: 0)))
        XCTAssertFalse(AgentListSwipe.completes(CGPoint(x: 65, y: 60)))
        XCTAssertFalse(AgentListSwipe.completes(CGPoint(x: 10, y: -100)))
    }
}
