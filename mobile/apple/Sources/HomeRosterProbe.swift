// ABOUTME: Reads a running session’s roster from Home using the saved SSH pin and Phone Key.
// ABOUTME: Keeps Keychain access and the blocking Rust request off the main actor.
import Foundation

enum HomeRosterProbe {
    static func read(_ profile: HostProfile) async throws -> MobileView? {
        let request = MobileRosterRequest()
        return try await withTaskCancellationHandler {
            try await Task.detached(priority: .utility) { () throws -> MobileView? in
                let vault = DeviceVault()
                let host = try vault.host(for: profile.address)
                guard let pin = host.fingerprint, let key = try vault.savedPhoneKey() else { return nil }
                let roster = try request.read(host: host, auth: key.auth, session: profile.session)
                guard try vault.fingerprint(profile.pinAccount) == pin else { return nil }
                return roster
            }.value
        } onCancel: {
            request.cancel()
        }
    }
}
