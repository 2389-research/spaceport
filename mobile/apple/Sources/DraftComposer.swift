// ABOUTME: Presents a multiline editor for a session's unsent terminal draft.
// ABOUTME: Checks the live destination before queuing a paste and hides inactive content.
import SwiftUI
import UIKit
import UniformTypeIdentifiers

struct DraftComposer: View {
    @Binding var draft: TerminalDraft
    @ObservedObject var connection: MobileConnection
    let presentation: UUID
    let inline: Bool
    let close: () -> Void
    @Environment(\.scenePhase) private var scenePhase
    @StateObject private var editing = DraftEditing()
    @State private var notice: String?
    @State private var choosingDestination = false
    @State private var operation: Task<Void, Never>?
    @State private var choosingFile = false
    @State private var sharedFile: URL?
    @State private var transfer: MobileFileTransfer?
    @AppStorage(TerminalPreferences.composeReturn) private var pressReturn = TerminalPreferences.defaultComposeReturn

    var body: some View {
        if scenePhase == .active {
            VStack(spacing: 0) {
                if !inline {
                    Capsule().fill(.secondary.opacity(0.6))
                        .frame(width: 36, height: 5).padding(.vertical, 8)
                        .frame(maxWidth: .infinity).contentShape(Rectangle())
                        .gesture(DragGesture().onEnded { drag in
                            if drag.translation.height > 44 { close() }
                        })
                        .accessibilityHidden(true)
                }
                HStack {
                    Text("Draft").font(.headline).accessibilityAddTraits(.isHeader)
                        .accessibilityIdentifier(inline ? "draft.inline" : "draft.sheet")
                    Spacer()
                    Button(transfer == nil ? "Close" : "Cancel upload") { operation?.cancel(); close() }
                        .frame(minWidth: 44, minHeight: 44)
                        .accessibilityIdentifier("draft.done")
                }
                .padding(.horizontal, 12)
                VStack(alignment: .leading, spacing: 6) {
                    let current = connection.draftDestination
                    Text(verbatim: "\(connection.profile.hostname) · \(connection.profile.session)")
                        .font(.caption).foregroundStyle(.secondary)
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
                        .padding(.horizontal, 12).frame(minHeight: 44)
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
                    DraftEditor(text: $draft.text, editing: editing, attach: attach)
                        .privacySensitive()
                    HStack {
                        if let file = draft.file {
                            Label(file.name, systemImage: "doc").lineLimit(1)
                            Spacer()
                            Button("Remove", systemImage: "xmark.circle.fill") { draft.file = nil }
                                .labelStyle(.iconOnly).accessibilityIdentifier("draft.removeFile")
                        } else if let image = draft.image {
                            Image(decorative: image.thumbnail, scale: 1)
                                .resizable().scaledToFit().frame(maxWidth: 68, maxHeight: 60)
                                .clipShape(RoundedRectangle(cornerRadius: 6)).privacySensitive()
                            VStack(alignment: .leading) {
                                Text("Screenshot").font(.subheadline)
                                Text("Uploads to the host, then pastes its path.").font(.caption).foregroundStyle(.secondary)
                            }
                            Spacer()
                            Button("Remove screenshot", systemImage: "xmark.circle.fill") { draft.image = nil }
                                .labelStyle(.iconOnly).accessibilityIdentifier("draft.removeImage")
                        } else {
                            Button("File", systemImage: "paperclip") { choosingFile = true }
                                .accessibilityIdentifier("draft.chooseFile")
                            Spacer()
                            PasteButton(supportedContentTypes: [.image], payloadAction: attach)
                                .accessibilityLabel("Paste screenshot").accessibilityIdentifier("draft.pasteImage")
                        }
                    }
                    .padding(.horizontal, 12).frame(minHeight: 44)
                    if let sharedFile, draft.image == nil, draft.file == nil {
                        HStack {
                            Button("Attach shared file") { loadFile(sharedFile, shared: true) }
                            Spacer()
                            Button("Discard share", role: .destructive) {
                                do { try SharedDraftInbox.remove(sharedFile); self.sharedFile = nil }
                                catch { notice = error.localizedDescription }
                            }
                        }
                    }
                    if let transfer, let file = draft.file {
                        TimelineView(.periodic(from: .now, by: 0.25)) { _ in
                            ProgressView(value: Double(transfer.bytesSent()), total: Double(max(1, file.data.count)))
                                .accessibilityLabel("Uploading file")
                        }
                    } else if operation != nil { ProgressView().accessibilityLabel("Preparing attachment") }
                    HStack(spacing: 10) {
                        Toggle(TerminalPreferences.pressReturnLabel, isOn: $pressReturn)
                            .font(.footnote).tint(.green)
                            .accessibilityIdentifier("draft.pressReturn")
                        sendButton
                    }
                    .frame(minHeight: 44)
                }
                .disabled(operation != nil)
                .padding(10)
                .background(Color(uiColor: .systemGroupedBackground))
                .sheet(isPresented: $choosingDestination) {
                    GoToSheet(connection: connection, presentation: presentation, initialSection: .agents, onArrival: {
                        if let destination = connection.draftDestination { draft.retarget(to: destination) }
                    })
                }
            }
            .background(Color(uiColor: .systemGroupedBackground))
            .accessibilityAction(.escape) { close() }
            .clipShape(RoundedRectangle(cornerRadius: inline ? 0 : 18))
            .onAppear { sharedFile = SharedDraftInbox.directory.flatMap { SharedDraftInbox.pending(in: $0) } }
            .fileImporter(isPresented: $choosingFile, allowedContentTypes: [.item]) { result in
                switch result {
                case .success(let url): loadFile(url, shared: false)
                case .failure(let error): notice = error.localizedDescription
                }
            }
            .onDisappear { operation?.cancel() }
        } else {
            Color(uiColor: .systemBackground).ignoresSafeArea()
        }
    }
    private var sendButton: some View {
        Button(pressReturn ? "Send" : "Paste") {
            guard scenePhase == .active, operation == nil else { return }
            guard let text = editing.committedText else {
                notice = "Finish composing the current word before pasting."
                return
            }
            draft.text = text
            if draft.image != nil || draft.file != nil {
                operation = Task {
                    let transfer = MobileFileTransfer()
                    self.transfer = transfer
                    defer { operation = nil; self.transfer = nil }
                    do {
                        try await connection.sendDraftAttachment(from: presentation, pressReturn: pressReturn, transfer: transfer)
                        close()
                    } catch MobileError.ImageRetentionUnconfirmed {
                        notice = "Couldn’t confirm the attachment on the host. A temporary copy may remain. Your draft is still here."
                    } catch MobileError.StaleView {
                        notice = "The destination changed. Check it before sending your draft."
                    } catch {
                        notice = "Couldn’t paste the attachment. Your draft is still here."
                    }
                }
            } else if draft.send(to: connection.view?.inputTarget, enqueuePaste: { text, target in
                connection.send(.paste(text, pressReturn: pressReturn), to: target, from: presentation)
            }) {
                close()
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

    private func loadFile(_ url: URL, shared: Bool) {
        guard operation == nil else { return }
        let destination = draft.destination
        operation = Task {
            defer { operation = nil }
            do {
                let file = try await Task.detached(priority: .userInitiated) {
                    try shared ? SharedDraftInbox.read(url) : DraftFile(url: url)
                }.value
                try Task.checkCancellation()
                guard draft.destination == destination else { throw MobileError.StaleView }
                if shared { try SharedDraftInbox.remove(url); sharedFile = nil }
                draft.image = nil
                draft.file = file
                notice = nil
            } catch { notice = error.localizedDescription }
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
                draft.file = nil
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
        font = UIFontMetrics(forTextStyle: .body).scaledFont(for: .monospacedSystemFont(ofSize: 15, weight: .regular))
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
