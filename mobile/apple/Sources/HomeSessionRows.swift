// ABOUTME: Presents observed agents and saved sessions on Home.
// ABOUTME: Labels cached activity with the relative time of the observation.
import SwiftUI

private extension SessionStatusSnapshot {
    var activity: PaneActivity? {
        if blocked > 0 { return .blocked }
        if working > 0 { return .working }
        return hasIdle ? .idle : nil
    }

    var blockedLabel: String { blocked == 1 ? "1 needs you" : "\(blocked) need you" }
    var labels: [String] {
        var labels: [String] = []
        if blocked > 0 { labels.append(blockedLabel) }
        if working > 0 { labels.append("\(working) working") }
        if labels.isEmpty && hasIdle { labels.append("Idle") }
        return labels
    }
}

struct HomeAgentsSection: View {
    let agents: [HomeAgent]
    let reviewOnly: Bool
    let now: Date
    let mark: (HomeAgent, AgentMarkAction) -> Void
    let open: (HomeAgent) -> Void

    @State private var showQuiet = false

    private struct LabelPath: Hashable {
        let hostname: String
        let session: String
        let name: String
        let location: String

        init(_ row: HomeAgent) {
            hostname = row.profile.hostname
            session = row.profile.session
            name = row.agent.name
            location = row.agent.location
        }
    }

    var body: some View {
        let groups = Dictionary(grouping: agents, by: LabelPath.init)
        let multipleSessions = Set(agents.map { $0.profile.id }).count > 1
        let accountCount = Set(agents.map { $0.profile.address }).count
        let bySection = Dictionary(grouping: agents) { $0.displaySection(at: now) }
        let sections = HomeAgentSection.allCases.filter { bySection[$0] != nil }
        if agents.isEmpty {
            Section {
                Text(reviewOnly ? "No agents have finished work to review." : "Open a session to see its agents here.")
                    .font(.subheadline).foregroundStyle(.secondary)
            } header: { Text("Across sessions") }
        }
        ForEach(sections, id: \.self) { section in
            Section {
                ForEach(section == .quiet && !showQuiet ? [] : bySection[section] ?? []) { row in
                    let duplicates = groups[LabelPath(row)] ?? []
                    let hasSamePane = duplicates.contains { $0.agent.paneId == row.agent.paneId && $0.id != row.id }
                    let detail = [
                        duplicates.contains { $0.profile.port != row.profile.port } ? "Port \(row.profile.port)" : "",
                        duplicates.contains { $0.profile.username != row.profile.username } ? "Account " + row.profile.username : "",
                        duplicates.count > 1 ? "Pane \(row.agent.paneId)" : "",
                        hasSamePane ? row.profile.login.title : ""
                    ].filter { !$0.isEmpty }.joined(separator: " · ")
                    let activity = PaneActivity(row.agent.status)
                    let markLabel: String? = switch section {
                    case .snoozed: row.agent.marks?.snoozeUntil.map { "Snoozed until " + $0.formatted(date: .abbreviated, time: .shortened) }
                    case .pinned: "Pinned"
                    default: nil
                    }
                    let spokenStatus = [markLabel, activity?.title, row.ageLabel].compactMap { $0 }.joined(separator: " ")
                    let path = row.agent.locationLabels
                    let label = AgentSelectorLabel(name: row.agent.name, terminalTitle: row.agent.terminalTitle,
                                                   workspace: path.workspace, tab: path.tab,
                                                   host: accountCount > 1 ? row.profile.hostname : "",
                                                   session: multipleSessions ? row.profile.session : "",
                                                   detail: detail, font: .headline, paneTitle: row.agent.paneTitle, address: row.profile.address)
                    let spokenLabel = [row.agent.name, row.profile.hostname, row.profile.session, row.agent.location, detail]
                        .filter { !$0.isEmpty }.joined(separator: ", ")
                    Button { open(row) } label: {
                        HStack(spacing: 10) {
                            AgentRowBadge(identity: row.agent.agentIdentity, activity: activity,
                                          background: Color(uiColor: .secondarySystemGroupedBackground),
                                          statusColor: activity?.color ?? .secondary)
                            label
                            Spacer(minLength: 4)
                            if section == .snoozed, let wake = row.agent.marks?.snoozeUntil {
                                Text(wake, style: .relative).font(.footnote).foregroundStyle(.secondary)
                            } else if let age = row.ageLabel {
                                Text(age).font(.subheadline.monospacedDigit()).foregroundStyle(.secondary)
                            }
                            Image(systemName: "chevron.right").font(.caption.weight(.semibold)).foregroundStyle(.tertiary)
                        }
                        .padding(.vertical, 3).contentShape(Rectangle())
                    }
                    .buttonStyle(.plain)
                    .accessibilityLabel(Text(verbatim: spokenLabel))
                    .accessibilityValue(Text("\(String(label.heading.characters)), \(spokenStatus), last seen \(row.seenAt, format: .relative(presentation: .named))"))
                    .accessibilityIdentifier("home.agent.\(row.profile.id).\(row.agent.bootId).\(row.agent.paneId)")
                    .modifier(AgentMarkActions(row: row, snoozed: section == .snoozed) { mark(row, $0) })
                }
            } header: {
                if section == .quiet {
                    Button { showQuiet.toggle() } label: {
                        Label("Quiet (\(bySection[section]?.count ?? 0))", systemImage: showQuiet ? "chevron.down" : "chevron.right")
                    }.accessibilityValue(showQuiet ? "Expanded" : "Collapsed")
                } else { Text(section.title) }
            } footer: {
                if section == sections.last, let first = agents.first, let latest = agents.map(\.seenAt).max() {
                    if accountCount > 1 {
                        Text("From \(accountCount) SSH accounts · Latest observation \(latest, format: .relative(presentation: .named))")
                    } else if multipleSessions {
                        Text("From agents on \(first.host.title) · Latest observation \(latest, format: .relative(presentation: .named))")
                    } else {
                        Text("From agents on \(first.host.title) · Last seen \(latest, format: .relative(presentation: .named))")
                    }
                }
            }
        }
    }
}

struct HomeSessionRow: View {
    let session: SavedSession
    var live = false
    var refreshFailed = false

    var body: some View {
        HStack(spacing: 12) {
            if let status = session.lastStatus {
                status.activity?.dot
            } else {
                Circle().strokeBorder(style: StrokeStyle(lineWidth: 1.5, dash: [2, 2]))
                    .foregroundStyle(.secondary).frame(width: 9, height: 9).accessibilityHidden(true)
            }
            VStack(alignment: .leading, spacing: 3) {
                Text(session.name).foregroundStyle(.primary)
                if let status = session.lastStatus {
                    let labels = status.labels.joined(separator: " · ")
                    Group {
                        if live {
                            Text(labels.isEmpty ? "Live" : "Live · " + labels)
                        } else {
                            Text("\(labels.isEmpty ? "" : labels + " · ")Last seen \(status.seenAt, format: .relative(presentation: .named))")
                        }
                    }
                    .font(.footnote).foregroundStyle(.secondary)
                    if refreshFailed && !live { Text("Couldn’t refresh").font(.footnote).foregroundStyle(.secondary) }
                } else {
                    Text("Not opened yet").font(.footnote).foregroundStyle(.secondary)
                }
            }
            Spacer()
            Image(systemName: "chevron.right").font(.caption.weight(.semibold)).foregroundStyle(.tertiary)
        }
        .padding(.vertical, 3).contentShape(Rectangle())
    }
}
