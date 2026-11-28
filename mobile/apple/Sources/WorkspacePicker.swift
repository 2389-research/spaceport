// ABOUTME: Projects the received Herdr roster into searchable workspaces and pane groups.
// ABOUTME: Keeps browsing independent of the workspace, tab, and pane that own input.
import Foundation

struct WorkspacePicker {
    struct Group: Identifiable {
        let tab: MobileTab
        let panes: [GoToPane]
        var id: String { tab.id }
    }

    private let roster: [MobileWorkspace]
    private let tabsByWorkspace: [String: [MobileTab]]
    private let panesByTab: [String: [MobilePane]]

    init(workspaces: [MobileWorkspace], tabs: [MobileTab], panes: [MobilePane]) {
        roster = workspaces
        tabsByWorkspace = Dictionary(grouping: tabs, by: \.workspaceId)
        panesByTab = Dictionary(grouping: panes, by: \.tabId)
    }

    func workspaces(matching query: String) -> [MobileWorkspace] {
        let words = query.split(whereSeparator: \.isWhitespace)
        return roster.filter { Self.matches(words, in: [$0.label]) }
    }

    func counts(in workspace: MobileWorkspace) -> (tabs: Int, panes: Int) {
        let tabs = tabsByWorkspace[workspace.id] ?? []
        return (tabs.count, tabs.reduce(0) { $0 + (panesByTab[$1.id]?.count ?? 0) })
    }

    func groups(in workspace: MobileWorkspace, matching query: String) -> [Group] {
        let words = query.split(whereSeparator: \.isWhitespace)
        return (tabsByWorkspace[workspace.id] ?? []).compactMap { tab in
            let panes = (panesByTab[tab.id] ?? []).compactMap { pane -> GoToPane? in
                let fields = [tab.label, pane.label, pane.terminalTitle, pane.agentName].compactMap { $0 }
                guard Self.matches(words, in: fields) else { return nil }
                return GoToPane(workspace: workspace, tab: tab, pane: pane)
            }
            guard !panes.isEmpty || Self.matches(words, in: [tab.label]) else { return nil }
            return Group(tab: tab, panes: panes)
        }
    }

    func current(workspaceID: String?, tabID: String?, paneID: String?) -> GoToPane? {
        guard let workspace = roster.first(where: { $0.id == workspaceID }),
              let tab = tabsByWorkspace[workspace.id]?.first(where: { $0.id == tabID }),
              let pane = panesByTab[tab.id]?.first(where: { $0.id == paneID }) else { return nil }
        return GoToPane(workspace: workspace, tab: tab, pane: pane)
    }

    private static func matches(_ words: [Substring], in fields: [String]) -> Bool {
        words.allSatisfy { word in fields.contains { $0.localizedStandardContains(String(word)) } }
    }
}
