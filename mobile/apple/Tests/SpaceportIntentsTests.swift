// ABOUTME: Exercises the device authentication and foreground contracts of Spaceport's shortcuts.
// ABOUTME: Checks that a status reply reads the selected agent's latest saved observation.
import AppIntents
import XCTest
@testable import SpaceportMobile

@MainActor
final class SpaceportIntentsTests: XCTestCase {
    func testNavigationOpensTheAppAndEveryActionRequiresDeviceAuthentication() {
        for action: any AppIntent.Type in [
            ResumeSessionIntent.self, NextWaitingAgentIntent.self, OpenAgentIntent.self, ComposePromptIntent.self,
        ] {
            XCTAssertTrue(action.openAppWhenRun)
            XCTAssertEqual(action.authenticationPolicy, .requiresLocalDeviceAuthentication)
        }
        XCTAssertFalse(GetAgentStatusIntent.openAppWhenRun)
        XCTAssertEqual(GetAgentStatusIntent.authenticationPolicy, .requiresLocalDeviceAuthentication)
    }

    func testForegroundActionsDeliverTheChosenTargetsAndUnmodifiedDraft() async throws {
        let profile = try HostProfile.make(hostname: "example.test", port: "22", username: "pilot", session: "lab")
        var catalog = HostCatalog()
        catalog.append(profile)
        let seen = SeenAgent(bootId: "boot", workspaceId: "w", tabId: "t", paneId: "p", name: "Codex",
                             status: "working", location: "lab › work")
        catalog.sessions[0].lastStatus = .init(seenAt: Date(), blocked: 0, working: 1, hasIdle: false,
                                               location: [], agents: [seen])
        let session = try XCTUnwrap(catalog.systemSessions.first)
        let agent = try XCTUnwrap(catalog.systemAgents.first)
        let scene = UUID()
        var received: [SpaceportShortcut] = []
        try SpaceportShortcutRouter.shared.activate(scene) { received.append($0) }
        defer { SpaceportShortcutRouter.shared.deactivate(scene) }

        let resume = ResumeSessionIntent()
        _ = try await resume.perform()
        resume.session = session
        _ = try await resume.perform()
        let waiting = NextWaitingAgentIntent()
        waiting.session = session
        _ = try await waiting.perform()
        let open = OpenAgentIntent()
        open.agent = agent
        _ = try await open.perform()
        let compose = ComposePromptIntent()
        compose.prompt = nil
        _ = try await compose.perform()
        compose.session = session
        compose.prompt = "  Check the moon.\n"
        _ = try await compose.perform()
        XCTAssertEqual(received, [.resume(session: nil), .resume(session: session.id), .nextWaiting(session: session.id),
                                  .openAgent(id: agent.id), .compose(session: nil, prompt: ""),
                                  .compose(session: session.id, prompt: "  Check the moon.\n")])
    }

    func testStatusUsesCurrentObservationAndRefusesARemovedAgent() throws {
        let suite = "spaceport.mobile.tests.\(UUID())"
        let defaults = try XCTUnwrap(UserDefaults(suiteName: suite))
        defer { defaults.removePersistentDomain(forName: suite) }
        let store = ProfileStore(defaults: defaults)
        let profile = try HostProfile.make(hostname: "example.test", port: "22", username: "pilot", session: "lab")
        try store.save(profile)
        let selectedAt = Date(timeIntervalSince1970: 100)
        let updatedAt = selectedAt.addingTimeInterval(86_400)
        func record(status: String, at date: Date) throws {
            let agent = SeenAgent(bootId: "boot", workspaceId: "w", tabId: "t", paneId: "p", name: "Codex",
                                  status: status, location: "lab › work")
            try store.recordSeen(profile: profile, snapshot: .init(seenAt: date, blocked: 0, working: 0,
                                                                   hasIdle: false, location: [], agents: [agent]))
        }
        try record(status: "working", at: selectedAt)
        let action = GetAgentStatusIntent()
        action.agent = try XCTUnwrap(store.catalog().systemAgents.first)
        try record(status: "blocked", at: updatedAt)

        let report = try action.statusReport(from: store)
        XCTAssertTrue(report.contains("Codex"))
        XCTAssertTrue(report.contains("Needs you"))
        XCTAssertTrue(report.contains(updatedAt.formatted(date: .abbreviated, time: .shortened)))
        XCTAssertFalse(report.contains("Working"))

        try store.remove(profile)
        XCTAssertThrowsError(try action.statusReport(from: store)) { error in
            guard case ProfileError.missingAgent = error else { return XCTFail("Unexpected error: \(error)") }
        }
    }
}
