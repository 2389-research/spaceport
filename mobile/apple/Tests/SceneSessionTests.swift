// ABOUTME: Verifies scene-local SSH ownership, exact session identity and eviction.
// ABOUTME: Exercises real connection controllers without opening a network transport.
import XCTest
@testable import SpaceportMobile

@MainActor
final class SceneSessionTests: XCTestCase {
    private func profile(_ name: String) throws -> HostProfile {
        try HostProfile.make(hostname: "cache-test.invalid", port: "22", username: "test", session: name, login: .password)
    }

    func testHomeObservationPreservesDraftsAndNeverCreatesASession() async throws {
        let owner = SceneSessions()
        let a = try profile("A"), b = try profile("B"), c = try profile("C")
        let draft = owner.connection(for: a)
        draft.draft.text = "unfinished"
        _ = owner.connection(for: b)
        let missing = try await owner.observeRetainedRoster(for: c)
        let cached = try await owner.observeRetainedRoster(for: a)
        XCTAssertNil(missing)
        XCTAssertNil(cached)
        XCTAssertNil(owner.current)
        XCTAssertFalse(owner.contains(c))
        XCTAssertEqual(draft.draft.text, "unfinished")
        _ = owner.connection(for: c)
        XCTAssertFalse(owner.contains(a), "Reading a draft must not change eviction order")
    }

    func testManualDisconnectSuppressesHomeRefreshUntilReopened() throws {
        let owner = SceneSessions()
        let a = try profile("A"), b = try profile("B")
        owner.disconnect(a)
        XCTAssertFalse(owner.permitsHomeRefresh(a))
        XCTAssertTrue(owner.permitsHomeRefresh(b))
        _ = owner.open(a, agent: nil)
        XCTAssertTrue(owner.permitsHomeRefresh(a))
    }

    func testForgettingThePinStopsHomeReadsForUnopenedSessionsOnTheHost() throws {
        let owner = SceneSessions()
        let a = try profile("A"), b = try profile("B")
        owner.forget(pinAccount: a.pinAccount)
        XCTAssertFalse(owner.permitsHomeRefresh(a))
        XCTAssertFalse(owner.permitsHomeRefresh(b))
        _ = owner.open(a, agent: nil)
        XCTAssertTrue(owner.permitsHomeRefresh(a))
    }

    func testRevisitingAProtectsItWhenAThirdSessionEvictsB() throws {
        let owner = SceneSessions()
        let a = try profile("A"), b = try profile("B"), c = try profile("C")
        let first = owner.connection(for: a)
        let second = owner.connection(for: b)
        let oldGeneration = second.generation
        second.password = "in-memory test password"
        XCTAssertTrue(owner.connection(for: a) === first)
        _ = owner.connection(for: c)
        XCTAssertFalse(owner.contains(b))
        XCTAssertTrue(owner.contains(a))
        XCTAssertTrue(owner.contains(c))
        XCTAssertNotEqual(second.generation, oldGeneration)
        XCTAssertTrue(second.password.isEmpty)
    }

    func testVerifiedPhoneKeyKeepsTheSameConnectionAcrossHome() throws {
        let suite = "spaceport.mobile.enroll.\(UUID())"
        let defaults = try XCTUnwrap(UserDefaults(suiteName: suite))
        defer { defaults.removePersistentDomain(forName: suite) }
        let store = ProfileStore(defaults: defaults)
        let original = try profile("A")
        try store.save(original)
        let owner = SceneSessions()
        let connection = owner.connection(for: original)
        let generation = connection.generation
        connection.password = "temporary test credential"
        try connection.saveVerifiedPhoneKey(using: store)

        let updated = try XCTUnwrap(store.load().first)
        XCTAssertEqual(updated.id, original.id)
        XCTAssertEqual(updated.login, .phoneKey)
        XCTAssertEqual(connection.method, .phoneKey)
        XCTAssertTrue(connection.password.isEmpty)
        owner.reconcile([updated])
        XCTAssertTrue(owner.connection(for: updated) === connection)
        XCTAssertEqual(connection.generation, generation)
    }

    func testVerifiedEnrollmentKeepsAllHostSessionsAndDraftsAcrossWindows() throws {
        let suite = "spaceport.mobile.enroll.\(UUID())"
        let defaults = try XCTUnwrap(UserDefaults(suiteName: suite))
        defer { defaults.removePersistentDomain(forName: suite) }
        let store = ProfileStore(defaults: defaults)
        let a = try profile("A"), b = try profile("B")
        try store.save(a)
        try store.save(b)
        let first = SceneSessions(), second = SceneSessions()
        let active = first.connection(for: a)
        let parked = first.connection(for: b)
        let window = second.connection(for: a)
        let owned = [active, parked, window]
        let generations = owned.map(\.generation)
        for (index, connection) in owned.enumerated() {
            connection.draft.text = "draft \(index)"
            connection.password = "ephemeral"
        }
        try active.saveVerifiedPhoneKey(using: store)
        let saved = try store.load()
        first.reconcile(saved)
        second.reconcile(saved)
        let verifiedA = try XCTUnwrap(saved.first { $0.id == a.id })
        let verifiedB = try XCTUnwrap(saved.first { $0.id == b.id })
        XCTAssertEqual(verifiedA.login, .phoneKey)
        XCTAssertEqual(verifiedB.login, .phoneKey)
        XCTAssertTrue(first.connection(for: verifiedA) === active)
        XCTAssertTrue(first.connection(for: verifiedB) === parked)
        XCTAssertTrue(second.connection(for: verifiedA) === window)
        XCTAssertEqual(owned.map(\.generation), generations)
        XCTAssertTrue(owned.allSatisfy { $0.password.isEmpty && $0.method == .phoneKey })
        XCTAssertEqual(owned.map { $0.draft.text }, ["draft 0", "draft 1", "draft 2"])
    }

    func testLateEnrollmentCannotChangeAnEditedDestination() throws {
        let suite = "spaceport.mobile.enroll.\(UUID())"
        let defaults = try XCTUnwrap(UserDefaults(suiteName: suite))
        defer { defaults.removePersistentDomain(forName: suite) }
        let store = ProfileStore(defaults: defaults)
        let original = try profile("A")
        try store.save(original)
        let connection = MobileConnection(profile: original)
        var changed = original
        changed.hostname = "other.invalid"
        try store.save(changed)
        XCTAssertThrowsError(try connection.saveVerifiedPhoneKey(using: store))
        XCTAssertEqual(try store.load(), [changed])
        XCTAssertEqual(connection.profile, original)
        XCTAssertEqual(connection.method, .password)
    }

    func testChangedLoginAndForgottenPinRetireExactConnections() throws {
        let owner = SceneSessions()
        let a = try profile("A"), b = try profile("B")
        let first = owner.connection(for: a)
        _ = owner.connection(for: b)
        var edited = a
        edited.login = .phoneKey
        owner.reconcile([edited, b])
        XCTAssertFalse(owner.contains(a))
        XCTAssertTrue(owner.contains(b))
        XCTAssertFalse(owner.connection(for: edited) === first)
        owner.forget(pinAccount: b.pinAccount)
        XCTAssertFalse(owner.contains(edited))
        XCTAssertFalse(owner.contains(b))
    }
    func testDisconnectRevokesPresentationAndDropsLocalSecretsAndDraft() throws {
        let owner = SceneSessions()
        let saved = try profile("A")
        let connection = owner.connection(for: saved)
        let previous = UUID()
        connection.present(previous, agent: nil, alreadyOpen: true)
        connection.password = "only in memory"
        connection.draft.text = "unfinished"
        owner.disconnect(saved)
        XCTAssertNil(connection.presentationID)
        XCTAssertFalse(connection.acceptsInput(from: previous))
        XCTAssertTrue(connection.password.isEmpty)
        XCTAssertTrue(connection.draft.text.isEmpty)
        XCTAssertFalse(owner.contains(saved))
    }

    func testUnchangedCatalogKeepsHandlesAndRemovingOneSessionRetiresOnlyThatTarget() throws {
        let owner = SceneSessions()
        let a = try profile("A"), b = try profile("B")
        let first = owner.connection(for: a)
        let second = owner.connection(for: b)
        owner.reconcile([a, b])
        XCTAssertTrue(owner.connection(for: a) === first)
        owner.reconcile([b])
        XCTAssertFalse(owner.contains(a))
        XCTAssertTrue(owner.connection(for: b) === second)
    }

    func testHomeDropsTemporaryLoginButKeepsItsUnsentDraft() throws {
        let owner = SceneSessions()
        let saved = try profile("A")
        let presentation = owner.open(saved, agent: nil)
        let connection = presentation.connection
        connection.method = .phoneKey
        connection.draft.text = "unfinished"
        owner.detach(presentation)
        XCTAssertTrue(owner.contains(saved))
        XCTAssertEqual(connection.draft.text, "unfinished")
        XCTAssertEqual(connection.method, saved.login)
        XCTAssertEqual(connection.phase, .closed)
        XCTAssertNil(connection.presentationID)
    }

    func testHomeRetainsAnUnsentDraftWhenItsTransportCannotBeParked() throws {
        let owner = SceneSessions()
        let saved = try profile("A")
        let presentation = owner.open(saved, agent: nil)
        let connection = presentation.connection
        connection.draft.text = "keep my draft"
        connection.password = "do not retain"
        owner.detach(presentation)
        XCTAssertTrue(owner.contains(saved))
        XCTAssertNil(owner.current)
        XCTAssertEqual(connection.phase, .closed)
        XCTAssertNil(connection.presentationID)
        XCTAssertTrue(connection.password.isEmpty)
        XCTAssertTrue(owner.connection(for: saved) === connection)
        XCTAssertEqual(connection.draft.text, "keep my draft")
        owner.disconnect(saved)
        XCTAssertTrue(connection.draft.text.isEmpty)
    }

}
