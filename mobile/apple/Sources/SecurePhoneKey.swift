// ABOUTME: Signs SSH challenges with a device-bound Secure Enclave P-256 key.
// ABOUTME: Supplies Rust with the public point and a signing callback.
import CryptoKit
import Foundation

final class SecurePhoneKey: EnclaveSigner {
    private let key: SecureEnclave.P256.Signing.PrivateKey

    init(_ key: SecureEnclave.P256.Signing.PrivateKey) { self.key = key }

    static var isAvailable: Bool {
        #if targetEnvironment(simulator)
        false
        #else
        SecureEnclave.isAvailable
        #endif
    }

    var publicPoint: Data { key.publicKey.x963Representation }
    var publicKey: String { get throws { try enclavePublicKey(publicKey: publicPoint) } }
    var auth: SshAuth { .secureEnclave(publicKey: publicPoint, signer: self) }

    func sign(message: Data) throws -> Data {
        do {
            return try key.signature(for: message).derRepresentation
        } catch {
            throw SshError.Key
        }
    }
}
