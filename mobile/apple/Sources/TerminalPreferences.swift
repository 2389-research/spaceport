// ABOUTME: Stores the terminal keyboard layout and the app's terminal preferences.
// ABOUTME: Resolves configured keys to the commands and modifiers Herdr accepts.
import Foundation

enum TerminalPreferences {
    static let toolbarKeys = "terminalToolbarKeys"
    static let textSize = "terminalTextSize"
    static let textRange = 9...24
    static let defaultTextSize = 12
    static let composeReturn = "composePressReturn"
    static let defaultComposeReturn = true
    static let pressReturnLabel = "Press Return After Pasting"
}

enum TerminalKey: String, CaseIterable {
    case ctrl, shift, alt, esc, tab, shiftTab, enter, backspace, delete
    case up, down, left, right, home, end, pageUp, pageDown

    var title: String {
        switch self {
        case .ctrl: "Control"
        case .esc: "Escape"
        case .shiftTab: "Shift Tab"
        case .up, .down, .left, .right: "\(rawValue.capitalized) Arrow"
        case .pageUp: "Page Up"
        case .pageDown: "Page Down"
        default: rawValue.capitalized
        }
    }

    var keycap: String {
        switch self {
        case .ctrl: "Ctrl"
        case .esc: "Esc"
        case .shiftTab: "⇧ Tab"
        case .backspace: "⌫"
        case .delete: "Del"
        case .up: "↑"
        case .down: "↓"
        case .left: "←"
        case .right: "→"
        case .pageUp: "PgUp"
        case .pageDown: "PgDn"
        default: title
        }
    }

    enum Action: Equatable {
        case arm(UInt8)
        case key(String, UInt8)
    }

    var action: Action {
        switch self {
        case .shift: .arm(1)
        case .ctrl: .arm(2)
        case .alt: .arm(4)
        case .shiftTab: .key("Tab", 1)
        case .pageUp: .key("PageUp", 0)
        case .pageDown: .key("PageDown", 0)
        default: .key(rawValue.capitalized, 0)
        }
    }
}

struct TerminalKeyPages: RawRepresentable, Equatable {
    private(set) var pages: [[TerminalKey]]
    static let standard = Self(pages: [[.ctrl, .esc, .tab], [.shiftTab, .shift, .alt]])
    private init(pages: [[TerminalKey]]) { self.pages = pages }
    init?(rawValue: String) {
        var pages = [[TerminalKey]]()
        for encoded in rawValue.split(separator: "|", omittingEmptySubsequences: false) {
            let slots = encoded.split(separator: ",", omittingEmptySubsequences: false)
            let keys = slots.compactMap { TerminalKey(rawValue: String($0)) }
            guard keys.count == 3, slots.count == keys.count else { return nil }
            pages.append(keys)
        }
        guard !pages.isEmpty else { return nil }
        self.pages = pages
    }

    var compactMenuKeys: [TerminalKey] {
        var seen: Set<TerminalKey> = [.esc]
        return pages.joined().filter { seen.insert($0).inserted }
    }

    var rawValue: String {
        pages.map { $0.map(\.rawValue).joined(separator: ",") }.joined(separator: "|")
    }

    mutating func replace(page: Int, slot: Int, with key: TerminalKey) {
        guard pages.indices.contains(page), pages[page].indices.contains(slot) else { return }
        pages[page][slot] = key
    }

    mutating func addPage() { pages.append([.left, .enter, .right]) }

    mutating func removePage(_ index: Int) {
        if pages.count > 1, pages.indices.contains(index) { pages.remove(at: index) }
    }
}
