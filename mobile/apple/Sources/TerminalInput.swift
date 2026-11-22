// ABOUTME: Converts native keyboard and paste events to semantic terminal input.
// ABOUTME: Keeps marked text on the device until the input method commits it.
import SwiftUI
import UIKit

struct TerminalInput: UIViewRepresentable {
    let target: InputTarget?
    let keyboard: Bool
    let pointSize: CGFloat
    let ready: Bool
    let send: (MobileConnection.Input, InputTarget) -> Void
    let dismissed: () -> Void

    func makeUIView(context: Context) -> InputView {
        let view = InputView()
        view.backgroundColor = .clear
        view.font = .monospacedSystemFont(ofSize: pointSize, weight: .regular)
        view.textContainerInset = UIEdgeInsets(top: 2, left: 0, bottom: 2, right: 0)
        view.textColor = .white
        view.tintColor = .white
        view.keyboardAppearance = .dark
        view.autocapitalizationType = .none
        view.autocorrectionType = .no
        view.spellCheckingType = .no
        view.smartQuotesType = .no
        view.smartDashesType = .no
        view.smartInsertDeleteType = .no
        view.isScrollEnabled = false
        view.accessibilityLabel = "Terminal input"
        view.accessibilityIdentifier = "Terminal input"
        view.accessibilityHint = "Types directly into the selected terminal."
        if let target { view.attach(to: target, send: send) }
        return view
    }

    func updateUIView(_ view: InputView, context: Context) {
        if view.font?.pointSize != pointSize { view.font = .monospacedSystemFont(ofSize: pointSize, weight: .regular) }
        view.didDismiss = dismissed
        view.accessibilityValue = ready ? "Ready" : "Waiting for pane"
        if keyboard && !view.isFirstResponder && view.window != nil { view.becomeFirstResponder() }
        if !keyboard && view.isFirstResponder { _ = view.resignFirstResponder() }
        view.wantsKeyboard = keyboard
    }

    static func dismantleUIView(_ view: InputView, coordinator: ()) {
        view.invalidate()
    }

    final class InputView: UITextView {
        private var send: ((MobileConnection.Input) -> Void)?
        var wantsKeyboard = false

        func attach(to target: InputTarget, send: @escaping (MobileConnection.Input, InputTarget) -> Void) {
            self.send = { send($0, target) }
        }

        // UIKit may finish marked text after the daemon has selected a different pane.
        func invalidate() {
            send = nil
            didDismiss = nil
            wantsKeyboard = false
            trackpad = nil
            clearComposition()
            if isFirstResponder { _ = resignFirstResponder() }
        }

        override func didMoveToWindow() {
            super.didMoveToWindow()
            if wantsKeyboard && window != nil { becomeFirstResponder() }
        }
        var didDismiss: (() -> Void)?
        private var acceptsInput: Bool { send != nil }
        private var discardingMarkedText = false

        private var trackpad: TerminalTrackpad?
        private var movingMarkedCursor = false

        override func beginFloatingCursor(at point: CGPoint) {
            movingMarkedCursor = markedTextRange != nil
            if movingMarkedCursor { super.beginFloatingCursor(at: point) }
            else if acceptsInput { trackpad = TerminalTrackpad(origin: point) }
        }

        override func updateFloatingCursor(at point: CGPoint) {
            if movingMarkedCursor { super.updateFloatingCursor(at: point) }
            else { trackpad?.move(to: point).forEach { send?(.key($0, 0)) } }
        }

        override func endFloatingCursor() {
            if movingMarkedCursor { super.endFloatingCursor() }
            movingMarkedCursor = false
            trackpad = nil
        }

        override func insertText(_ text: String) {
            // insertText replaces the provisional composition with its final spelling.
            clearComposition()
            guard acceptsInput, !text.isEmpty else { return }
            switch text {
            case "\n", "\r", "\r\n": send?(.key("Enter", 0))
            case "\t": send?(.key("Tab", 0))
            default: send?(.text(text))
            }
        }

        override func unmarkText() {
            guard markedTextRange != nil else { super.unmarkText(); return }
            let committed = text ?? ""
            super.unmarkText()
            self.text = ""
            if acceptsInput && !discardingMarkedText && !committed.isEmpty { send?(.text(committed)) }
        }

        func clearComposition() {
            discardingMarkedText = true
            super.unmarkText()
            text = ""
            discardingMarkedText = false
        }

        override func deleteBackward() {
            if markedTextRange != nil { super.deleteBackward() }
            else if acceptsInput { send?(.key("Backspace", 0)) }
        }

        override func paste(_ sender: Any?) {
            if acceptsInput, let string = UIPasteboard.general.string { send?(.paste(string)) }
        }

        override func resignFirstResponder() -> Bool {
            clearComposition()
            let resigned = super.resignFirstResponder()
            if resigned { DispatchQueue.main.async { [weak self] in self?.didDismiss?() } }
            return resigned
        }

        override func pressesBegan(_ presses: Set<UIPress>, with event: UIPressesEvent?) {
            var remaining = Set<UIPress>()
            for press in presses {
                guard let key = press.key, !key.modifierFlags.contains(.command), markedTextRange == nil else {
                    remaining.insert(press); continue
                }
                let flags = key.modifierFlags
                let modifiers: UInt8 = (flags.contains(.shift) ? 1 : 0) | (flags.contains(.control) ? 2 : 0) | (flags.contains(.alternate) ? 4 : 0)
                let special: String? = switch key.keyCode {
                case .keyboardUpArrow: "Up"
                case .keyboardDownArrow: "Down"
                case .keyboardLeftArrow: "Left"
                case .keyboardRightArrow: "Right"
                case .keyboardEscape: "Esc"
                case .keyboardTab: "Tab"
                case .keyboardDeleteForward: "Delete"
                case .keyboardHome: "Home"
                case .keyboardEnd: "End"
                case .keyboardPageUp: "PageUp"
                case .keyboardPageDown: "PageDown"
                default: nil
                }
                if let special {
                    if acceptsInput { send?(.key(special, modifiers)) }
                } else if flags.contains(.control), key.charactersIgnoringModifiers.count == 1 {
                    if acceptsInput { send?(.key(key.charactersIgnoringModifiers, modifiers)) }
                } else { remaining.insert(press) }
            }
            if !remaining.isEmpty { super.pressesBegan(remaining, with: event) }
        }
    }
}
