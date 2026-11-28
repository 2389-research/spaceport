// ABOUTME: Resolves pane destinations from the workspace and tab IDs reported by Herdr.
// ABOUTME: Presents agent status consistently throughout the navigation sheet.
import SwiftUI

enum PaneActivity {
    case blocked, working, idle

    init?(_ status: String) {
        switch status {
        case "blocked": self = .blocked
        case "working": self = .working
        case "idle", "done": self = .idle
        default: return nil
        }
    }

    static func rank(_ status: String) -> Int {
        switch Self(status) {
        case .blocked: 1
        case .working: 0
        case .idle: 2
        case nil: 3
        }
    }

    var title: String {
        switch self {
        case .blocked: "Needs you"
        case .working: "Working"
        case .idle: "Idle"
        }
    }

    var color: Color {
        switch self {
        case .blocked: .accentColor
        case .working:
            Color(uiColor: UIColor { traits in
                traits.userInterfaceStyle == .dark
                ? UIColor(red: 100/255, green: 168/255, blue: 1, alpha: 1) : .systemBlue
            })
        case .idle: Color(uiColor: .systemGray)
        }
    }

    var dot: some View { dot(in: color) }

    private func dot(in color: Color) -> some View {
        Circle().fill(self == .idle ? Color.clear : color)
            .overlay { if self == .idle { Circle().strokeBorder(color, lineWidth: 1.5) } }
            .frame(width: 9, height: 9).accessibilityHidden(true)
    }

    func agentDot(in color: Color) -> some View {
        dot(in: color).modifier(AgentDotMotion(activity: self))
    }

    func pulse(reduceMotion: Bool, scene: ScenePhase) -> AgentPulse? {
        guard !reduceMotion, scene == .active else { return nil }
        switch self {
        case .working: return .working
        case .blocked: return .waiting
        case .idle: return nil
        }
    }

    var textColor: Color {
        guard self == .working else { return color }
        return Color(uiColor: UIColor { traits in
            traits.userInterfaceStyle == .dark
                ? UIColor(red: 100/255, green: 168/255, blue: 1, alpha: 1)
                : UIColor(red: 0, green: 98/255, blue: 204/255, alpha: 1)
        })
    }
}

enum AgentPulse {
    case working, waiting

    @KeyframesBuilder<Double> var opacity: some Keyframes<Double> {
        if self == .working {
            LinearKeyframe(0.52, duration: 1.2, timingCurve: .easeInOut)
            LinearKeyframe(1, duration: 1.2, timingCurve: .easeInOut)
        } else {
            LinearKeyframe(0.58, duration: 0.22, timingCurve: .easeInOut)
            LinearKeyframe(1, duration: 0.22, timingCurve: .easeInOut)
            LinearKeyframe(0.58, duration: 0.28, timingCurve: .easeInOut)
            LinearKeyframe(1, duration: 0.22, timingCurve: .easeInOut)
            LinearKeyframe(1, duration: 2.5)
        }
    }
}

private struct AgentDotMotion: ViewModifier {
    let activity: PaneActivity
    @Environment(\.accessibilityReduceMotion) private var reduceMotion
    @Environment(\.scenePhase) private var scene

    func body(content: Content) -> some View {
        if let pulse = activity.pulse(reduceMotion: reduceMotion, scene: scene) {
            content.keyframeAnimator(initialValue: 1.0) { dot, opacity in
                dot.opacity(opacity)
            } keyframes: { _ in pulse.opacity }
            .id(pulse)
        } else {
            content
        }
    }
}

extension MobilePane {
    var displayPaneTitle: String? {
        guard !label.isEmpty, label != id, label != agentName, label != terminalTitle else { return nil }
        return label
    }
}

struct GoToPane: Identifiable, Equatable {
    let workspace: MobileWorkspace
    let tab: MobileTab
    let pane: MobilePane
    var id: String { pane.id }
    var location: String { [workspace.label, tab.label].joined(separator: " › ") }

    var sidebarAge: String? {
        switch PaneActivity(pane.status) {
        case .blocked: return "now"
        case .working: return "live"
        case .idle:
            let now = HomeAgent.milliseconds(.now)
            guard let first = pane.idleFirstObservedMs,
                  idleAgeGroup(firstObservedMs: first, onsetObserved: pane.idleOnsetObserved, confirmedAtMs: now) != .unplaced
            else { return nil }
            return HomeAgent.idleLabel(elapsedMs: now >= first ? now - first : 0, observedOnset: pane.idleOnsetObserved)
        case nil: return nil
        }
    }

    static func agents(workspaces: [MobileWorkspace], tabs: [MobileTab], panes: [MobilePane]) -> [Self] {
        destinations(workspaces: workspaces, tabs: tabs, panes: panes).filter { $0.pane.agentName != nil }.sorted { lhs, rhs in
            let left = PaneActivity.rank(lhs.pane.status), right = PaneActivity.rank(rhs.pane.status)
            if left != right { return left < right }
            guard PaneActivity(lhs.pane.status) == .idle else { return false }
            return (lhs.pane.stateChangeSeq ?? 0) > (rhs.pane.stateChangeSeq ?? 0)
        }
    }

    static func search(_ query: String, workspaces: [MobileWorkspace], tabs: [MobileTab], panes: [MobilePane],
                       recent: [String]) -> [Self] {
        func folded(_ text: String) -> String {
            text.folding(options: [.caseInsensitive, .diacriticInsensitive], locale: .current)
        }
        let words = folded(query).split(whereSeparator: \.isWhitespace)
        guard !words.isEmpty else { return [] }
        return destinations(workspaces: workspaces, tabs: tabs, panes: panes).compactMap { row -> (Self, Int)? in
            let names = [row.pane.terminalTitle, row.pane.agentName, row.pane.label].compactMap { $0 }.map(folded)
            let path = folded(row.location)
            var score = 0
            for word in words {
                if names.contains(String(word)) { score += 3 }
                else if names.contains(where: { $0.hasPrefix(word) }) { score += 2 }
                else if names.contains(where: { $0.contains(word) }) { score += 1 }
                else if !path.contains(word) { return nil }
            }
            return (row, score)
        }.sorted { left, right in
            if left.1 != right.1 { return left.1 > right.1 }
            return (recent.firstIndex(of: left.0.id) ?? Int.max) < (recent.firstIndex(of: right.0.id) ?? Int.max)
        }.map(\.0)
    }

    private static func destinations(workspaces: [MobileWorkspace], tabs: [MobileTab], panes: [MobilePane]) -> [Self] {
        let byWorkspace = Dictionary(workspaces.map { ($0.id, $0) }, uniquingKeysWith: { first, _ in first })
        let byTab = Dictionary(tabs.map { ($0.id, $0) }, uniquingKeysWith: { first, _ in first })
        return panes.compactMap { pane -> Self? in
            guard let tab = byTab[pane.tabId], let workspace = byWorkspace[tab.workspaceId] else { return nil }
            return Self(workspace: workspace, tab: tab, pane: pane)
        }
    }

    func isFocused(in view: MobileView) -> Bool {
        workspace.id == view.focusedWorkspace && tab.id == view.focusedTab && pane.id == view.focusedPane
    }
}

extension MobileView {
    var focusedPaneTarget: InputTarget? {
        guard !goToPending, let target = inputTarget, target.popupId == nil,
              target.bootId == bootId, target.paneId == focusedPane else { return nil }
        return target
    }

    func confirmsInput(to pane: String, revision: UInt64) -> Bool {
        goToArrival == revision && focusedPaneTarget?.paneId == pane
    }
}
