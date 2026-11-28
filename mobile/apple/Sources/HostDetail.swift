// ABOUTME: Shows the shared SSH account, host trust, and sessions saved for one host.
// ABOUTME: Requires confirmation before forgetting a trusted fingerprint or removing a host.
import SwiftUI

struct HostDetail: View {
    let host: SavedHost
    let sessions: [SavedSession]
    let saveTags: (String) throws -> Void
    @State private var editingTags = false
    @State private var tagText = ""
    let open: (HostProfile) -> Void
    let edit: () -> Void
    let add: () -> Void
    let remove: () throws -> Void
    @State private var fingerprint: String?
    @State private var installingKey = false
    @State private var confirmForget = false
    @State private var confirmRemove = false
    @State private var notice: String?

    var body: some View {
        List {
            Section("Connection") {
                LabeledContent("Host", value: host.address.hostname)
                LabeledContent("Account", value: host.address.username)
                LabeledContent("Port", value: String(host.address.port))
                Button(action: edit) {
                    HStack {
                        LabeledContent("Sign In With", value: host.login.title)
                        Image(systemName: "chevron.right").font(.caption.weight(.semibold)).foregroundStyle(.tertiary)
                    }.contentShape(Rectangle())
                }.buttonStyle(.plain)
                Button(host.login == .password ? "Authorize This Phone" : "Install key to host", systemImage: "key") { installingKey = true }
            }
            Section {
                if let fingerprint {
                    HostFingerprint(fingerprint: fingerprint)
                    Button("Forget Host Key…", role: .destructive) { confirmForget = true }
                } else {
                    Text("No trusted key saved").foregroundStyle(.secondary)
                }
            } header: { Text("Trusted Host Key") } footer: {
                if fingerprint != nil { Text("Verified on this device. This key applies to every account on this host and port.") }
            }
            Section("Tags") {
                Button {
                    tagText = (host.tags ?? []).joined(separator: ", ")
                    editingTags = true
                } label: {
                    LabeledContent("Edit tags", value: (host.tags ?? []).joined(separator: ", "))
                }
            }
            Section("Herdr Sessions") {
                ForEach(sessions) { session in
                    Button { open(host.profile(session: session)) } label: {
                        HStack { Text(session.name); Spacer(); Image(systemName: "chevron.right").font(.caption).foregroundStyle(.tertiary) }
                            .contentShape(Rectangle())
                    }.buttonStyle(.plain)
                }
                Button("Add Session", systemImage: "plus", action: add)
            }
            Section {
                Button("Remove Host", role: .destructive) { confirmRemove = true }.frame(maxWidth: .infinity)
            }
        }
        .listStyle(.insetGrouped).textCase(nil).navigationTitle(host.title).navigationBarTitleDisplayMode(.inline)
        .navigationDestination(isPresented: $installingKey) { InstallKeyForm(host: host, openHostDetails: { installingKey = false }) }
        .toolbar { ToolbarItem(placement: .topBarTrailing) { Button("Edit", action: edit) } }
        .task(id: host.address.pinAccount) {
            fingerprint = nil
            let account = host.address.pinAccount
            do {
                let value = try await Task.detached { try DeviceVault().fingerprint(account) }.value
                if !Task.isCancelled { fingerprint = value }
            } catch { if !Task.isCancelled { notice = error.localizedDescription } }
        }
        .confirmationDialog("Forget the trusted key for \(host.address.hostname):\(String(host.address.port))?",
                            isPresented: $confirmForget, titleVisibility: .visible) {
            Button("Forget Host Key", role: .destructive) {
                let account = host.address.pinAccount
                Task {
                    do {
                        try await Task.detached { try DeviceVault().remove(account) }.value
                        NotificationCenter.default.post(name: SceneSessions.forgotPin, object: account)
                        fingerprint = nil
                    } catch { notice = error.localizedDescription }
                }
            }
        } message: { Text("You will need to verify the host key again before connecting.") }
        .confirmationDialog("Remove \(host.title) from this device?", isPresented: $confirmRemove, titleVisibility: .visible) {
            Button("Remove Host", role: .destructive) {
                do { try remove() } catch { notice = error.localizedDescription }
            }
        } message: { Text("This removes its saved sessions from this device. Herdr keeps running on the host.") }
        .alert("Host tags", isPresented: $editingTags) {
            TextField("Work, Personal", text: $tagText)
            Button("Cancel", role: .cancel) {}
            Button("Save") { do { try saveTags(tagText) } catch { notice = error.localizedDescription } }
        } message: { Text("Enter up to eight tags, separated by commas.") }
        .alert("Host Details", isPresented: $notice.presented) { Button("OK") { notice = nil } } message: { Text(notice ?? "") }
    }
}
