// ABOUTME: Keeps live agent status and confirmed pane navigation within thumb reach while typing.
// ABOUTME: Shares the attention-first roster and colors with the expanded session sidebar.
import SwiftUI

enum SessionChrome {
    static let terminal = Color(uiColor: UIColor { $0.userInterfaceStyle == .dark ? UIColor(white: 0.055, alpha: 1) : UIColor(white: 0.97, alpha: 1) })
    static let sidebar = Color(uiColor: .secondarySystemBackground)
    static let field = Color(uiColor: .tertiarySystemFill)
    static let selected = Color(uiColor: .systemFill)
    static let keyboard = Color(uiColor: .secondarySystemBackground)
    static let secondary = Color.secondary
    static let waiting = Color.accentColor
    static let working = PaneActivity.working.color
    static let idle = Color(uiColor: .systemGray)
    static let waitingBackground = Color.accentColor.opacity(0.14)
}

extension PaneActivity {
    var sessionColor: Color {
        switch self {
        case .blocked: SessionChrome.waiting
        case .working: SessionChrome.working
        case .idle: SessionChrome.idle
        }
    }

    var sessionTextColor: Color {
        self == .idle ? .secondary : textColor
    }

    var sessionDot: some View {
        agentDot(in: sessionColor)
    }
}

struct AgentRail: View {
    struct Section: Identifiable {
        let kind: HomeAgentSection
        let rows: [GoToPane]
        var id: HomeAgentSection { kind }
    }

    let sections: [Section]
    let focused: String?
    let expand: () -> Void
    let select: (GoToPane) -> Void

    static func groups(in view: MobileView?) -> [Section] {
        guard let view else { return [] }
        return groups(GoToPane.agents(workspaces: view.workspaces, tabs: view.tabs, panes: view.panes))
    }

    nonisolated static func groups(_ agents: [GoToPane]) -> [Section] {
        let grouped = Dictionary(grouping: agents) { row -> HomeAgentSection in
            switch PaneActivity(row.pane.status) {
            case .blocked: .waiting
            case .working: .working
            case .idle: .idle
            case nil: .other
            }
        }
        return [HomeAgentSection.waiting, .working, .idle, .other].compactMap { kind in
            grouped[kind].map { Section(kind: kind, rows: $0) }
        }
    }

    static func summary(_ sections: [Section]) -> String {
        guard !sections.isEmpty else { return "No agents" }
        return sections.map { section in
            let label = switch section.kind {
            case .waiting: "waiting"
            case .other: "other"
            default: section.kind.title.lowercased()
            }
            return "\(section.rows.count.formatted()) \(label)"
        }.joined(separator: " · ")
    }

    var body: some View {
        VStack(spacing: 0) {
            Button(action: expand) {
                Image(systemName: "sidebar.leading").frame(width: 44, height: 44)
            }
            .accessibilityLabel("Expand agents sidebar")
            .accessibilityIdentifier("session.rail.expand")
            Divider().padding(.horizontal, 8)
            rows(content: agentButton)
        }
        .buttonStyle(.plain)
        .background(SessionChrome.sidebar)
        .accessibilityElement(children: .contain)
        .accessibilityIdentifier("session.rail")
    }

    func rows<Row: View>(@ViewBuilder content: @escaping (GoToPane) -> Row) -> some View {
        ScrollView {
            LazyVStack(spacing: 2) {
                ForEach(sections) { section in
                    ForEach(section.rows) { row in
                        content(row)
                            .padding(.top, section.id != sections.first?.id && row.id == section.rows.first?.id ? 8 : 0)
                    }
                }
            }.padding(.vertical, 8)
        }
    }

    private func agentButton(_ row: GoToPane) -> some View {
        let name = row.pane.agentName ?? row.pane.label
        let activity = PaneActivity(row.pane.status)
        let selected = row.id == focused
        let shape = RoundedRectangle(cornerRadius: 12)
        return Button { select(row) } label: {
            AgentBadge(identity: row.pane.agentIdentity, fallbackName: name)
                .frame(width: 21, height: 21)
                .frame(width: 44, height: 44)
                .background(selected ? SessionChrome.selected : .clear, in: shape)
                .overlay {
                    if selected { shape.strokeBorder(activity?.sessionColor ?? SessionChrome.secondary) }
                }
                .overlay(alignment: .topTrailing) {
                    if let activity {
                        activity.sessionDot.padding(2)
                            .background(SessionChrome.sidebar, in: Circle()).padding(3)
                    }
                }
                .contentShape(Rectangle())
        }
        .accessibilityLabel(Text(verbatim: [name, activity?.title].compactMap { $0 }.joined(separator: ", ")))
        .accessibilityValue(Text(verbatim: row.location))
        .accessibilityAddTraits(selected ? .isSelected : [])
        .accessibilityIdentifier("session.rail.agent.\(row.id)")
    }
}
