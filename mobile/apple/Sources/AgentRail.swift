// ABOUTME: Keeps live agent status and confirmed pane navigation within thumb reach while typing.
// ABOUTME: Shares the attention-first roster and colors with the expanded session sidebar.
import SwiftUI

enum SessionChrome {
    static let terminal = Color(red: 14/255, green: 14/255, blue: 16/255)
    static let sidebar = Color(red: 28/255, green: 28/255, blue: 30/255)
    static let field = Color(red: 44/255, green: 44/255, blue: 46/255)
    static let selected = Color(red: 58/255, green: 58/255, blue: 60/255)
    static let keyboard = Color(red: 42/255, green: 42/255, blue: 45/255)
    static let secondary = Color(red: 161/255, green: 161/255, blue: 166/255)
    static let waiting = Color(red: 1, green: 92/255, blue: 147/255)
    static let working = Color(red: 91/255, green: 155/255, blue: 240/255)
    static let idle = Color(red: 142/255, green: 142/255, blue: 147/255)
    static let connected = Color(red: 48/255, green: 209/255, blue: 88/255)
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
        switch self {
        case .blocked: Color(red: 1, green: 122/255, blue: 168/255)
        case .working: Color(red: 127/255, green: 176/255, blue: 1)
        case .idle: SessionChrome.secondary
        }
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

    var body: some View {
        VStack(spacing: 0) {
            Button(action: expand) {
                Image(systemName: "sidebar.leading").frame(width: 44, height: 44)
            }
            .accessibilityLabel("Expand agents sidebar")
            .accessibilityIdentifier("session.rail.expand")
            Divider().padding(.horizontal, 8)
            ScrollView {
                LazyVStack(spacing: 10) {
                    ForEach(sections) { section in
                        VStack(spacing: 2) {
                            ForEach(section.rows) { row in agentButton(row) }
                        }
                    }
                }.padding(.vertical, 8)
            }
        }
        .buttonStyle(.plain)
        .background(SessionChrome.sidebar)
        .accessibilityElement(children: .contain)
        .accessibilityIdentifier("session.rail")
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
