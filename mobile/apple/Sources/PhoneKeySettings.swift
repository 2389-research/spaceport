// ABOUTME: Shows the iPhone's public SSH identity and installs it on a selected account.
// ABOUTME: Requires explicit host trust and a verified key-only login before saving the sign-in choice.
import SwiftUI
import UIKit

struct PhoneKeySettings: View {
    let hosts: [SavedHost]
    let openHostDetails: (UUID) -> Void
    @State private var publicKey = ""
    @State private var issue: String?

    var body: some View {
        Section {
            if !publicKey.isEmpty {
                Text(verbatim: publicKey).font(.footnote.monospaced()).textSelection(.enabled)
                Button("Copy Public Key", systemImage: "doc.on.doc") { UIPasteboard.general.string = publicKey }
                ShareLink(item: publicKey) { Label("Share…", systemImage: "square.and.arrow.up") }
            } else if let issue { Text(issue) } else { ProgressView() }

            NavigationLink("Install on a Host…") {
                List(hosts) { host in
                    NavigationLink {
                        InstallKeyForm(host: host, openHostDetails: { openHostDetails(host.id) })
                    } label: {
                        VStack(alignment: .leading) {
                            Text(verbatim: host.address.destination)
                            Text(host.login.title).font(.caption).foregroundStyle(.secondary)
                        }
                    }
                    .accessibilityLabel(host.address.destination)
                    .accessibilityValue(host.login.title)
                }
                .navigationTitle("Choose a host")
            }
            .disabled(hosts.isEmpty || publicKey.isEmpty)
        } header: { Text("Phone Key") } footer: {
            Text("The Secure Enclave generates this P-256 key and signs SSH logins while your device is unlocked.")
        }
        .task {
            do { publicKey = try await Task.detached { try DeviceVault().securePhoneKey().publicKey }.value }
            catch { issue = error.localizedDescription }
        }
    }
}

struct InstallKeyForm: View {
    let host: SavedHost
    let openHostDetails: () -> Void
    @Environment(\.dismiss) private var dismiss
    @State private var password = ""
    @State private var busy = false
    @State private var challenge: HostChallenge?
    @State private var installed = false
    @State private var issue: String?

    var body: some View {
        Group {
            if let challenge {
                VerifyHostView(address: host.address, challenge: challenge, trustLabel: "Trust host",
                               trust: { install(trusting: challenge.fingerprint) }, cancel: { dismiss() }, openHostDetails: openHostDetails)
                    .disabled(busy)
            } else {
                Form {
                    Section {
                        Label(host.address.destination, systemImage: "server.rack").font(.headline)
                        if installed {
                            Label("Key installed", systemImage: "checkmark.circle")
                            Text("The host accepted a fresh SSH login using this phone’s key.")
                        } else {
                            SecureField("SSH password", text: $password).textContentType(.password)
                                .onSubmit { install() }
                            Button("Install key") { install() }.disabled(password.isEmpty)
                        }
                    } footer: {
                        if !installed {
                            Text("Use this account’s SSH password to add the phone’s public key to authorized_keys. Spaceport then checks that it can sign in using the key.")
                        }
                    }
                    if busy { ProgressView("Checking SSH access…") }
                    if let issue { Text(issue) }
                }
                .disabled(busy)
            }
        }
        .navigationTitle("Install key to host")
        .navigationBarTitleDisplayMode(.inline)
        .navigationBarBackButtonHidden(busy)
        .interactiveDismissDisabled(busy)
        .onDisappear { password = "" }
    }

    private func install(trusting fingerprint: String? = nil) {
        guard !busy, !installed, !password.isEmpty else { return }
        busy = true
        issue = nil
        let credential = password
        let host = self.host
        Task { @MainActor in
            defer { busy = false }
            do {
                try await Task.detached {
                    let vault = DeviceVault()
                    if let fingerprint { try vault.trust(fingerprint, account: host.address.pinAccount) }
                    let key = try vault.securePhoneKey()
                    try authorizeEnclavePhone(host: vault.host(for: host.address), password: credential,
                                              publicKey: key.publicPoint, signer: key)
                }.value
                challenge = nil
                password = ""
                installed = true
                do {
                    if try !ProfileStore().usePhoneKey(for: host) {
                        issue = "Another saved host already uses this account with Phone Key. Both hosts kept their sign-in settings."
                    }
                }
                catch { issue = "The host accepted the key. Spaceport could not save the sign-in preference." }
            } catch {
                challenge = HostChallenge(sshError: error)
                if challenge?.changed == true { password = "" }
                guard challenge == nil else { return }
                switch error {
                case SshError.Authentication: issue = "SSH refused password sign-in for this account."
                case SshError.UnverifiedEnrollment: issue = "The key may have been installed. A fresh key-only login failed."
                case SshError.Enrollment, SshError.Channel: issue = "SSH signed in, but the host refused to install the public key."
                case SshError.Timeout: issue = "SSH timed out. Check the host address and network."
                case SshError.Connection: issue = "Spaceport could not reach the SSH host."
                default: issue = error.localizedDescription
                }
            }
        }
    }
}
