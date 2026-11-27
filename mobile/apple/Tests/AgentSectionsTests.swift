// ABOUTME: Exercises saved agent groups against observations from multiple Herdr sessions.
// ABOUTME: Checks that cache writes and relaunches preserve idle age evidence.
import Foundation
import SwiftUI
import XCTest
@testable import SpaceportMobile

@MainActor
final class AgentSectionsTests: XCTestCase {
    func testOnlyActiveAgentDotsPulseWhenMotionIsEnabled() {
        XCTAssertEqual(PaneActivity.working.pulse(reduceMotion: false, scene: .active), .working)
        XCTAssertEqual(PaneActivity.blocked.pulse(reduceMotion: false, scene: .active), .waiting)
        XCTAssertNil(PaneActivity.idle.pulse(reduceMotion: false, scene: .active))
        for activity in [PaneActivity.working, .blocked, .idle] {
            XCTAssertNil(activity.pulse(reduceMotion: true, scene: .active))
            XCTAssertNil(activity.pulse(reduceMotion: false, scene: .background))
            XCTAssertNil(activity.pulse(reduceMotion: false, scene: .inactive))
        }
    }

    func testWorkingBreathesOnceAndWaitingPulsesTwiceBeforeResting() {
        func brightness(_ pulse: AgentPulse, at times: [Double]) -> [Double] {
            let timeline = KeyframeTimeline(initialValue: 1.0) { pulse.opacity }
            return times.map { timeline.value(time: $0) }
        }
        let working = brightness(.working, at: [0, 1.2, 2.4])
        XCTAssertLessThan(working[1], working[0])
        XCTAssertEqual(working[2], working[0], accuracy: 0.01)
        let waiting = brightness(.waiting, at: [0, 0.22, 0.44, 0.72, 0.94, 1.8])
        XCTAssertLessThan(waiting[1], waiting[0])
        XCTAssertGreaterThan(waiting[2], waiting[1])
        XCTAssertLessThan(waiting[3], waiting[2])
        XCTAssertEqual(waiting[4], waiting[0], accuracy: 0.01)
        XCTAssertEqual(waiting[5], waiting[4], accuracy: 0.01)
    }

    private func row(_ name: String, _ status: String, seq: UInt64) -> SeenAgent {
        SeenAgent(bootId: "boot", workspaceId: "w", tabId: "t", paneId: name,
                  name: name, status: status, location: "Build › Review", stateChangeSeq: seq)
    }

    private func catalog(_ rows: [SeenAgent], seenAt: TimeInterval) throws -> HostCatalog {
        let profile = try HostProfile.make(hostname: "host", port: "22", username: "biz", session: "main")
        var catalog = HostCatalog()
        catalog.append(profile)
        catalog.sessions[0].lastStatus = SessionStatusSnapshot(
            seenAt: Date(timeIntervalSince1970: seenAt), blocked: 1, working: 1, hasIdle: true,
            location: [], agents: rows)
        return catalog
    }

    func testHomePlacesWaitingFirstAndUsesDaemonOrderForUntimedIdle() throws {
        let rows = [row("Work", "working", seq: 3), row("Old", "idle", seq: 1),
                    row("Wait", "blocked", seq: 4), row("New", "done", seq: 6)]
        XCTAssertEqual(try catalog(rows, seenAt: 2_000).observedAgents.map(\.agent.name),
                       ["Wait", "Work", "New", "Old"])
    }

    func testAgeBandsFreezeAtTheSnapshotAndPlaceFirstSightingsLast() throws {
        let now: TimeInterval = 90_000
        func idle(_ name: String, seconds: TimeInterval, witnessed: Bool) -> SeenAgent {
            var agent = row(name, "idle", seq: 5)
            agent.changeObservedAt = Date(timeIntervalSince1970: now - seconds)
            agent.idleOnsetObserved = witnessed
            return agent
        }
        let input = [
            idle("Five", seconds: 18_000, witnessed: false),
            idle("New", seconds: 59, witnessed: true),
            idle("Hour", seconds: 3_600, witnessed: true),
            idle("Unplaced", seconds: 3_599, witnessed: false),
            idle("Recent", seconds: 3_599, witnessed: true),
            row("Unknown", "unknown", seq: 2)
        ]
        let rows = try catalog(input, seenAt: now).observedAgents
        XCTAssertEqual(rows.map(\.agent.name), ["New", "Recent", "Hour", "Five", "Unplaced", "Unknown"])
        XCTAssertEqual(rows.map(\.section), [.recentIdle, .recentIdle, .idleHours, .oldIdle, .idle, .other])
        XCTAssertEqual(rows.map(\.ageLabel), ["<1m", "59m", "1h", "5h+", nil, nil])
    }

    func testQuietUsesObservedDayAndFinishedWorkCanBeSettled() throws {
        var idle = row("quiet", "idle", seq: 1)
        idle.changeObservedAt = Date(timeIntervalSince1970: 10)
        idle.idleOnsetObserved = false
        let done = row("review", "done", seq: 2)
        let rows = try catalog([idle, done], seenAt: 86_410).observedAgents
        XCTAssertEqual(rows.map(\.section), [.finished, .quiet])
        var marks = AgentMarks()
        marks.apply(.settle(true), status: "done")
        XCTAssertTrue(marks.observed(status: "done", at: .now)?.settled == true)
        XCTAssertNil(marks.observed(status: "blocked", at: .now))
        XCTAssertNil(marks.observed(status: "working", at: .now))
    }

    func testReconnectConfirmsLowerBoundOnlyForTheSameIdleStreak() throws {
        let suite = "spaceport.mobile.idle.\(UUID())"
        let defaults = try XCTUnwrap(UserDefaults(suiteName: suite))
        defer { defaults.removePersistentDomain(forName: suite) }
        let profile = try HostProfile.make(hostname: "host", port: "22", username: "biz", session: "main")
        let store = ProfileStore(defaults: defaults)
        try store.save(profile)
        func observe(_ status: String, seq: UInt64, at time: TimeInterval, legacy: Bool = false) throws {
            var agent = row("Scout", status, seq: seq)
            let date = Date(timeIntervalSince1970: time)
            agent.changeObservedAt = legacy ? .distantPast : date
            agent.idleOnsetObserved = legacy ? nil : false
            try store.recordSeen(profile: profile, snapshot: SessionStatusSnapshot(
                seenAt: date, blocked: 0, working: 0, hasIdle: true, location: [], agents: [agent]))
        }
        func saved() throws -> HomeAgent {
            try XCTUnwrap(ProfileStore(defaults: defaults).catalog().observedAgents.first)
        }
        try observe("idle", seq: 7, at: 1_000, legacy: true)
        XCTAssertEqual(try saved().section, .idle)
        try observe("idle", seq: 7, at: 8_200)
        XCTAssertEqual(try saved().section, .idleHours)
        XCTAssertEqual(try saved().ageLabel, "2h+")
        try observe("idle", seq: 8, at: 8_300)
        XCTAssertEqual(try saved().section, .idle)
        XCTAssertNil(try saved().ageLabel)
    }
}
