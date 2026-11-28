// ABOUTME: Shows read-only matches from the current pane's retained Herdr history.
// ABOUTME: Keeps query input separate from the terminal and copies only an explicit selection.
import SwiftUI
import UIKit

struct TerminalHistoryView: View {
    @ObservedObject var connection: MobileConnection
    let presentation: UUID
    let target: InputTarget
    let done: () -> Void
    @State private var query = ""
    @State private var request: Search?
    @State private var match: HistoryMatch?
    @State private var searched = false
    @State private var busy = false
    @State private var issue: String?
    @State private var copied = false
    @FocusState private var editing: Bool

    private struct Search {
        let id = UUID()
        let query: String
        let newer: Bool
        let previous: HistoryCursor?
    }

    private var isCurrent: Bool {
        connection.acceptsInput(from: presentation) && connection.view?.inputTarget == target
    }

    var body: some View {
        List {
            Section {
                HStack {
                    TextField("Find in terminal history", text: $query)
                        .textInputAutocapitalization(.never).autocorrectionDisabled()
                        .submitLabel(.search).focused($editing)
                        .onSubmit { start() }.accessibilityIdentifier("history.query")
                    Button("Find") { start() }
                        .buttonStyle(.borderless).disabled(query.isEmpty || busy || !isCurrent)
                        .accessibilityIdentifier("history.submit")
                }
            }
            if !isCurrent {
                Text("The focused terminal changed.").foregroundStyle(.secondary)
            } else if let issue {
                Text(issue).foregroundStyle(.secondary)
            } else if busy {
                ProgressView("Searching history…")
            } else if let match {
                Section {
                    Text(excerpt(match))
                        .font(.body.monospaced()).textSelection(.enabled)
                        .accessibilityIdentifier("history.excerpt")
                    HStack {
                        Button("Older", systemImage: "chevron.up") { start(previous: match.cursor) }
                            .accessibilityIdentifier("history.older")
                        Spacer()
                        Button("Newer", systemImage: "chevron.down") { start(newer: true, previous: match.cursor) }
                            .accessibilityIdentifier("history.newer")
                    }.buttonStyle(.borderless).disabled(match.total < 2)
                    Button(copied ? "Copied match" : "Copy match", systemImage: copied ? "checkmark" : "doc.on.doc") {
                        guard isCurrent else { return }
                        UIPasteboard.general.string = match.matched
                        copied = true
                    }.accessibilityIdentifier("history.copy")
                } header: {
                    Text("Match \(match.index) of \(match.total)").accessibilityIdentifier("history.count")
                }
            } else if searched {
                Text("No matches").foregroundStyle(.secondary).accessibilityIdentifier("history.empty")
            }
        }
        .navigationTitle("Find in Terminal").navigationBarTitleDisplayMode(.inline)
        .toolbar {
            ToolbarItem(placement: .confirmationAction) {
                Button("Done", action: done).accessibilityIdentifier("history.done")
            }
        }
        .onAppear { editing = true }
        .onChange(of: query) { _, _ in reset() }
        .onChange(of: isCurrent) { _, current in if !current { reset() } }
        .task(id: request?.id) {
            guard let request else { return }
            do {
                let hit = try await connection.findHistory(request.query, newer: request.newer, previous: request.previous,
                                                           target: target, from: presentation)
                guard self.request?.id == request.id, !Task.isCancelled else { return }
                match = hit
                searched = true
            } catch {
                guard self.request?.id == request.id, !Task.isCancelled else { return }
                switch error as? MobileError {
                case let .Client(message): issue = message
                case .StaleView: issue = "The focused terminal changed."
                case .Disconnected: issue = "The terminal disconnected."
                case .HistoryTimeout: issue = "Herdr took too long to answer. Tap Find to try again."
                case .HistoryBusy: issue = "Herdr is still answering earlier searches. Try again shortly."
                default: issue = "Herdr could not read this match."
                }
            }
            busy = false
        }
    }

    private func excerpt(_ match: HistoryMatch) -> AttributedString {
        var selected = AttributedString(match.matched)
        selected[AttributeScopes.SwiftUIAttributes.BackgroundColorAttribute.self] = .yellow
        selected[AttributeScopes.SwiftUIAttributes.ForegroundColorAttribute.self] = .black
        return AttributedString(match.before) + selected + AttributedString(match.after)
    }

    private func start(newer: Bool = false, previous: HistoryCursor? = nil) {
        guard !busy, isCurrent, !query.isEmpty else { return }
        editing = false
        issue = nil
        copied = false
        busy = true
        request = Search(query: query, newer: newer, previous: previous)
    }

    private func reset() {
        request = nil
        busy = false
        match = nil
        searched = false
        copied = false
        issue = nil
    }
}
