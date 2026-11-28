// ABOUTME: Hosts one Ghostty terminal for one direct Herdr attachment.
// ABOUTME: Keeps keyboard focus, terminal geometry, and input tied to that surface.
import GhosttyTerminal
import SwiftUI
import UIKit

@MainActor
final class GhosttyPaneSurface: TerminalSurfaceOpenURLHandlingDelegate, TerminalSurfaceColorChangeDelegate {
    @Published private(set) var reportedAppearance: TerminalAppearance?

    func terminalDidChangeColor(_ change: TerminalColorChange) {
        guard change.kind == .background else { return }
        let appearance = TerminalAppearance(background: change.color)
        if appearance != reportedAppearance { reportedAppearance = appearance }
    }

    // Each attachment owns a separate terminal grid. The font and Metal runtime are shared.
    static let controller = TerminalController(theme: (TerminalAppearance(rawValue:
        UserDefaults.standard.string(forKey: TerminalAppearance.preference) ?? "") ?? .dark).theme) { config in
        config.withWindowPaddingX(0)
        config.withWindowPaddingY(0)
        config.withCustom("clipboard-read", "deny")
        config.withCustom("clipboard-write", "deny")
        config.withCustom("desktop-notifications", "false")
        config.withCustom("image-storage-limit", "0")
        config.withCustom("link-previews", "false")
    }

    private static let keys: [String: GhosttyTerminal.TerminalKey] = [
        "Esc": .escape, "Escape": .escape, "Tab": .tab, "Enter": .enter, "Backspace": .backspace,
        "Delete": .delete, "Up": .arrowUp, "Down": .arrowDown, "Left": .arrowLeft, "Right": .arrowRight,
        "Home": .home, "End": .end, "PageUp": .pageUp, "PageDown": .pageDown,
    ]

    private let textSize: Int
    private let session: InMemoryTerminalSession
    private let write: @Sendable (Data) -> Bool
    private weak var nativeView: PhoneGhosttyView?

    init(write: @escaping @Sendable (Data) -> Bool, resize: @escaping @Sendable (Viewport) -> Void,
         textSize: Int? = nil) {
        self.write = write
        self.textSize = textSize ?? TerminalPreferences.defaultTextSize
        session = InMemoryTerminalSession(write: { _ = write($0) }, resize: { grid in
            guard grid.columns > 0, grid.rows > 0 else { return }
            resize(Viewport(columns: grid.columns, rows: grid.rows,
                            cellWidth: grid.cellWidthPixels, cellHeight: grid.cellHeightPixels))
        })
    }

    fileprivate func attach(to view: PhoneGhosttyView) {
        nativeView = view
        view.readScreen = { [weak self] in self?.visibleText }
        view.isAccessibilityElement = true
        view.accessibilityLabel = "Terminal display"
        view.accessibilityIdentifier = "terminal.ghostty"
        view.delegate = self
        view.configuration = TerminalSurfaceOptions(backend: .inMemory(session), fontSize: Float(textSize))
        view.controller = Self.controller
        view.inputAccessoryItems = []
    }

    func terminalShouldHandleOpenURL(_ url: String, kind: TerminalOpenURLKind) -> Bool {
        // Consume the request here so an untrusted hyperlink never reaches a system opener.
        true
    }

    var visibleText: String? { session.readViewportText() }

    func holdKeyboard(using handoff: PaneKeyboardHandoff) -> Bool {
        guard let nativeView else { return false }
        return handoff.hold(from: nativeView)
    }

    func receive(_ data: Data, whenParsed: (@MainActor @Sendable () -> Void)? = nil) {
        if let whenParsed {
            // A fresh native view can still be mounting. Ghostty drains buffered bytes after it attaches.
            session.setOutputBacklogHandler(highWater: 1, lowWater: 0) { [weak self] waiting in
                guard !waiting else { return }
                Task { @MainActor in
                    guard let self else { return }
                    self.session.setOutputBacklogHandler(highWater: 1, lowWater: 0, nil)
                    whenParsed()
                }
            }
        }
        session.receive(data)
    }

    @discardableResult
    func sendKey(_ key: String, modifiers: UInt8) -> Bool {
        let flags = TerminalInputModifiers(rawValue: UInt32(modifiers & 7))
        guard let nativeView else { return false }
        if let physical = Self.keys[key] { return nativeView.sendKey(physical, modifiers: flags) }
        guard key.count == 1, let character = key.first,
              let press = TerminalKeyPress(typing: character, modifiers: flags) else { return false }
        return nativeView.sendKey(press)
    }

    @discardableResult
    func paste(_ text: String, pressReturn: Bool = false) -> Bool {
        // Herdr direct attach enables bracketed paste in terminal_setup before accepting input.
        // One PTY write keeps Return with its draft and reports the actual queue result.
        let overhead = 12 + (pressReturn ? 1 : 0)
        guard !text.isEmpty, text.utf8.count <= 64 * 1024 - overhead,
              text.unicodeScalars.allSatisfy({ scalar in
                  let code = scalar.value
                  return code == 9 || code == 10 || code == 13 || (code >= 32 && !(127...159).contains(code))
              }) else { return false }
        var bytes = Data("\u{1b}[200~".utf8)
        bytes.append(contentsOf: text.utf8)
        bytes.append(contentsOf: "\u{1b}[201~".utf8)
        if pressReturn { bytes.append(13) }
        return write(bytes)
    }
}

final class PhoneGhosttyView: UITerminalView {
    let agentSwipe = AgentListSwipe()
    var readScreen: (() -> String?)?
    var shiftArmed = false
    var updatingModifiers = false
    var keyboardTapped: (() -> Void)?
    var keyboardDismissed: (() -> Void)?
    var wantsKeyboard = false {
        didSet { if wantsKeyboard != oldValue { updateKeyboard() } }
    }

    override func didMoveToWindow() {
        super.didMoveToWindow()
        if window != nil { updateKeyboard() }
    }

    private func updateKeyboard() {
        DispatchQueue.main.async { [weak self] in
            guard let self, window != nil else { return }
            if wantsKeyboard && !isFirstResponder { becomeFirstResponder() }
            if !wantsKeyboard && isFirstResponder { _ = resignFirstResponder() }
        }
    }

    override func resignFirstResponder() -> Bool {
        let result = super.resignFirstResponder()
        if result && wantsKeyboard {
            DispatchQueue.main.async { [weak self] in
                guard let self, window != nil, !isFirstResponder, wantsKeyboard else { return }
                keyboardDismissed?()
            }
        }
        return result
    }
    var didSpendShift: (() -> Void)?

    override var accessibilityValue: String? {
        get { readScreen?() ?? super.accessibilityValue }
        set { super.accessibilityValue = newValue }
    }

    override func toggleSoftwareKeyboard() {
        if let keyboardTapped { keyboardTapped() }
        else { super.toggleSoftwareKeyboard() }
    }

    var modifierFlags: UInt8 {
        (shiftArmed ? 1 : 0) | (stickyActivation(for: .ctrl) == .inactive ? 0 : 2)
            | (stickyActivation(for: .alt) == .inactive ? 0 : 4)
    }

    func setModifiers(_ flags: UInt8) {
        guard flags != modifierFlags else { return }
        updatingModifiers = true
        defer { updatingModifiers = false }
        resetStickyModifiers()
        shiftArmed = flags & 1 != 0
        if flags & 2 != 0 { toggleStickyModifier(.ctrl) }
        if flags & 4 != 0 { toggleStickyModifier(.alt) }
    }

    override func insertText(_ text: String) {
        guard shiftArmed, markedTextRange == nil, text.count == 1, let char = text.first,
              let key = TerminalKeyPress(typing: char, modifiers: [.shift]) else {
            super.insertText(text)
            return
        }
        shiftArmed = false
        _ = sendKey(key)
        didSpendShift?()
    }
}

struct GhosttyPaneView: UIViewRepresentable {
    let surface: GhosttyPaneSurface
    @Binding var keyboard: Bool
    @Binding var modifiers: UInt8
    var openAgents: (() -> Void)? = nil

    func makeUIView(context: Context) -> PhoneGhosttyView {
        let view = PhoneGhosttyView(frame: .zero)
        view.agentSwipe.canStart = { [weak view] in
            view?.canPerformAction(#selector(UIResponderStandardEditActions.copy(_:)), withSender: nil) == false
        }
        view.addGestureRecognizer(view.agentSwipe)
        surface.attach(to: view)
        return view
    }

    func updateUIView(_ view: PhoneGhosttyView, context: Context) {
        view.agentSwipe.openAgents = openAgents
        view.keyboardTapped = { keyboard.toggle() }
        view.keyboardDismissed = { keyboard = false }
        view.wantsKeyboard = keyboard
        view.didSpendShift = { [weak view] in modifiers = view?.modifierFlags ?? 0 }
        view.setStickyModifierChangeHandler { [weak view] in
            guard let view, !view.updatingModifiers, modifiers != view.modifierFlags else { return }
            modifiers = view.modifierFlags
        }
        view.setModifiers(modifiers)
    }

    static func dismantleUIView(_ view: PhoneGhosttyView, coordinator: ()) {
        view.agentSwipe.openAgents = nil
        view.keyboardTapped = nil
        view.keyboardDismissed = nil
        view.didSpendShift = nil
        view.setStickyModifierChangeHandler(nil)
        view.wantsKeyboard = false
        view.setSurfaceVisible(false)
    }
}
