// ABOUTME: Presents a workspace directory and its pane destinations on iPhone and Duo.
// ABOUTME: Marks the live terminal separately from the workspace being inspected.
import SwiftUI

struct BrowsedWorkspace: Equatable {
    let id: String
    let boot: String
    let label: String
}

struct WorkspacePickerView: View {
    let view: MobileView
    let picker: WorkspacePicker
    let location: BrowsedWorkspace?
    let query: String
    let address: SSHAddress
    let inspect: (MobileWorkspace) -> Void
    let choose: (GoToPane) -> Void
    let menu: (GoToPane) -> PaneContextMenu
    @State private var removedPane: String?

    private struct ListedPane: Equatable {
        let id: String
        let title: String
    }

    var body: some View {
        if let location {
            if location.boot == view.bootId, let workspace = view.workspaces.first(where: { $0.id == location.id }) {
                let groups = picker.groups(in: workspace, matching: query)
                detail(workspace, groups: groups)
            } else {
                ContentUnavailableView("Workspace unavailable", systemImage: "folder.badge.questionmark",
                                       description: Text(verbatim: "“\(location.label)” is no longer in this session."))
            }
        } else {
            directory(picker)
        }
    }

    private func directory(_ picker: WorkspacePicker) -> some View {
        let matches = picker.workspaces(matching: query)
        return Section {
            if matches.isEmpty {
                if view.workspaces.isEmpty {
                    ContentUnavailableView("No workspaces", systemImage: "folder")
                } else {
                    ContentUnavailableView.search(text: query)
                }
            }
            ForEach(matches, id: \.id) { item in
                let summary = Self.summary(picker.counts(in: item))
                let here = item.id == view.focusedWorkspace
                Button { inspect(item) } label: {
                    HStack(spacing: 10) {
                        Image(systemName: "folder").font(.subheadline)
                            .foregroundStyle(here ? Color.accentColor : .secondary)
                            .frame(width: 32, height: 32)
                            .background(SessionChrome.field, in: .rect(cornerRadius: 8))
                        VStack(alignment: .leading, spacing: 3) {
                            Text(verbatim: item.label).font(.body.weight(.medium)).lineLimit(2)
                            Text(verbatim: summary).font(.caption).foregroundStyle(.secondary)
                        }
                        Spacer(minLength: 3)
                        if here { Image(systemName: "checkmark.circle.fill").foregroundStyle(Color.accentColor) }
                        Image(systemName: "chevron.right").font(.caption.weight(.semibold)).foregroundStyle(.secondary)
                    }
                    .frame(minHeight: 44).contentShape(Rectangle())
                }
                .buttonStyle(.plain)
                .accessibilityLabel(Text(verbatim: item.label))
                .accessibilityValue(Text(verbatim: here ? "Current workspace, " + summary : summary))
                .accessibilityAddTraits(here ? .isSelected : [])
                .accessibilityIdentifier("goTo.workspace.\(item.id)")
            }
        } header: { Text("Workspaces") }
    }

    static func summary(_ counts: (tabs: Int, panes: Int)) -> String {
        "\(counts.tabs) \(counts.tabs == 1 ? "tab" : "tabs") · \(counts.panes) \(counts.panes == 1 ? "pane" : "panes")"
    }

    @ViewBuilder private func detail(_ workspace: MobileWorkspace, groups: [WorkspacePicker.Group]) -> some View {
        let listed = groups.flatMap { $0.panes.map { ListedPane(id: $0.id, title: title($0)) } }
        if let removedPane {
            Label { Text(verbatim: "“\(removedPane)” is no longer in this workspace.") } icon: {
                Image(systemName: "info.circle")
            }
            .font(.footnote).foregroundStyle(.secondary)
        }
        if groups.isEmpty {
            if query.contains(where: { !$0.isWhitespace }) {
                ContentUnavailableView.search(text: query)
            } else {
                ContentUnavailableView("No panes in this workspace", systemImage: "rectangle.split.2x1")
            }
        }
        ForEach(groups) { group in
            Section {
                if group.panes.isEmpty { Text("No panes in this tab").foregroundStyle(.secondary) }
                ForEach(group.panes) { row in
                    paneRow(row)
                }
                if group.tab.id == view.focusedTab, workspace.id == view.focusedWorkspace,
                   query.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty,
                   let map = view.paneMap, map.panes.count > 1 {
                    PaneMapView(view: view) { id in
                        if let pane = group.panes.first(where: { $0.id == id }) { choose(pane) }
                    }
                }
            } header: { Text(verbatim: group.tab.label).textCase(nil) }
        }
        .onChange(of: listed) { previous, now in
            let currentIDs = Set(now.map(\.id))
            let visibleLoss = previous.filter { !currentIDs.contains($0.id) }
            guard !visibleLoss.isEmpty else { return }
            // A pane hidden by the search query still exists in the roster.
            let liveIDs = Set(view.panes.map(\.id))
            if let lost = visibleLoss.first(where: { !liveIDs.contains($0.id) }) { removedPane = lost.title }
        }
    }

    private func title(_ row: GoToPane) -> String {
        AgentSelectorLabel.displayTitle(row.pane.terminalTitle, address: address) ?? row.pane.label
    }

    private func paneRow(_ row: GoToPane) -> some View {
        let name = title(row)
        let detail = [row.pane.label == name ? nil : row.pane.label, row.pane.agentName]
            .compactMap { $0 }.joined(separator: " · ")
        let here = row.isFocused(in: view)
        return Button { choose(row) } label: {
            HStack(spacing: 6) {
                VStack(alignment: .leading, spacing: 3) {
                    Text(verbatim: name).font(.body.weight(.medium)).lineLimit(2)
                    if !detail.isEmpty { Text(verbatim: detail).font(.caption).foregroundStyle(.secondary).lineLimit(2) }
                }
                Spacer(minLength: 2)
                if let activity = PaneActivity(row.pane.status) {
                    activity.sessionDot
                    Text(activity.title).font(.caption).foregroundStyle(activity.sessionTextColor)
                }
                if here { Image(systemName: "checkmark").font(.caption.weight(.bold)) }
            }
            .frame(minHeight: 44).contentShape(Rectangle())
        }
        .buttonStyle(.plain)
        .accessibilityAddTraits(here ? .isSelected : [])
        .accessibilityIdentifier("goTo.pane.\(row.id)")
        .modifier(menu(row))
    }
}
