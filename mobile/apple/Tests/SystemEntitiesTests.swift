// ABOUTME: Checks shortcut choices against the catalog used by the phone.
// ABOUTME: Verifies that a saved agent reference cannot select a replacement.
import XCTest
@testable import SpaceportMobile

final class SystemEntitiesTests: XCTestCase {
    private let host = SavedHost(id: UUID(), address: .init(hostname: "dev.example", port: 22, username: "test"),
                                 login: .phoneKey, name: "Dev")
    private let observed = Date(timeIntervalSince1970: 700)

    private func agent(_ boot: String = "boot", _ workspace: String = "w", _ tab: String = "t",
                       _ pane: String = "p", _ name: String = "Claude", _ status: String = "working") -> SeenAgent {
        SeenAgent(bootId: boot, workspaceId: workspace, tabId: tab, paneId: pane, name: name,
                  status: status, location: "space › tab")
    }

    private func session(id: UUID = UUID(), row: SeenAgent? = nil, opened: Date? = nil) -> SavedSession {
        SavedSession(id: id, hostID: host.id, name: "work", lastOpened: opened,
                     lastStatus: SessionStatusSnapshot(seenAt: observed, blocked: 0, working: 1,
                                                       hasIdle: false, location: [], agents: row.map { [$0] }))
    }

    func testLastSessionRequiresAnOpenAndBreaksTiesBySavedID() {
        let never = session()
        let earlier = session(opened: observed.addingTimeInterval(-10))
        let a = session(opened: observed), b = session(opened: observed)
        let expected = min(a.id.uuidString, b.id.uuidString)
        XCTAssertNil(HostCatalog(hosts: [host], sessions: [never]).lastOpenedSession)
        for sessions in [[b, never, earlier, a], [a, b, earlier, never]] {
            let catalog = HostCatalog(hosts: [host], sessions: sessions)
            XCTAssertEqual(catalog.lastOpenedSession?.id.uuidString, expected)
        }
        XCTAssertNil(HostCatalog(hosts: [], sessions: [earlier]).lastOpenedSession)
    }

    func testEntitiesUseSavedIDsAndTheObservationTimeSurvivesAStatusLookup() throws {
        let saved = session(row: agent())
        let catalog = HostCatalog(hosts: [host], sessions: [saved])
        XCTAssertEqual(catalog.systemHosts.map(\.id), [host.id])
        XCTAssertEqual(catalog.systemSessions.map(\.id), [saved.id])
        let entity = try XCTUnwrap(catalog.systemAgents.first)
        XCTAssertEqual(entity.name, "Claude")
        XCTAssertEqual(entity.status, "working")
        XCTAssertEqual(entity.observedAt, observed)
        let updated = HostCatalog(hosts: [host], sessions: [session(id: saved.id, row: agent("boot", "w", "t", "p", "Claude", "blocked"))])
        XCTAssertEqual(updated.systemAgent(id: entity.id)?.agent.status, "blocked")
        XCTAssertEqual(updated.systemAgents.first?.id, entity.id)
    }

    func testTwoAgentsWithTheSameNameAndLocationCanBeDistinguishedInShortcuts() throws {
        var saved = session()
        saved.lastStatus?.agents = [agent(), agent("boot", "w", "t", "second")]
        let entities = HostCatalog(hosts: [host], sessions: [saved]).systemAgents
        XCTAssertEqual(Set(entities.map(\.name)), ["Claude"])
        XCTAssertEqual(Set(entities.map(\.detail)).count, 2)
    }

    func testEveryTargetComponentFencesAgentLookup() throws {
        let saved = session(row: agent())
        let id = try XCTUnwrap(HostCatalog(hosts: [host], sessions: [saved]).systemAgents.first?.id)
        let replacements = [session(row: agent()), session(id: saved.id, row: agent("restart")),
                            session(id: saved.id, row: agent("boot", "workspace")),
                            session(id: saved.id, row: agent("boot", "w", "tab")),
                            session(id: saved.id, row: agent("boot", "w", "t", "pane")),
                            session(id: saved.id, row: agent("boot", "w", "t", "p", "Codex")),
                            session(id: saved.id)]
        for replacement in replacements {
            XCTAssertNil(HostCatalog(hosts: [host], sessions: [replacement]).systemAgent(id: id))
        }
    }

    func testIDComponentsKeepTheirBoundariesWithPunctuationAndUnicode() {
        let id = UUID()
        let a = session(id: id, row: agent("🌕/b", "c"))
        let b = session(id: id, row: agent("🌕", "b/c"))
        XCTAssertNotEqual(HostCatalog(hosts: [host], sessions: [a]).systemAgents.first?.id,
                          HostCatalog(hosts: [host], sessions: [b]).systemAgents.first?.id)
    }
}
