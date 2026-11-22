// ABOUTME: Covers the ordering of scene interruptions and established SSH loss.
// ABOUTME: Prevents retries while hidden and repeated dials from an unstable replacement.
import XCTest
@testable import SpaceportMobile

final class TransportRecoveryTests: XCTestCase {
    func testInactiveDropDefersOneAttemptUntilVisible() {
        var recovery = TransportRecovery()
        recovery.sceneInactive(established: true)
        XCTAssertEqual(recovery.loss(eligible: true, presented: true), .deferUntilActive)
        recovery.sceneInactive(established: false)
        XCTAssertTrue(recovery.sceneActive())
        XCTAssertFalse(recovery.sceneActive())
        XCTAssertEqual(recovery.loss(eligible: true, presented: true), .stop)
    }

    func testLateDropAfterHealthyProbeAndForegroundDropHaveOneAttempt() {
        var recovery = TransportRecovery()
        XCTAssertEqual(recovery.loss(eligible: true, presented: true), .reconnect)
        XCTAssertEqual(recovery.loss(eligible: true, presented: true), .stop)
        recovery.sceneInactive(established: true)
        XCTAssertFalse(recovery.sceneActive())
        XCTAssertEqual(recovery.loss(eligible: true, presented: true), .reconnect)
        XCTAssertEqual(recovery.loss(eligible: true, presented: true), .stop)
        recovery.allowRetry()
        XCTAssertEqual(recovery.loss(eligible: true, presented: true), .reconnect)
    }

    func testAnInterruptedFocusChangeRequiresAnExplicitSelection() {
        var recovery = TransportRecovery()
        XCTAssertEqual(recovery.loss(eligible: true, presented: true, focusPending: true), .stop)
        recovery.sceneInactive(established: true)
        XCTAssertEqual(recovery.loss(eligible: true, presented: true, focusPending: true), .stop)
        XCTAssertFalse(recovery.sceneActive())
    }

    func testHiddenAndNonTransportFailuresCannotRedial() {
        var recovery = TransportRecovery()
        recovery.sceneInactive(established: true)
        XCTAssertEqual(recovery.loss(eligible: true, presented: false), .stop)
        XCTAssertEqual(recovery.loss(eligible: false, presented: true), .stop)
        XCTAssertFalse(recovery.sceneActive())
        XCTAssertEqual(recovery.loss(eligible: false, presented: true), .stop)
    }
}
