// ABOUTME: Captures last-seen pane activity and focus labels across the active Herdr session.
// ABOUTME: Keeps terminal contents and SSH credentials out of local preferences.
import Foundation

extension SessionStatusSnapshot {
    init(view: MobileView, seenAt: Date = Date()) {
        let workspace = view.workspaces.first { $0.id == view.focusedWorkspace }
        let tab = view.tabs.first { $0.id == view.focusedTab && $0.workspaceId == workspace?.id }
        let pane = view.panes.first { $0.id == view.focusedPane && $0.tabId == tab?.id }
        let agents = GoToPane.agents(workspaces: view.workspaces, tabs: view.tabs, panes: view.panes)
        let rows = agents.compactMap { row -> SeenAgent? in
            guard let name = row.pane.agentName else { return nil }
            let firstSeen = row.pane.idleFirstObservedMs.map { Date(timeIntervalSince1970: Double($0) / 1_000) }
            return SeenAgent(bootId: view.bootId, workspaceId: row.workspace.id, tabId: row.tab.id,
                             paneId: row.id, name: name, status: row.pane.status, location: row.location,
                             stateChangeSeq: row.pane.stateChangeSeq, changeObservedAt: firstSeen ?? seenAt,
                             idleOnsetObserved: PaneActivity(row.pane.status) == .idle ? row.pane.idleOnsetObserved : nil,
                             terminalTitle: row.pane.terminalTitle)
        }
        // Save attention requests before applying the per-session cache limit.
        let ordered = rows.filter { $0.status == "blocked" } + rows.filter { $0.status != "blocked" }
        self.init(seenAt: seenAt,
                  blocked: view.panes.filter { $0.status == "blocked" }.count,
                  working: view.panes.filter { $0.status == "working" }.count,
                  hasIdle: view.panes.contains { $0.status == "idle" || $0.status == "done" },
                  location: [workspace?.label, tab?.label, pane?.label].compactMap { $0 }, agents: ordered)
    }
}
