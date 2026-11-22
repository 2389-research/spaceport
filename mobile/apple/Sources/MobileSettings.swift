// ABOUTME: Presents device identity, terminal preferences and the installed app version.
// ABOUTME: Edits the same keyboard pages and Compose preference the session reads.
import SwiftUI

struct MobileSettings: View {
    let hosts: [SavedHost]
    let openHostDetails: (UUID) -> Void
    @AppStorage(TerminalPreferences.textSize) private var savedTextSize: Int?
    @AppStorage(TerminalPreferences.toolbarKeys) private var keys = TerminalKeyPages.standard
    @AppStorage(TerminalPreferences.composeReturn) private var pressReturn = TerminalPreferences.defaultComposeReturn

    @AppStorage(TerminalPreferences.ghostty) private var ghostty = false

    private var textSize: Binding<Int> {
        Binding(get: { TerminalPreferences.textSize(saved: savedTextSize, ghostty: ghostty) },
                set: { savedTextSize = $0 })
    }

    var body: some View {
        Form {
            PhoneKeySettings(hosts: hosts, openHostDetails: openHostDetails)
            Section {
                Stepper(value: textSize, in: TerminalPreferences.textRange) {
                    LabeledContent("Text Size", value: "\(textSize.wrappedValue) pt")
                }
                .accessibilityIdentifier("settings.textSize")
                NavigationLink {
                    ToolbarKeyEditor(keys: $keys)
                } label: {
                    LabeledContent("Toolbar Keys", value: keys.pages.count == 1 ? "1 page" : "\(keys.pages.count) pages")
                }
                .accessibilityIdentifier("settings.toolbarKeys")
                Toggle(TerminalPreferences.pressReturnLabel, isOn: $pressReturn)
                    .tint(.green)
                    .accessibilityIdentifier("settings.composeReturn")
            } header: {
                Text("Terminal")
            } footer: {
                Text("Swipe the terminal keys to change pages. Hold the space bar and drag to move the cursor.")
            }
            Section {
                Toggle("Use Ghostty", isOn: $ghostty)
                    .accessibilityIdentifier("settings.ghostty")
            } header: {
                Text("Terminal Experiment")
            } footer: {
                Text("Use Ghostty to attach directly to the selected terminal over your SSH connection.")
            }
            Section {
                LabeledContent("Version", value: version)
            } header: { Text("About") }
        }
        .textCase(nil).navigationTitle("Settings")
    }

    private var version: String {
        let info = Bundle.main.infoDictionary ?? [:]
        let version = info["CFBundleShortVersionString"] as? String ?? "—"
        let build = info["CFBundleVersion"] as? String ?? "—"
        return "\(version) (\(build))"
    }
}

private struct ToolbarKeyEditor: View {
    @Binding var keys: TerminalKeyPages

    var body: some View {
        Form {
            ForEach(keys.pages.indices, id: \.self) { page in
                Section("Page \(page + 1)") {
                    ForEach(0..<3) { slot in
                        Picker("Key \(slot + 1)", selection: selection(page: page, slot: slot)) {
                            ForEach(TerminalKey.allCases, id: \.self) { key in
                                Text(key.title).tag(key)
                            }
                        }
                        .pickerStyle(.menu)
                        .accessibilityIdentifier("toolbar.page.\(page).slot.\(slot)")
                    }
                    if keys.pages.count > 1 {
                        Button("Delete Page", role: .destructive) { keys.removePage(page) }
                            .accessibilityIdentifier("toolbar.page.\(page).delete")
                    }
                }
            }
            Button("Add Page", systemImage: "plus") { keys.addPage() }
                .accessibilityIdentifier("toolbar.page.add")
        }
        .navigationTitle("Toolbar Keys")
        .navigationBarTitleDisplayMode(.inline)
    }

    private func selection(page: Int, slot: Int) -> Binding<TerminalKey> {
        Binding {
            // A menu can finish its dismissal after its page has left the Form.
            keys.pages.indices.contains(page) ? keys.pages[page][slot] : .esc
        } set: { key in
            keys.replace(page: page, slot: slot, with: key)
        }
    }
}
