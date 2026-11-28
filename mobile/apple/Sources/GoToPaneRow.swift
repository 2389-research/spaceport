// ABOUTME: Presents a Go To pane in the expanded sidebar and the navigation sheet.
// ABOUTME: Shares the pane's title, activity and accessible status across both layouts.
import SwiftUI

struct GoToPaneRow: View, Equatable {
    let row: GoToPane
    let address: SSHAddress
    let here: Bool
    let sidebar: Bool
    let searching: Bool
    let age: String?

    private var name: String { row.pane.agentName.flatMap { $0.isEmpty ? nil : $0 } ?? row.pane.label }
    private var activity: PaneActivity? { PaneActivity(row.pane.status) }

    private var label: AgentSelectorLabel {
        AgentSelectorLabel(name: name, terminalTitle: row.pane.terminalTitle,
                           workspace: row.workspace.label, tab: row.tab.label, font: .body.weight(.medium),
                           paneTitle: row.pane.displayPaneTitle, isAgent: row.pane.agentName != nil, address: address)
    }

    var body: some View {
        let role = row.pane.agentName == nil ? "result" : (activity == .blocked ? "needsYou" : "agent")
        let value = [String(label.heading.characters), row.location, activity?.title, here ? "Here" : nil]
            .compactMap { $0 }.joined(separator: ", ")
        Group {
            if sidebar { sidebarLabel } else { sheetLabel }
        }
        .accessibilityValue(Text(verbatim: value))
        .accessibilityIdentifier("goTo.\(role).\(row.id)")
    }

    private var sidebarLabel: some View {
        HStack(spacing: 6) {
            if row.pane.agentName != nil {
                AgentRowBadge(identity: row.pane.agentIdentity, activity: activity,
                              background: here ? SessionChrome.selected : SessionChrome.sidebar,
                              statusColor: activity?.sessionColor ?? .secondary)
            } else if let activity { activity.sessionDot }
            VStack(alignment: .leading, spacing: 3) {
                Text(label.heading).font(.subheadline.weight(.semibold)).lineLimit(1)
                Text(label.subtitle(includingName: row.pane.agentName != nil)).font(.caption).lineLimit(2)
            }
            Spacer(minLength: 2)
            if let age {
                Text(verbatim: age).font(.caption).foregroundStyle(SessionChrome.secondary)
            }
        }
        .padding(.horizontal, 8).padding(.vertical, 6).frame(minHeight: 44)
        .background(here ? SessionChrome.selected : .clear, in: RoundedRectangle(cornerRadius: 10))
        .contentShape(Rectangle())
        .accessibilityLabel(Text(verbatim: name))
        .accessibilityAddTraits(here ? .isSelected : [])
    }

    private var sheetLabel: some View {
        HStack(spacing: 10) {
            if row.pane.agentName != nil {
                AgentRowBadge(identity: row.pane.agentIdentity, activity: activity,
                              background: Color(uiColor: .secondarySystemGroupedBackground),
                              statusColor: activity?.color ?? .secondary)
            }
            label
            Spacer(minLength: 4)
            VStack(alignment: .trailing, spacing: 3) {
                HStack(spacing: 4) {
                    if searching, let activity {
                        Text(activity.title).foregroundStyle(activity.textColor)
                    } else if let age, activity == .idle {
                        Text(verbatim: age).monospacedDigit()
                    }
                }
                if here { Text("Here").accessibilityIdentifier("goTo.here") }
            }
            .font(.footnote).foregroundStyle(.secondary)
        }
        .padding(.vertical, 2).frame(minHeight: 44).contentShape(Rectangle())
        .foregroundStyle(.primary)
        .accessibilityLabel(Text(verbatim: name == row.pane.label ? name : "\(name), \(row.pane.label)"))
    }
}
