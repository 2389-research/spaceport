// ABOUTME: Keeps small excerpts from locally displayed panes in connection memory.
// ABOUTME: Scopes cached output and recent search destinations to a Herdr boot.
import SwiftUI

struct PanePreviewCache {
    struct Entry {
        let pane: String
        let text: String
        let seenAt: Date
        fileprivate let agent: String?
        fileprivate var viewport: String?
    }
    let capacity: Int
    private var boot: String?
    private var entries: [Entry] = []
    private(set) var recent: [String] = []

    init(capacity: Int = 12) { self.capacity = capacity }

    mutating func visit(_ pane: String, boot: String) {
        if self.boot != boot {
            entries.removeAll()
            recent.removeAll()
            self.boot = boot
        }
        recent.removeAll { $0 == pane }
        recent.insert(pane, at: 0)
        if recent.count > capacity { recent.removeLast() }
    }

    mutating func record(_ text: String, pane: String, boot: String, agent: String? = nil, at date: Date = Date()) {
        visit(pane, boot: boot)
        entries.removeAll { $0.pane == pane }
        guard let excerpt = Self.excerpt(text) else { return }
        // Two screens cover A → B → A. Keep only the short Peek excerpt for older panes.
        let viewport = text.utf8.count <= 64 * 1024 ? text : nil
        entries.insert(Entry(pane: pane, text: excerpt, seenAt: date, agent: agent, viewport: viewport), at: 0)
        if entries.count > 2 { entries[2].viewport = nil }
        if entries.count > capacity { entries.removeLast() }
    }

    func lastSeen(pane: String, boot: String) -> Entry? {
        self.boot == boot ? entries.first { $0.pane == pane } : nil
    }

    func viewport(pane: String, boot: String, agent: String? = nil) -> String? {
        guard let entry = lastSeen(pane: pane, boot: boot), entry.agent == agent else { return nil }
        return entry.viewport
    }

    static func excerpt(_ text: String) -> String? {
        let lines = text.split(separator: "\n", omittingEmptySubsequences: false)
        guard let last = lines.lastIndex(where: { $0.contains { !$0.isWhitespace } }) else { return nil }
        return lines[...last].suffix(8).map { line in
            String(decoding: line.utf8.prefix(500), as: UTF8.self).trimmingCharacters(in: .newlines)
        }.joined(separator: "\n")
    }
}

enum PaneExcerpt {
    case visible(String)
    case lastSeen(PanePreviewCache.Entry)
}

extension MobileConnection {
    func preview(pane: String, boot: String) -> PaneExcerpt? {
        guard phase == .ready, let view, view.bootId == boot,
              view.panes.contains(where: { $0.id == pane }) else { return nil }
        if let target = view.focusedPaneTarget, target.paneId == pane {
            let screen = directPane.flatMap { $0.target == target && $0.failure == nil ? $0.surface?.visibleText : nil }
                ?? view.terminal?.visibleText
            if let screen { return PanePreviewCache.excerpt(screen).map(PaneExcerpt.visible) }
        }
        return panePreviews.lastSeen(pane: pane, boot: boot).map(PaneExcerpt.lastSeen)
    }
}

struct AgentPeek: View {
    let row: GoToPane
    let address: SSHAddress
    let read: () -> PaneExcerpt?
    @State private var snapshot: PaneExcerpt?

    var body: some View {
        VStack(alignment: .leading, spacing: 10) {
            AgentSelectorLabel(name: row.pane.agentName ?? row.pane.label, terminalTitle: row.pane.terminalTitle,
                               workspace: row.workspace.label, tab: row.tab.label, font: .headline,
                               paneTitle: row.pane.displayPaneTitle, isAgent: row.pane.agentName != nil, address: address)
            if let activity = PaneActivity(row.pane.status) {
                HStack { activity.dot; Text(activity.title).font(.subheadline) }
            }
            switch snapshot {
            case let .visible(text):
                excerpt(text, label: Text("Visible screen"))
            case let .lastSeen(entry):
                excerpt(entry.text, label: Text("Last seen \(entry.seenAt, format: .relative(presentation: .named))"))
            case nil:
                Text("No screen preview yet.").font(.footnote).foregroundStyle(.secondary)
            }
        }
        .frame(maxWidth: .infinity, alignment: .leading).padding()
        .onAppear { snapshot = read() }
    }

    private func excerpt(_ text: String, label: Text) -> some View {
        VStack(alignment: .leading, spacing: 6) {
            label.font(.caption).foregroundStyle(.secondary)
            Text(verbatim: text).font(.system(.caption, design: .monospaced)).lineLimit(8)
        }
    }
}
