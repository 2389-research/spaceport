// ABOUTME: Uses private Keychain items to exercise host trust and key retention.
// ABOUTME: Verifies that a second connection cannot silently replace a pin.
import Combine
import XCTest
@testable import SpaceportMobile

final class HostTrustTests: XCTestCase {
    @MainActor
    func testCopiedDiagnosticsExcludeFailedHostDataAndPassword() async throws {
        let profile = try HostProfile.make(hostname: "private-host.invalid", port: "22", username: "private-user",
                                           session: "private-session", login: .password)
        let connection = MobileConnection(profile: profile)
        connection.password = "private-password"
        let failed = expectation(description: "Host verification failed")
        let observation = connection.$phase.filter { $0 == .closed }.first().sink { _ in failed.fulfill() }
        connection.firstOpen { _ in throw NSError(domain: "private-remote-message", code: 7) }
        await fulfillment(of: [failed], timeout: 5)
        withExtendedLifetime(observation) {}
        let report = connection.diagnosticReport()
        XCTAssertTrue(report.contains("Last failure: SSH host verification"))
        XCTAssertTrue(report.contains("Last roster observation: unknown"))
        XCTAssertTrue(report.contains("Last negotiated capabilities: unavailable"))
        XCTAssertFalse(report.contains("private-"))
    }

    func testPinIsWriteOnceAcrossVaultInstancesAndRemovedOnlyByExplicitAction() throws {
        let service = "spaceport.mobile.tests.\(UUID())"
        let vault = DeviceVault(service: service)
        let account = "host/[example.org]:22"
        defer { try? vault.remove(account) }
        try vault.trust("SHA256:one", account: account)
        try vault.trust("SHA256:one", account: account)
        XCTAssertThrowsError(try vault.trust("SHA256:two", account: account))
        XCTAssertEqual(try DeviceVault(service: service).fingerprint(account), "SHA256:one")
        try vault.remove(account)
        XCTAssertNil(try vault.fingerprint(account))
    }

    @MainActor
    func testBackgroundAfterApprovalIgnoresCompletionOfTheRealPinWrite() async throws {
        let connection = try await waitingForTrust()
        let vault = DeviceVault(service: "spaceport.mobile.tests.\(UUID())")
        let account = connection.profile.pinAccount
        defer { try? vault.remove(account) }
        let token = connection.generation
        let fingerprint = try XCTUnwrap(connection.challenge?.fingerprint)

        connection.background()
        connection.foreground()
        XCTAssertEqual(connection.phase, .trust)
        XCTAssertEqual(connection.challenge?.fingerprint, fingerprint)
        XCTAssertEqual(connection.generation, token)
        XCTAssertNil(try vault.fingerprint(account))

        let approval = try XCTUnwrap(connection.trust(using: vault))
        XCTAssertEqual(connection.phase, .connecting)
        connection.background()
        let backgroundToken = connection.generation
        XCTAssertNotEqual(backgroundToken, token)
        await approval.value
        XCTAssertEqual(try vault.fingerprint(account), fingerprint)
        XCTAssertEqual(connection.phase, .closed)
        XCTAssertEqual(connection.generation, backgroundToken)
    }

    @MainActor
    func testPinWriteFailureRestoresOnlyTheCurrentApprovalPrompt() async throws {
        let vault = DeviceVault(service: "spaceport.mobile.tests.\(UUID())")
        let connection = try await waitingForTrust()
        let account = connection.profile.pinAccount
        defer { try? vault.remove(account) }
        try vault.trust("SHA256:already-pinned", account: account)

        let rejected = try XCTUnwrap(connection.trust(using: vault))
        XCTAssertEqual(connection.phase, .connecting)
        await rejected.value
        XCTAssertEqual(connection.phase, .trust)
        XCTAssertNotNil(connection.message)
        XCTAssertEqual(connection.challenge?.fingerprint, "SHA256:presented")

        connection.message = nil
        let stale = try XCTUnwrap(connection.trust(using: vault))
        connection.background()
        await stale.value
        XCTAssertEqual(connection.phase, .closed)
        XCTAssertNil(connection.challenge)
        XCTAssertNil(connection.message)
        XCTAssertEqual(try vault.fingerprint(account), "SHA256:already-pinned")
    }

    @MainActor
    private func waitingForTrust() async throws -> MobileConnection {
        let profile = try HostProfile.make(hostname: "trust-test.invalid", port: "22", username: "test",
                                           session: "default", login: .password)
        let connection = MobileConnection(profile: profile)
        let prompted = expectation(description: "SSH preflight presented a host key")
        let observation = connection.$phase.filter { $0 == .trust }.first().sink { _ in prompted.fulfill() }
        connection.firstOpen { _ in throw SshError.UnknownHost(fingerprint: "SHA256:presented") }
        await fulfillment(of: [prompted], timeout: 5)
        withExtendedLifetime(observation) {}
        return connection
    }

}
