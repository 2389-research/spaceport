// ABOUTME: Carries foreground Shortcuts actions until their SSH session can present them.
// ABOUTME: Adds dictated text to a local draft while retaining its attachment and destination.
import Foundation

enum SessionShortcutAction {
    case revealTerminal, nextWaiting, compose(String)
}

struct SessionShortcutRequest: Identifiable {
    let id = UUID()
    let action: SessionShortcutAction
    let destination: DraftDestination?
    var needsRosterRefresh: Bool
}

extension TerminalDraft {
    mutating func compose(_ addition: String, at current: DraftDestination?) {
        open(at: current)
        guard !addition.isEmpty else { return }
        if !text.isEmpty && !text.hasSuffix("\n") { text += "\n" }
        text += addition
    }
}
