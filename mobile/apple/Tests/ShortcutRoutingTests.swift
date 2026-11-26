// ABOUTME: Verifies a system action reaches exactly one foreground window.
// ABOUTME: Checks that a missing explicit session cannot fall through to another host.
import XCTest
@testable import SpaceportMobile

@MainActor
final class ShortcutRoutingTests: XCTestCase {
    func testColdLaunchKeepsOneRequestUntilAWindowActivates() throws {
        let router = SpaceportShortcutRouter()
        let first = UUID(), second = UUID()
        var delivered: [SpaceportShortcut] = []
        let request = SpaceportShortcut.compose(session: nil, prompt: "a draft")
        try router.submit(request)
        XCTAssertThrowsError(try router.submit(.resume(session: nil)))
        try router.activate(first) { delivered.append($0) }
        try router.activate(second) { _ in XCTFail("Cold request replayed") }
        XCTAssertEqual(delivered, [request])
    }

    func testMostRecentlyActiveWindowReceivesEachActionOnce() throws {
        let router = SpaceportShortcutRouter()
        let first = UUID(), second = UUID()
        var windows: [UUID] = []
        try router.activate(first) { _ in windows.append(first) }
        try router.activate(second) { _ in windows.append(second) }
        try router.submit(.nextWaiting(session: nil))
        router.deactivate(second)
        try router.submit(.resume(session: nil))
        XCTAssertEqual(windows, [second, first])
    }

    func testRefusalReachesTheCallerOnWarmAndColdLaunch() throws {
        let router = SpaceportShortcutRouter()
        let id = UUID()
        try router.submit(.resume(session: nil))
        XCTAssertThrowsError(try router.activate(id) { _ in throw ProfileError.missingHost })
        XCTAssertThrowsError(try router.submit(.resume(session: nil)))
    }

    func testSecondShortcutCannotDropAPromptWaitingForSignIn() throws {
        var catalog = HostCatalog()
        for name in ["first", "second"] {
            catalog.append(try .make(hostname: "test.invalid", port: "22", username: "pilot",
                                     session: name, login: .password))
        }
        let sessions = SceneSessions()
        // Keep the first session on its credential screen without starting SSH.
        let profile = catalog.profiles[0]
        let connection = sessions.connection(for: profile)
        let first = try sessions.openShortcut(.compose(session: profile.id, prompt: "save me"), in: catalog)
        defer { sessions.disconnect(profile) }
        XCTAssertThrowsError(try sessions.openShortcut(.resume(session: catalog.profiles[1].id), in: catalog))
        XCTAssertEqual(sessions.current, first)
        guard case let .compose(text)? = connection.pendingShortcut?.action else { return XCTFail("Lost draft request") }
        XCTAssertEqual(text, "save me")
    }

    func testExplicitSessionMustStillExistAndForegroundBeatsLastOpened() throws {
        var catalog = HostCatalog()
        for name in ["first", "second"] {
            catalog.append(try .make(hostname: "hosts.invalid", port: "22", username: "pilot",
                                     session: name, login: .password))
        }
        let first = catalog.profiles[0], second = catalog.profiles[1]
        catalog.sessions[0].lastOpened = Date()
        XCTAssertEqual(try SpaceportShortcut.resume(session: nil).destination(in: catalog, active: second).profile, second)
        XCTAssertEqual(try SpaceportShortcut.resume(session: nil).destination(in: catalog, active: nil).profile, first)
        XCTAssertEqual(try SpaceportShortcut.nextWaiting(session: first.id).destination(in: catalog, active: second).profile, first)
        XCTAssertThrowsError(try SpaceportShortcut.compose(session: UUID(), prompt: "hello").destination(in: catalog, active: second))
        XCTAssertThrowsError(try SpaceportShortcut.openAgent(id: "missing").destination(in: catalog, active: second))
    }
}
