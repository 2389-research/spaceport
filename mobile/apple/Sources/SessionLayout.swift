// ABOUTME: Fits the agent sidebar and keyboard controls to the usable session width.
// ABOUTME: Reserves enough space for terminal input as the window changes size.
import SwiftUI

enum SessionLayout {
    enum Sidebar: Equatable {
        case full(CGFloat), rail, hidden
        func transitionAnimation(reduceMotion: Bool) -> Animation? {
            guard self != .hidden, !reduceMotion else { return nil }
            return .smooth(duration: 0.28)
        }

        var width: CGFloat {
            switch self {
            case let .full(width): width
            case .rail: 64
            case .hidden: 0
            }
        }
    }

    enum KeyRow { case flat, paged, compact }
    static let composerWidth: CGFloat = 360

    static func sidebarWidth(in width: CGFloat, regular: Bool) -> CGFloat? {
        guard regular, width >= 620 else { return nil }
        return min(280, max(248, width * 0.30))
    }

    static func sidebar(width: CGFloat, regular: Bool, keyboard: Bool, composing: Bool, userCollapsed: Bool) -> Sidebar {
        guard !composing, let full = sidebarWidth(in: width, regular: regular) else { return .hidden }
        return keyboard || userCollapsed ? .rail : .full(full)
    }

    static func keyRow(width: CGFloat, keyCount: Int) -> KeyRow {
        // The toggle, horizontal padding and gap take 68 points. A page needs
        // three 44-point targets and eight points of padding.
        let available = width - 68
        if CGFloat(keyCount) * 44 <= available { return .flat }
        return available >= 140 ? .paged : .compact
    }
}

// Herdr sequences order simultaneous arrivals. A first snapshot establishes a baseline.
struct WaitingInterrupt {
    private var boot: String?
    private var previous: [String: UInt64] = [:]
    private(set) var pane: String?

    mutating func update(boot: String, waiting: [String: UInt64], focused: String?, typing: Bool) {
        defer { self.boot = boot; previous = waiting }
        guard self.boot == boot, typing else { pane = nil; return }
        if let pane, pane == focused || waiting[pane] == nil { self.pane = nil }
        let arrivals = waiting.filter { $0.key != focused && previous[$0.key] == nil }
        if let latest = arrivals.max(by: { ($0.value, $0.key) < ($1.value, $1.key) }) {
            pane = latest.key
        }
    }

    mutating func dismiss() { pane = nil }
}
