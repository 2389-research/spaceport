// ABOUTME: Adds or edits an SSH address and its shared sign-in method.
// ABOUTME: Creates and renames Herdr sessions without entering the address again.
import SwiftUI
import UIKit

struct HostEditor: View {
    let host: SavedHost?
    let save: (SSHAddress, LoginMethod, String) throws -> Void
    @Environment(\.dismiss) private var dismiss
    @State private var address: String
    @State private var name: String
    @State private var session = ""
    @State private var login: LoginMethod
    @State private var notice: String?
    @State private var copied = false

    init(host: SavedHost? = nil, save: @escaping (SSHAddress, LoginMethod, String) throws -> Void) {
        self.host = host
        self.save = save
        _address = State(initialValue: host?.address.destination ?? "")
        _name = State(initialValue: host?.name ?? "")
        _login = State(initialValue: host?.login ?? .phoneKey)
    }

    var body: some View {
        NavigationStack {
            Form {
                Section {
                    TextField("user@host:port", text: $address)
                        .keyboardType(.URL).accessibilityLabel("Address").accessibilityIdentifier("Address")
                } header: { Text("Address") } footer: {
                    if let parsed = try? SSHAddress.parse(address) {
                        Text("Account **\(parsed.username)** on **\(parsed.hostname)**, port \(String(parsed.port)).")
                    } else { Text("Enter user@host. Add :port if the SSH port is different from 22.") }
                }
                if host != nil {
                    Section("Display Name") {
                        TextField("Name", text: $name, prompt: Text("Optional")).accessibilityLabel("Name")
                    }
                }
                Section {
                    ForEach(LoginMethod.allCases, id: \.self) { method in
                        Button {
                            login = method
                        } label: {
                            HStack {
                                Text(method.title).foregroundStyle(.primary)
                                if method == .phoneKey { Text("· Recommended").foregroundStyle(.secondary) }
                                Spacer()
                                if method == login { Image(systemName: "checkmark").fontWeight(.semibold) }
                            }
                        }
                        .accessibilityLabel(method.title)
                        .accessibilityAddTraits(method == login ? .isSelected : [])
                    }
                } header: { Text("Sign In With") } footer: {
                    VStack(alignment: .leading, spacing: 8) {
                        if login == .phoneKey {
                            Text("Add this phone’s Secure Enclave public key to the account once.")
                            Button(copied ? "Public Key Copied" : "Copy Public Key") { copyPublicKey() }
                        } else {
                            Text("Sign in with your SSH password. On the sign-in screen, choose whether to authorize this phone for future logins.")
                        }
                    }
                }
                if host == nil {
                    Section {
                        TextField("default", text: $session).accessibilityLabel("Session").accessibilityIdentifier("Session")
                    } header: { Text("Herdr Session") } footer: {
                        Text("Herdr starts this session if needed. Add more sessions later from the host.")
                    }
                }
            }
            .textInputAutocapitalization(.never).autocorrectionDisabled()
            .navigationTitle(host == nil ? "Add Host" : "Edit Host").navigationBarTitleDisplayMode(.inline)
            .toolbar {
                ToolbarItem(placement: .cancellationAction) {
                    Button("Cancel", systemImage: "xmark") { dismiss() }.labelStyle(.iconOnly)
                }
                ToolbarItem(placement: .confirmationAction) {
                    Button(host == nil ? "Connect" : "Save") {
                        do {
                            try save(SSHAddress.parse(address), login,
                                     host == nil ? (session.isEmpty ? "default" : session) : name)
                            dismiss()
                        } catch { notice = error.localizedDescription }
                    }
                    .buttonStyle(.borderedProminent).buttonBorderShape(.capsule)
                }
            }
            .alert("Host Settings", isPresented: $notice.presented) { Button("OK") { notice = nil } }
                message: { Text(notice ?? "") }
        }
    }

    private func copyPublicKey() {
        Task {
            do {
                let key = try await Task.detached { try DeviceVault().securePhoneKey().publicKey }.value
                UIPasteboard.general.string = key
                copied = true
            } catch { notice = error.localizedDescription }
        }
    }
}

struct SessionEditor: View {
    let profile: HostProfile?
    let save: (String) throws -> Void
    @State private var name: String
    @State private var notice: String?
    @Environment(\.dismiss) private var dismiss

    init(profile: HostProfile? = nil, save: @escaping (String) throws -> Void) {
        self.profile = profile
        self.save = save
        _name = State(initialValue: profile?.session ?? "")
    }

    var body: some View {
        NavigationStack {
            Form {
                Section("Herdr Session") {
                    TextField("default", text: $name).accessibilityLabel("Session").accessibilityIdentifier("Session")
                        .autocorrectionDisabled().textInputAutocapitalization(.never)
                }
            }
            .navigationTitle(profile == nil ? "Add Session" : "Edit Session").navigationBarTitleDisplayMode(.inline)
            .toolbar {
                ToolbarItem(placement: .cancellationAction) { Button("Cancel") { dismiss() } }
                ToolbarItem(placement: .confirmationAction) {
                    Button(profile == nil ? "Add" : "Save") {
                        do { try save(name.isEmpty ? "default" : name); dismiss() }
                        catch { notice = error.localizedDescription }
                    }
                }
            }
            .alert("Session Settings", isPresented: $notice.presented) { Button("OK") { notice = nil } }
                message: { Text(notice ?? "") }
        }
    }
}
