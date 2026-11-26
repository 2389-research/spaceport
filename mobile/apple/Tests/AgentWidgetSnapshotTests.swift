// ABOUTME: Verifies the handoff between the app, the widget timeline, and a widget tap.
// ABOUTME: Checks duplicate updates, removal, and exact saved-session links.
import XCTest
@testable import SpaceportMobile

final class AgentWidgetSnapshotTests: XCTestCase {
    func testIndependentReadersSeeOneCompleteObservationAndThenItsRemoval() throws {
        let suite = "spaceport-widget-tests.\(UUID())"
        let app = try XCTUnwrap(UserDefaults(suiteName: suite))
        let widget = try XCTUnwrap(UserDefaults(suiteName: suite))
        defer { app.removePersistentDomain(forName: suite) }
        let sample = AgentWidgetSnapshot(destinationID: UUID(), counts: AgentCounts(waiting: 3, working: 2, idle: 40),
                                         observedAt: Date(timeIntervalSince1970: 1234))
        XCTAssertNil(AgentWidgetSnapshot.load(from: widget))
        XCTAssertTrue(AgentWidgetSnapshot.store(sample, in: app))
        XCTAssertEqual(AgentWidgetSnapshot.load(from: widget), sample)
        XCTAssertFalse(AgentWidgetSnapshot.store(sample, in: app))
        let zero = AgentWidgetSnapshot(destinationID: sample.destinationID, counts: AgentCounts(), observedAt: sample.observedAt)
        XCTAssertTrue(AgentWidgetSnapshot.store(zero, in: app))
        XCTAssertEqual(AgentWidgetSnapshot.load(from: widget), zero)
        XCTAssertTrue(AgentWidgetSnapshot.store(nil, in: app))
        XCTAssertNil(AgentWidgetSnapshot.load(from: widget))
        XCTAssertFalse(AgentWidgetSnapshot.store(nil, in: app))
    }

    func testTapIdentifiesOnlyTheDisplayedSavedSession() throws {
        let sample = AgentWidgetSnapshot(destinationID: UUID(), counts: AgentCounts(), observedAt: Date())
        let url = try XCTUnwrap(sample.destinationURL)
        XCTAssertEqual(AgentWidgetSnapshot.destinationID(from: url), sample.destinationID)
        for malformed in [url.absoluteString + "/extra", url.absoluteString + "?host=elsewhere",
                          "https://next-waiting/\(sample.destinationID)", "spaceport-widget://next-waiting/not-a-session"] {
            XCTAssertNil(AgentWidgetSnapshot.destinationID(from: try XCTUnwrap(URL(string: malformed))))
        }
    }
}
