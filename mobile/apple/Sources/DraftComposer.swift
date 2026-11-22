// ABOUTME: Presents a multiline editor for a session's unsent terminal draft.
// ABOUTME: Checks the live destination before queuing a paste and hides inactive content.
import SwiftUI
import UIKit
import UniformTypeIdentifiers

struct DraftComposer: View {
    @Binding var draft: TerminalDraft
    @ObservedObject var connection: MobileConnection
    let presentation: UUID
    @Environment(\.dismiss) private var dismiss
    @Environment(\.scenePhase) private var scenePhase
    @StateObject private var editing = DraftEditing()
    @State private var notice: String?
    @State private var choosingDestination = false
    @State private var operation: Task<Void, Never>?
    @AppStorage(TerminalPreferences.composeReturn) private var pressReturn = TerminalPreferences.defaultComposeReturn

    var body: some View {
        if scenePhase == .active {
            NavigationStack {
                VStack(alignment: .leading, spacing: 12) {
                    let current = connection.draftDestination
                    Button { choosingDestination = true } label: {
                        HStack {
                            Text("To").foregroundStyle(.secondary)
                            if let destination = draft.destination, let activity = PaneActivity(destination.status) { activity.dot }
                            VStack(alignment: .leading) {
                                Text(verbatim: draft.destination?.name ?? "Choose a pane").font(.headline)
                                if let destination = draft.destination, !destination.path.isEmpty {
                                    Text(verbatim: destination.path).font(.caption).foregroundStyle(.secondary)
                                }
                            }
                            .frame(maxWidth: .infinity, alignment: .leading)
                            Image(systemName: "chevron.right").font(.caption.weight(.semibold))
                        }
                        .padding(.horizontal, 16).frame(minHeight: 54)
                        .background(Color(uiColor: .secondarySystemGroupedBackground), in: RoundedRectangle(cornerRadius: 18))
                    }
                    .buttonStyle(.plain).accessibilityIdentifier("draft.destination")
                    if let current, current.target != draft.destination?.target {
                        Button("Use current destination: \(current.label)") {
                            draft.retarget(to: current)
                            notice = nil
                        }
                        .accessibilityIdentifier("draft.retarget")
                    } else if current == nil {
                        Text("Waiting for a terminal. You can keep editing.")
                            .foregroundStyle(.secondary)
                    }
                    if let notice {
                        Text(notice)
                            .foregroundStyle(.secondary)
                    }
                    HStack {
                        if let image = draft.image {
                            Image(decorative: image.thumbnail, scale: 1)
                                .resizable().scaledToFit().frame(maxWidth: 68, maxHeight: 60)
                                .clipShape(RoundedRectangle(cornerRadius: 6)).privacySensitive()
                            Text("Screenshot").font(.subheadline)
                            Spacer()
                            Button("Remove screenshot", systemImage: "xmark.circle.fill") { draft.image = nil }
                                .labelStyle(.iconOnly).accessibilityIdentifier("draft.removeImage")
                        } else {
                            Label("Screenshot", systemImage: "photo").font(.subheadline)
                            Spacer()
                            PasteButton(supportedContentTypes: [.image], payloadAction: attach)
                                .accessibilityLabel("Paste screenshot").accessibilityIdentifier("draft.pasteImage")
                        }
                    }
                    .padding(.horizontal, 12).frame(minHeight: 44)
                    if operation != nil { ProgressView().accessibilityLabel("Preparing screenshot") }
                    DraftEditor(text: $draft.text, editing: editing, attach: attach)
                        .privacySensitive()
                    Toggle(TerminalPreferences.pressReturnLabel, isOn: $pressReturn)
                        .tint(.green)
                        .padding(.horizontal, 16).frame(minHeight: 52)
                        .background(Color(uiColor: .secondarySystemGroupedBackground), in: RoundedRectangle(cornerRadius: 18))
                        .accessibilityIdentifier("draft.pressReturn")
                    Text(pressReturn ? "Send pastes the draft and presses Return." : "Paste leaves the text in the terminal for you to review.")
                        .font(.footnote).foregroundStyle(.secondary).padding(.horizontal, 8)
                }
                .disabled(operation != nil)
                .padding()
                .background(Color(uiColor: .systemGroupedBackground))
                .navigationTitle("Compose")
                .navigationBarTitleDisplayMode(.inline)
                .sheet(isPresented: $choosingDestination) {
                    GoToSheet(connection: connection, presentation: presentation, initialSection: .agents)
                }
                .toolbar {
                    ToolbarItem(placement: .cancellationAction) {
                        Button("Cancel", systemImage: "xmark") { operation?.cancel(); dismiss() }.labelStyle(.iconOnly)
                            .accessibilityIdentifier("draft.done")
                    }
                    ToolbarItem(placement: .confirmationAction) {
                        Button(pressReturn ? "Send" : "Paste") {
                            guard scenePhase == .active, operation == nil else { return }
                            guard let text = editing.committedText else {
                                notice = "Finish composing the current word before pasting."
                                return
                            }
                            draft.text = text
                            if draft.image != nil {
                                operation = Task {
                                    defer { operation = nil }
                                    do {
                                        try await connection.sendDraftImage(from: presentation, pressReturn: pressReturn)
                                        dismiss()
                                    } catch MobileError.ImageRetentionUnconfirmed {
                                        notice = "Couldn’t confirm the screenshot on the host. A temporary copy may remain. Your draft is still here."
                                    } catch MobileError.StaleView {
                                        notice = "The destination changed. Check it before sending your draft."
                                    } catch {
                                        notice = "Couldn’t paste the screenshot. Your draft is still here."
                                    }
                                }
                            } else if draft.send(to: connection.view?.inputTarget, enqueuePaste: { text, target in
                                connection.send(.paste(text, pressReturn: pressReturn), to: target, from: presentation)
                            }) {
                                dismiss()
                            } else {
                                notice = "Couldn’t queue this draft. Your text is still here."
                            }
                        }
                        .buttonStyle(.borderedProminent).buttonBorderShape(.capsule)
                        .disabled(operation != nil || !draft.canSend(to: connection.view?.inputTarget))
                        .accessibilityHint(pressReturn ? "Pastes the draft and presses Return in the selected terminal." :
                                                        "Pastes the draft into the selected terminal.")
                        .accessibilityIdentifier("draft.send")
                    }
                }
            }
            .onDisappear { operation?.cancel() }
            .preferredColorScheme(.dark)
            .presentationCornerRadius(34)
            .presentationDragIndicator(.visible)
        } else {
            Color(uiColor: .systemBackground).ignoresSafeArea()
        }
    }
    private func attach(_ providers: [NSItemProvider]) {
        guard operation == nil, let provider = providers.first else { return }
        let target = draft.destination?.target
        notice = nil
        operation = Task {
            defer { operation = nil }
            do {
                let image = try await DraftImage.load(from: provider)
                try Task.checkCancellation()
                guard target == draft.destination?.target else { return }
                draft.image = image
            } catch {
                notice = error.localizedDescription
            }
        }
    }

}

private struct DraftEditor: UIViewRepresentable {
    @Binding var text: String
    let editing: DraftEditing
    let attach: ([NSItemProvider]) -> Void

    func makeUIView(context: Context) -> DraftTextView {
        let editor = DraftTextView(content: $text)
        editor.attachImage = attach
        editing.editor = editor
        return editor
    }

    func updateUIView(_ editor: DraftTextView, context: Context) {
        editor.attachImage = attach
        editor.display(text)
    }
}

@MainActor
final class DraftEditing: ObservableObject {
    weak var editor: DraftTextView?

    var committedText: String? { editor?.committedText }
}

final class DraftTextView: UITextView, UITextViewDelegate {
    private let content: Binding<String>
    var attachImage: (([NSItemProvider]) -> Void)?

    override func paste(_ sender: Any?) {
        if let attachImage, UIPasteboard.general.hasImages {
            attachImage(UIPasteboard.general.itemProviders)
        } else {
            super.paste(sender)
        }
    }

    init(content: Binding<String>) {
        self.content = content
        super.init(frame: .zero, textContainer: nil)
        font = .preferredFont(forTextStyle: .body)
        adjustsFontForContentSizeCategory = true
        autocapitalizationType = .none
        autocorrectionType = .no
        spellCheckingType = .no
        smartQuotesType = .no
        smartDashesType = .no
        smartInsertDeleteType = .no
        accessibilityLabel = "Draft"
        accessibilityIdentifier = "draft.editor"
        backgroundColor = .secondarySystemBackground
        layer.cornerRadius = 18
        textContainerInset = UIEdgeInsets(top: 14, left: 12, bottom: 14, right: 12)
        delegate = self
        display(content.wrappedValue)
    }

    @available(*, unavailable)
    required init?(coder: NSCoder) { fatalError("Create the draft editor with its text binding.") }

    var committedText: String? { markedTextRange == nil ? text : nil }

    func display(_ draft: String) {
        if markedTextRange == nil, text != draft { text = draft }
    }

    func textViewDidChange(_ textView: UITextView) {
        if let committedText { content.wrappedValue = committedText }
    }
}
