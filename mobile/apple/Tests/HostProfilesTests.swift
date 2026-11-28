// ABOUTME: Exercises host profile storage using an isolated preference domain.
// ABOUTME: Checks that edits preserve identity and never mix SSH endpoints.
import Foundation
import XCTest
@testable import SpaceportMobile

@MainActor
final class HostProfilesTests: XCTestCase {
    func testFilteringTagsPreservesSavedSessionIdentityAcrossReload() throws {
        let suite = "spaceport.tags.\(UUID())"
        let defaults = try XCTUnwrap(UserDefaults(suiteName: suite))
        defer { defaults.removePersistentDomain(forName: suite) }
        let store = ProfileStore(defaults: defaults)
        let a = try HostProfile.make(hostname: "one", port: "22", username: "u", session: "default")
        let b = try HostProfile.make(hostname: "two", port: "22", username: "u", session: "default")
        try store.save(a)
        try store.save(b)
        let id = try XCTUnwrap(store.catalog().hosts.first { $0.address == a.address }?.id)
        try store.tagHost(id, text: "Work, work, home")
        let loaded = try ProfileStore(defaults: defaults).catalog()
        XCTAssertEqual(loaded.filtered(by: "WORK").profiles, [a])
        XCTAssertEqual(loaded.hosts.first { $0.id == id }?.tags, ["Work", "home"])
        try store.tagHost(id, text: "")
        XCTAssertEqual(Set(try store.catalog().filtered(by: "Work").profiles), Set([a, b]))
    }

    func testSavedHostHeadingShowsAnAccountOnlyToDistinguishLogins() throws {
        let address = try SSHAddress.parse("pilot@box")
        let host = SavedHost(id: UUID(), address: address, login: .password)
        let other = SavedHost(id: UUID(), address: try SSHAddress.parse("deploy@box"), login: .password)
        XCTAssertEqual(host.listHeading(in: [host]), "box")
        XCTAssertEqual(host.listHeading(in: [host, other]), "box · Account pilot")
        XCTAssertEqual(other.listHeading(in: [host, other]), "box · Account deploy")
    }

    func testEditingSavedHostReplacesOriginalAndSurvivesRelaunch() throws {
        let suite = "spaceport.mobile.tests.\(UUID())"
        let defaults = try XCTUnwrap(UserDefaults(suiteName: suite))
        defer { defaults.removePersistentDomain(forName: suite) }
        let first = try HostProfile.make(hostname: " example.org ", port: "22", username: "biz", session: "main")
        let edited = try HostProfile.make(id: first.id, hostname: "example.org", port: "2222", username: "biz", session: "lab")
        let store = ProfileStore(defaults: defaults)
        try store.save(first)
        try store.save(edited)
        XCTAssertEqual(try ProfileStore(defaults: defaults).load(), [edited])
        XCTAssertNotEqual(first.pinAccount, edited.pinAccount)
        XCTAssertEqual(first.hostname, "example.org")
        try store.remove(edited)
        XCTAssertEqual(try store.load(), [])
    }

    func testChangingHostLoginUpdatesEverySessionAfterRelaunch() throws {
        let suite = "spaceport.mobile.tests.\(UUID())"
        let defaults = try XCTUnwrap(UserDefaults(suiteName: suite))
        defer { defaults.removePersistentDomain(forName: suite) }
        let store = ProfileStore(defaults: defaults)
        var main = try HostProfile.make(hostname: "host", port: "22", username: "biz", session: "main", login: .password)
        let lab = try HostProfile.make(hostname: "host", port: "22", username: "biz", session: "lab", login: .password)
        try store.save(main)
        try store.save(lab)
        main.login = .phoneKey
        try store.save(main)
        let profiles = try ProfileStore(defaults: defaults).load()
        XCTAssertEqual(profiles.map(\.id), [main.id, lab.id])
        XCTAssertEqual(profiles.map(\.login), [.phoneKey, .phoneKey])
    }

    func testLegacyMigrationPreservesSessionIDsAndPinsWhenGroupingHosts() throws {
        let suite = "spaceport.mobile.tests.\(UUID())"
        let defaults = try XCTUnwrap(UserDefaults(suiteName: suite))
        defer { defaults.removePersistentDomain(forName: suite) }
        let first = try HostProfile.make(hostname: "EXAMPLE.org", port: "22", username: "biz", session: "main")
        let second = try HostProfile.make(hostname: "EXAMPLE.org", port: "22", username: "biz", session: "lab")
        var alternate = first
        alternate.id = UUID()
        alternate.login = .password
        let legacy = [first, alternate, second]
        defaults.set(try JSONEncoder().encode(legacy), forKey: "sshSessionProfiles")
        let store = ProfileStore(defaults: defaults)
        let catalog = try store.catalog()
        XCTAssertEqual(catalog.hosts.count, 2)
        XCTAssertEqual(catalog.sessions.map(\.id), legacy.map(\.id))
        XCTAssertEqual(catalog.sessions[0].hostID, catalog.sessions[2].hostID)
        XCTAssertNotEqual(catalog.sessions[0].hostID, catalog.sessions[1].hostID)
        XCTAssertEqual(catalog.profiles, legacy)
        XCTAssertNil(defaults.object(forKey: "sshSessionProfiles"))
        try store.removeHost(catalog.hosts[0].id)
        XCTAssertEqual(try ProfileStore(defaults: defaults).load(), [alternate])
    }

    func testAddressParsingAndEffectiveDefaults() throws {
        let address = try SSHAddress.parse(" pilot@[::1]:23892 ")
        XCTAssertEqual(address.hostname, "::1")
        XCTAssertEqual(address.port, 23892)
        XCTAssertEqual(address.destination, "pilot@[::1]:23892")
        XCTAssertEqual(address.endpoint, "[::1]:23892")
        XCTAssertEqual(try SSHAddress.parse("pilot@box").endpoint, "box")
        XCTAssertEqual(try SSHAddress.parse("pilot@box").port, 22)
        for invalid in ["host", "a@b:", "a@b:65536", "a@[::1", "a@host/path", "a@b@c", "a@host:0"] {
            XCTAssertThrowsError(try SSHAddress.parse(invalid), invalid)
        }
    }

    func testLastSeenSurvivesRelaunchAndNeverMovesToChangedDestination() throws {
        let suite = "spaceport.mobile.tests.\(UUID())"
        let defaults = try XCTUnwrap(UserDefaults(suiteName: suite))
        defer { defaults.removePersistentDomain(forName: suite) }
        let store = ProfileStore(defaults: defaults)
        let main = try HostProfile.make(hostname: "host", port: "22", username: "biz", session: "main")
        try store.save(main)
        let snapshot = SessionStatusSnapshot(seenAt: Date(timeIntervalSince1970: 1234),
                                            blocked: 1, working: 2, hasIdle: true,
                                            location: ["Tools", "Build", "Shell"])
        try store.recordSeen(profile: main, snapshot: snapshot)
        XCTAssertEqual(try ProfileStore(defaults: defaults).catalog().sessions.first?.lastStatus, snapshot)
        var other = main
        other.hostname = "other"
        try store.save(other)
        try store.recordSeen(profile: main, snapshot: snapshot)
        XCTAssertNil(try store.catalog().sessions.first?.lastStatus)
    }

    func testEnrollmentPreservesBothNamedGroupsAndTheirHistoriesWhenLoginCollides() throws {
        let suite = "spaceport.mobile.tests.\(UUID())"
        let defaults = try XCTUnwrap(UserDefaults(suiteName: suite))
        defer { defaults.removePersistentDomain(forName: suite) }
        let store = ProfileStore(defaults: defaults)
        let password = try HostProfile.make(hostname: "box", port: "22", username: "me", session: "main", login: .password)
        let phoneKey = try HostProfile.make(hostname: "box", port: "22", username: "me", session: "main")
        try store.save(password)
        try store.save(phoneKey)
        for var host in try store.catalog().hosts {
            host.name = host.login == .password ? "Operations" : "Development"
            try store.saveHost(host)
        }
        for (index, profile) in [password, phoneKey].enumerated() {
            let snapshot = SessionStatusSnapshot(seenAt: Date(timeIntervalSince1970: Double(index + 1)),
                                                 blocked: index, working: index + 1, hasIdle: index == 0,
                                                 location: ["Workspace \(index)", "Tab", "Pane"])
            try store.recordSeen(profile: profile, snapshot: snapshot)
        }
        let before = try store.catalog()
        let enrolled = try XCTUnwrap(before.hosts.first(where: { $0.login == .password }))
        XCTAssertFalse(try store.usePhoneKey(for: enrolled))
        let after = try ProfileStore(defaults: defaults).catalog()
        XCTAssertEqual(after.hosts, before.hosts)
        XCTAssertEqual(after.sessions, before.sessions)
        XCTAssertEqual(after.profiles, before.profiles)
    }

    func testHostSurvivesRemovingItsLastSessionAndAcceptsAnother() throws {
        let suite = "spaceport.mobile.tests.\(UUID())"
        let defaults = try XCTUnwrap(UserDefaults(suiteName: suite))
        defer { defaults.removePersistentDomain(forName: suite) }
        let store = ProfileStore(defaults: defaults)
        let first = try HostProfile.make(hostname: "host", port: "22", username: "me", session: "main", login: .password)
        try store.save(first)
        let host = try XCTUnwrap(store.catalog().hosts.first)
        try store.remove(first)
        XCTAssertEqual(try store.catalog().hosts, [host])
        XCTAssertTrue(try store.usePhoneKey(for: host))
        XCTAssertTrue(try store.usePhoneKey(for: host))
        try store.saveSession(on: host.id, id: nil, name: "new")
        XCTAssertEqual(try store.load().map(\.session), ["new"])
        XCTAssertEqual(try store.load().map(\.login), [.phoneKey])
        XCTAssertThrowsError(try store.saveSession(on: host.id, id: nil, name: "new"))
        try store.removeHost(host.id)
        XCTAssertThrowsError(try store.saveSession(on: host.id, id: nil, name: "gone"))
        XCTAssertThrowsError(try store.usePhoneKey(for: host))
    }

    func testOlderSnapshotDecodesWithoutInventingAgents() throws {
        let old = Data(#"{"seenAt":1234,"blocked":1,"working":0,"hasIdle":false,"location":["Build"]}"#.utf8)
        let decoded = try JSONDecoder().decode(SessionStatusSnapshot.self, from: old)
        XCTAssertEqual(decoded.blocked, 1)
        XCTAssertNil(decoded.agents)
    }

    func testAgentCacheLimitsCountAndKeepsTargetIdentifiersExact() {
        func agent(pane: String, name: String = "Agent") -> SeenAgent {
            SeenAgent(bootId: "boot", workspaceId: "work", tabId: "tab", paneId: pane,
                      name: name, status: "working", location: "Work › Tab › Pane")
        }
        let rows = [agent(pane: String(repeating: "x", count: 1024)), agent(pane: "x", name: String(repeating: "👩🏾‍🚀", count: 256))]
                   + (0 ... SeenAgent.cacheLimit + 4).map { agent(pane: String($0)) }
        let saved = SeenAgent.cache(rows)
        XCTAssertEqual(saved.map(\.paneId), (0 ..< SeenAgent.cacheLimit).map(String.init))
    }

    func testHomeAgentIdentityIncludesSessionAndDaemonBoot() throws {
        let suite = "spaceport.mobile.tests.\(UUID())"
        let defaults = try XCTUnwrap(UserDefaults(suiteName: suite))
        defer { defaults.removePersistentDomain(forName: suite) }
        let store = ProfileStore(defaults: defaults)
        let main = try HostProfile.make(hostname: "box", port: "22", username: "me", session: "main")
        let lab = try HostProfile.make(hostname: "box", port: "22", username: "me", session: "lab")
        let rows = ["boot-a", "boot-b"].map { boot in
            SeenAgent(bootId: boot, workspaceId: "w", tabId: "t", paneId: "p", name: "Builder",
                      status: "blocked", location: "Tools › Build › Agent")
        }
        let seen = Date(timeIntervalSince1970: 100)
        for profile in [main, lab] {
            try store.save(profile)
            try store.recordSeen(profile: profile, snapshot: SessionStatusSnapshot(
                seenAt: seen, blocked: 1, working: 0, hasIdle: false, location: [], agents: [rows[0]]))
        }
        let first = try store.catalog().observedAgents
        XCTAssertEqual(first.map(\.profile.id), [main.id, lab.id])
        XCTAssertEqual(Set(first.map(\.id)).count, 2)
        try store.recordSeen(profile: main, snapshot: SessionStatusSnapshot(
            seenAt: seen.addingTimeInterval(1), blocked: 1, working: 0, hasIdle: false, location: [], agents: [rows[1]]))
        let next = try store.catalog().observedAgents
        XCTAssertNotEqual(next.first?.id, first.first?.id)
        XCTAssertEqual(next.first?.agent, rows[1])
        XCTAssertEqual(next.last?.id, first.last?.id)
    }

    func testLastSeenCapturesReportedAgentsAcrossWorkspacesWithoutPlainTerminalRows() {
        let spaces = [MobileWorkspace(id: "one", label: "Dev", status: "working"),
                      MobileWorkspace(id: "two", label: "Ops", status: "blocked")]
        let tabs = [MobileTab(id: "tab1", workspaceId: "one", label: "Build", status: "working"),
                    MobileTab(id: "tab2", workspaceId: "two", label: "Watch", status: "blocked")]
        let panes = [MobilePane(stateChangeSeq: nil, agentName: "Builder", id: "phone", tabId: "tab1", label: "Compile", status: "working"),
                     MobilePane(stateChangeSeq: nil, agentName: nil, id: "shell", tabId: "tab1", label: "Shell", status: "idle"),
                     MobilePane(stateChangeSeq: nil, agentName: "Reviewer", id: "review", tabId: "tab2", label: "Tests", status: "blocked", terminalTitle: "Verify the release")]
        let view = MobileView(version: 4, bootId: "boot", workspaces: spaces, tabs: tabs, panes: panes,
                              paneMap: nil, focusedWorkspace: "one", focusedTab: "tab1", focusedPane: "shell",
                              terminal: nil, inputTarget: nil, error: nil, disconnected: false,
                              hostChallenge: nil, goToPending: false, goToArrival: nil)
        let seen = Date(timeIntervalSince1970: 918)
        let snapshot = SessionStatusSnapshot(view: view, seenAt: seen)
        XCTAssertEqual(snapshot.seenAt, seen)
        XCTAssertEqual(snapshot.location, ["Dev", "Build", "Shell"])
        XCTAssertEqual(snapshot.agents, [
            SeenAgent(bootId: "boot", workspaceId: "two", tabId: "tab2", paneId: "review", name: "Reviewer",
                      status: "blocked", location: "Ops › Watch", changeObservedAt: seen, terminalTitle: "Verify the release",
                      workspaceLabelBytes: "Ops".utf8.count, paneTitle: "Tests"),
            SeenAgent(bootId: "boot", workspaceId: "one", tabId: "tab1", paneId: "phone", name: "Builder",
                      status: "working", location: "Dev › Build", changeObservedAt: seen, workspaceLabelBytes: "Dev".utf8.count, paneTitle: "Compile")
        ])
    }

    func testIdleOrderAcrossHostsSurvivesRefreshingAnUnchangedSession() throws {
        let suite = "spaceport.mobile.tests.\(UUID())"
        let defaults = try XCTUnwrap(UserDefaults(suiteName: suite))
        defer { defaults.removePersistentDomain(forName: suite) }
        let store = ProfileStore(defaults: defaults)
        func target(_ hostname: String) throws -> HostProfile {
            let p = try HostProfile.make(hostname: hostname, port: "22", username: "me", session: "main")
            try store.save(p)
            return p
        }
        func observe(_ profile: HostProfile, seq: UInt64, at second: TimeInterval) throws {
            let date = Date(timeIntervalSince1970: second)
            let row = SeenAgent(bootId: "boot-" + profile.hostname, workspaceId: "w", tabId: "t", paneId: "p",
                                name: profile.hostname, status: "idle", location: "w › t",
                                stateChangeSeq: seq, changeObservedAt: date)
            try store.recordSeen(profile: profile, snapshot: SessionStatusSnapshot(
                seenAt: date, blocked: 0, working: 0, hasIdle: true, location: [], agents: [row]))
        }
        let a = try target("alpha"), b = try target("beta")
        try observe(a, seq: 900, at: 10)
        try observe(b, seq: 2, at: 20)
        try observe(a, seq: 900, at: 30)
        XCTAssertEqual(try store.catalog().observedAgents.map(\.agent.name), ["beta", "alpha"])
        try observe(a, seq: 903, at: 40)
        XCTAssertEqual(try store.catalog().observedAgents.map(\.agent.name), ["alpha", "beta"])
    }

    func testPinBelongsToEndpointAcrossUsersAndSessions() throws {
        let a = try HostProfile.make(hostname: "EXAMPLE.org", port: "2222", username: "one", session: "lab")
        let b = try HostProfile.make(hostname: "example.org", port: "2222", username: "two", session: "default")
        XCTAssertEqual(a.pinAccount, b.pinAccount)
    }

    func testBracketedIPv6AddressSharesTrustWithUnbracketedAddress() throws {
        let bracketed = try HostProfile.make(hostname: "[::1]", port: "23892", username: "pilot", session: "main")
        let bare = try HostProfile.make(hostname: "::1", port: "23892", username: "pilot", session: "main")
        XCTAssertEqual(bracketed.hostname, "::1")
        XCTAssertEqual(bracketed.pinAccount, bare.pinAccount)
        XCTAssertEqual(bracketed.address.destination, "pilot@[::1]:23892")
    }

    func testRejectsMissingTargetAndInvalidPort() {
        for port in ["", "0", "65536", "-22"] {
            XCTAssertThrowsError(try HostProfile.make(hostname: "example.org", port: port, username: "me", session: "main"))
        }
        XCTAssertThrowsError(try HostProfile.make(hostname: "example.org", port: "22", username: " ", session: "main"))
        XCTAssertThrowsError(try HostProfile.make(hostname: "example.org", port: "22", username: "me", session: " "))
    }

    func testCorruptLegacyCatalogSurvivesMigrationFailure() throws {
        let suite = "spaceport.mobile.tests.\(UUID())"
        let defaults = try XCTUnwrap(UserDefaults(suiteName: suite))
        defer { defaults.removePersistentDomain(forName: suite) }
        defaults.set("broken data", forKey: "sshSessionProfiles")
        let store = ProfileStore(defaults: defaults)
        XCTAssertThrowsError(try store.catalog())
        XCTAssertEqual(defaults.string(forKey: "sshSessionProfiles"), "broken data")
        XCTAssertNil(defaults.object(forKey: ProfileStore.catalogKey))
    }

    func testCorruptCatalogCannotBeOverwritten() throws {
        let suite = "spaceport.mobile.tests.\(UUID())"
        let defaults = try XCTUnwrap(UserDefaults(suiteName: suite))
        defer { defaults.removePersistentDomain(forName: suite) }
        defaults.set(Data([0xff]), forKey: ProfileStore.catalogKey)
        let profile = try HostProfile.make(hostname: "example.org", port: "22", username: "me", session: "main")
        XCTAssertThrowsError(try ProfileStore(defaults: defaults).save(profile))
        XCTAssertEqual(defaults.data(forKey: ProfileStore.catalogKey), Data([0xff]))
    }
}
