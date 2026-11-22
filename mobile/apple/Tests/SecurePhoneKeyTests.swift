// ABOUTME: Exercises phone-key creation, persistence and migration through the device Keychain.
// ABOUTME: Verifies hardware signatures and checks that unavailable hardware fails closed.
import CryptoKit
import Security
import XCTest

@testable import SpaceportMobile

final class SecurePhoneKeyTests: XCTestCase {
    private let legacyAccount = "phone-key"

    private func legacyItem(_ vault: DeviceVault) throws -> Data? {
        let query: [String: Any] = [
            kSecClass as String: kSecClassGenericPassword,
            kSecAttrService as String: vault.service,
            kSecAttrAccount as String: legacyAccount,
            kSecReturnData as String: true
        ]
        var result: CFTypeRef?
        let status = SecItemCopyMatching(query as CFDictionary, &result)
        if status == errSecItemNotFound { return nil }
        guard status == errSecSuccess else { throw NSError(domain: NSOSStatusErrorDomain, code: Int(status)) }
        return try XCTUnwrap(result as? Data)
    }

    func testLegacyKeyRemainsByteIdenticalWhenRequestingTheHardwareIdentity() throws {
        let vault = DeviceVault(service: "spaceport.mobile.tests.\(UUID())")
        defer {
            try? vault.remove(legacyAccount)
            try? vault.remove(DeviceVault.enclaveAccount)
        }
        XCTAssertNil(try legacyItem(vault))
        let data = Data(#"{"privateKey":"opaque old key","publicKey":"opaque old public key"}"#.utf8)
        let item: [String: Any] = [
            kSecClass as String: kSecClassGenericPassword,
            kSecAttrService as String: vault.service,
            kSecAttrAccount as String: legacyAccount,
            kSecAttrAccessible as String: kSecAttrAccessibleWhenUnlockedThisDeviceOnly,
            kSecValueData as String: data
        ]
        XCTAssertEqual(SecItemAdd(item as CFDictionary, nil), errSecSuccess)
        if SecurePhoneKey.isAvailable {
            _ = try vault.securePhoneKey()
        } else {
            XCTAssertThrowsError(try vault.securePhoneKey())
        }
        XCTAssertEqual(try legacyItem(vault), data)
    }

    func testReopeningAndConcurrentCreationUseOneHardwareIdentity() async throws {
        try XCTSkipUnless(SecurePhoneKey.isAvailable, "Requires a hardware Secure Enclave.")
        let service = "spaceport.mobile.tests.\(UUID())"
        let vault = DeviceVault(service: service)
        defer { try? vault.remove(DeviceVault.enclaveAccount) }
        let points = try await withThrowingTaskGroup(of: Data.self) { group in
            for _ in 0..<4 { group.addTask { try DeviceVault(service: service).securePhoneKey().publicPoint } }
            var results = Set<Data>()
            for try await point in group { results.insert(point) }
            return results
        }
        let reopened = try vault.securePhoneKey()
        XCTAssertEqual(points, [reopened.publicPoint])
        let pub = try P256.Signing.PublicKey(x963Representation: reopened.publicPoint)
        let message = Data(UUID().uuidString.utf8)
        let signature = try P256.Signing.ECDSASignature(derRepresentation: reopened.sign(message: message))
        XCTAssertTrue(pub.isValidSignature(signature, for: message))
        XCTAssertFalse(pub.isValidSignature(signature, for: Data()))
        XCTAssertNil(try legacyItem(vault))
    }

    func testUnavailableHardwareNeverCreatesASoftwareIdentity() throws {
        try XCTSkipIf(SecurePhoneKey.isAvailable, "Requires a device without Secure Enclave support.")
        let vault = DeviceVault(service: "spaceport.mobile.tests.\(UUID())")
        XCTAssertThrowsError(try vault.securePhoneKey())
        XCTAssertNil(try legacyItem(vault))
    }
}
