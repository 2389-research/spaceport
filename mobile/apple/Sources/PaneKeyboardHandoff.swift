// ABOUTME: Holds the system keyboard while Herdr confirms the next pane.
// ABOUTME: Input during that gap has no terminal destination.
import SwiftUI

final class PaneKeyboardHandoff: UIView, ObservableObject, UIKeyInput {
    var hasText: Bool { false }
    func insertText(_ text: String) {}
    func deleteBackward() {}
    override var canBecomeFirstResponder: Bool { true }

    func hold(from terminal: PhoneGhosttyView) -> Bool {
        guard terminal.wantsKeyboard, terminal.isFirstResponder, let window = terminal.window else { return false }
        accessibilityElementsHidden = true
        window.addSubview(self)
        // The outgoing view must not report this intentional transfer as a user dismissal.
        terminal.wantsKeyboard = false
        guard becomeFirstResponder() else {
            cancel()
            return false
        }
        return true
    }

    func cancel() {
        if isFirstResponder { _ = resignFirstResponder() }
        removeFromSuperview()
    }

    override func resignFirstResponder() -> Bool {
        let resigned = super.resignFirstResponder()
        if resigned {
            // Let UIKit finish the transfer to the new Ghostty view before detaching.
            Task { @MainActor in
                if !self.isFirstResponder { self.removeFromSuperview() }
            }
        }
        return resigned
    }
}
