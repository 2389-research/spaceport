// ABOUTME: Presents the complete negotiated SSH fingerprint before the user grants trust.
// ABOUTME: Shows both fingerprints on a mismatch and redirects trust management to the saved host.
import SwiftUI

extension HostChallenge {
    init?(sshError: Error) {
        switch sshError {
        case let SshError.UnknownHost(fingerprint): self.init(fingerprint: fingerprint, changed: false)
        case let SshError.ChangedHost(fingerprint): self.init(fingerprint: fingerprint, changed: true)
        default: return nil
        }
    }
}

struct VerifyHostView: View {
    let address: SSHAddress
    let challenge: HostChallenge
    let trustLabel: String
    let trust: () -> Void
    let cancel: () -> Void

    let openHostDetails: () -> Void
    @State private var savedFingerprint: String?

    var body: some View {
        ScrollView {
            VStack(alignment: .leading, spacing: 18) {
                Image(systemName: challenge.changed ? "exclamationmark.shield.fill" : "checkmark.shield.fill")
                    .font(.system(size: 34, weight: .medium))
                    .foregroundStyle(challenge.changed ? Color.red : .accentColor)
                    .frame(width: 64, height: 64)
                    .background((challenge.changed ? Color.red : .accentColor).opacity(0.1), in: RoundedRectangle(cornerRadius: 18))
                    .accessibilityHidden(true)
                Text(challenge.changed ? "Host key changed" : "Verify \(address.hostname)")
                    .font(.largeTitle.bold()).accessibilityAddTraits(.isHeader)
                Text(challenge.changed
                     ? "\(address.hostname) sent a different key from the one you trusted. Check with the host administrator before connecting again."
                     : "First time connecting. Compare the full fingerprint with the host before you trust it.")
                    .foregroundStyle(.secondary)
                VStack(alignment: .leading, spacing: 12) {
                    Text(verbatim: address.endpoint).font(.subheadline.weight(.semibold))
                    if challenge.changed {
                        Divider()
                        Text("Trusted").font(.subheadline).foregroundStyle(.secondary)
                        if let savedFingerprint {
                            HostFingerprint(fingerprint: savedFingerprint, identifier: "trust.savedFingerprint", compact: true)
                        } else {
                            Text("Saved fingerprint unavailable").font(.subheadline).foregroundStyle(.secondary)
                        }
                        Divider()
                        Text("Presented now").font(.subheadline).foregroundStyle(.secondary)
                    }
                    HostFingerprint(fingerprint: challenge.fingerprint, compact: challenge.changed)
                }
                .padding(20).frame(maxWidth: .infinity, alignment: .leading)
                .background(Color(uiColor: .secondarySystemGroupedBackground), in: RoundedRectangle(cornerRadius: 26))
                if challenge.changed {
                    Text("If the change is expected, verify the new key with the administrator, then forget the old key in Host Details.")
                        .font(.footnote).foregroundStyle(.secondary)
                } else {
                    VStack(alignment: .leading, spacing: 4) {
                        Text("Ask the administrator for the fingerprint. For an Ed25519 host key, run this on the host.")
                        Text(verbatim: "ssh-keygen -lf /etc/ssh/ssh_host_ed25519_key.pub")
                            .fontDesign(.monospaced).textSelection(.enabled)
                    }.font(.footnote).foregroundStyle(.secondary)
                }
                Button(action: challenge.changed ? openHostDetails : trust) {
                    Text(challenge.changed ? "Open Host Details" : trustLabel)
                        .fontWeight(.semibold).frame(maxWidth: .infinity, minHeight: 42)
                }
                .buttonStyle(.borderedProminent).buttonBorderShape(.capsule)
                Button(challenge.changed ? "Cancel" : "It Doesn't Match", role: .cancel, action: cancel)
                    .frame(maxWidth: .infinity, minHeight: 44)
            }
            .frame(maxWidth: 520, alignment: .leading).padding(20).frame(maxWidth: .infinity)
        }
        .background(Color(uiColor: .systemGroupedBackground))
        .task(id: address.pinAccount) {
            savedFingerprint = nil
            guard challenge.changed else { return }
            let account = address.pinAccount
            let value = await Task.detached { try? DeviceVault().fingerprint(account) }.value
            if !Task.isCancelled { savedFingerprint = value }
        }
    }
}

struct HostFingerprint: View {
    let fingerprint: String
    var identifier = "trust.fingerprint"
    var compact = false
    @Environment(\.dynamicTypeSize) private var textSize

    var body: some View {
        let parts = fingerprint.split(separator: ":", maxSplits: 1)
        if let algorithm = parts.first, parts.count == 2 {
            let digest = Array(parts[1])
            let chunks = stride(from: 0, to: digest.count, by: 4).map { start in
                String(digest[start..<min(start + 4, digest.count)])
            }
            VStack(alignment: .leading, spacing: 6) {
                Text(String(algorithm)).font(.caption.weight(.semibold))
                if compact {
                    Text(verbatim: chunks.joined(separator: " "))
                        .font(.subheadline.monospaced()).textSelection(.enabled)
                        .fixedSize(horizontal: false, vertical: true)
                } else {
                    LazyVGrid(columns: Array(repeating: GridItem(.flexible(), alignment: .leading),
                                             count: textSize.isAccessibilitySize ? 2 : 4),
                              alignment: .leading, spacing: 8) {
                        ForEach(chunks.indices, id: \.self) { index in
                            Text(verbatim: chunks[index]).font(.body.monospaced()).padding(.vertical, 5)
                        }
                    }
                }
            }
            .accessibilityElement(children: .ignore)
            .accessibilityLabel(fingerprint)
            .accessibilityIdentifier(identifier)
        } else {
            Text(verbatim: fingerprint).font(.body.monospaced()).textSelection(.enabled)
        }
    }
}
