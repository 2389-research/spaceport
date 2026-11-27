// ABOUTME: Keeps the phone identity and verified SSH host keys in Keychain.
// ABOUTME: Adds trusted pins atomically and refuses conflicting first connections.
import CryptoKit
import Foundation
import Security

struct DeviceVault {
    static let enclaveAccount = "phone-key.secure-enclave.p256"
    let service: String

    init(service: String = "ai.2389.spaceport.mobile.ssh") { self.service = service }

    func savedPhoneKey() throws -> SecurePhoneKey? {
        guard SecurePhoneKey.isAvailable else { throw VaultError.enclaveUnavailable }
        guard let wrapped = try read(Self.enclaveAccount) else { return nil }
        return try SecurePhoneKey(SecureEnclave.P256.Signing.PrivateKey(dataRepresentation: wrapped))
    }

    func securePhoneKey() throws -> SecurePhoneKey {
        if let existing = try savedPhoneKey() { return existing }
        var error: Unmanaged<CFError>?
        guard let access = SecAccessControlCreateWithFlags(nil, kSecAttrAccessibleWhenUnlockedThisDeviceOnly,
                                                          .privateKeyUsage, &error) else {
            if let error { throw error.takeRetainedValue() }
            throw VaultError.unavailable
        }
        let key = try SecureEnclave.P256.Signing.PrivateKey(compactRepresentable: false, accessControl: access)
        // The enclave wraps this reference. A concurrent creator must reopen the item that won the insert.
        if try insert(key.dataRepresentation, account: Self.enclaveAccount) { return SecurePhoneKey(key) }
        guard let winner = try read(Self.enclaveAccount) else { throw VaultError.unavailable }
        return try SecurePhoneKey(SecureEnclave.P256.Signing.PrivateKey(dataRepresentation: winner))
    }

    func host(for address: SSHAddress) throws -> SshHost {
        try SshHost(hostname: address.hostname, port: address.port, username: address.username,
                    fingerprint: fingerprint(address.pinAccount))
    }

    func fingerprint(_ account: String) throws -> String? {
        guard let data = try read(account) else { return nil }
        guard let value = String(data: data, encoding: .utf8), !value.isEmpty else {
            throw VaultError.unavailable
        }
        return value
    }

    func trust(_ fingerprint: String, account: String) throws {
        guard !fingerprint.isEmpty else { throw VaultError.unavailable }
        if try insert(Data(fingerprint.utf8), account: account) { return }
        guard try self.fingerprint(account) == fingerprint else { throw VaultError.pinConflict }
    }

    private func query(_ account: String) -> [String: Any] {
        [kSecClass as String: kSecClassGenericPassword,
         kSecAttrSynchronizable as String: false,
         kSecAttrService as String: service,
         kSecAttrAccount as String: account]
    }

    private func read(_ account: String) throws -> Data? {
        var lookup = query(account)
        lookup[kSecReturnData as String] = true
        var result: CFTypeRef?
        let status = SecItemCopyMatching(lookup as CFDictionary, &result)
        if status == errSecItemNotFound { return nil }
        guard status == errSecSuccess, let data = result as? Data else { throw VaultError.status(status) }
        return data
    }

    private func insert(_ data: Data, account: String) throws -> Bool {
        var attributes = query(account)
        attributes[kSecAttrAccessible as String] = kSecAttrAccessibleWhenUnlockedThisDeviceOnly
        attributes[kSecValueData as String] = data
        let status = SecItemAdd(attributes as CFDictionary, nil)
        if status == errSecDuplicateItem { return false }
        guard status == errSecSuccess else { throw VaultError.status(status) }
        return true
    }

    func remove(_ account: String) throws {
        let status = SecItemDelete(query(account) as CFDictionary)
        guard status == errSecSuccess || status == errSecItemNotFound else { throw VaultError.status(status) }
    }
}

private enum VaultError: LocalizedError {
    case unavailable, enclaveUnavailable, pinConflict, status(OSStatus)
    var errorDescription: String? {
        switch self {
        case .enclaveUnavailable: "A hardware Secure Enclave is required for the iPhone SSH key."
        case .unavailable: "The saved SSH identity could not be read."
        case .pinConflict: "A different key is already trusted for this host and port."
        case let .status(code): "Keychain could not access this item (\(code))."
        }
    }
}
