// ABOUTME: Captures last-seen pane activity and focus labels across the active Herdr session.
// ABOUTME: Keeps terminal contents and SSH credentials out of local preferences.
import Foundation

struct RosterSaveAttempt {
    private let boot: String
    private let accepted: Bool
    private let observedAt: UInt64?
    private let panes: [MobilePane]
    private let tabs: [MobileTab]
    private let workspaces: [MobileWorkspace]

    init(_ view: MobileView, accepted: Bool) {
        self.accepted = accepted
        observedAt = view.rosterObservedAtMs
        boot = view.bootId
        panes = view.panes
        tabs = view.tabs
        workspaces = view.workspaces
    }

    func matches(_ view: MobileView) -> Bool {
        if let current = view.rosterObservedAtMs, let observedAt {
            if accepted && current >= observedAt && current - observedAt >= 60_000 { return false }
            if !accepted && current != observedAt { return false }
        }
        return boot == view.bootId && panes == view.panes &&
            tabs.elementsEqual(view.tabs) {
                $0.id == $1.id && $0.workspaceId == $1.workspaceId && $0.label == $1.label
            } && workspaces.elementsEqual(view.workspaces) {
                $0.id == $1.id && $0.label == $1.label
            }
    }
}

extension MobileView {
    /// An unchanged roster occasionally advances Last seen when the daemon confirms it.
    func observedRoster(after saved: MobileView?) -> SessionStatusSnapshot? {
        guard !bootId.isEmpty, let date = rosterObservationDate else { return nil }
        if let saved, RosterSaveAttempt(saved, accepted: true).matches(self),
           saved.focusedWorkspace == focusedWorkspace, saved.focusedTab == focusedTab, saved.focusedPane == focusedPane {
            return nil
        }
        return SessionStatusSnapshot(view: self, seenAt: date)
    }

    func changedAgentStatus(since attempt: RosterSaveAttempt?) -> SessionStatusSnapshot? {
        guard !bootId.isEmpty, let date = rosterObservationDate, attempt?.matches(self) != true else { return nil }
        return SessionStatusSnapshot(view: self, seenAt: date)
    }

    var rosterObservationDate: Date? {
        rosterObservedAtMs.map { Date(timeIntervalSince1970: Double($0) / 1_000) }
    }
}

extension AgentCounts {
    init(view: MobileView) {
        self.init(agents: GoToPane.agents(workspaces: view.workspaces, tabs: view.tabs, panes: view.panes))
    }

    fileprivate init(agents: [GoToPane]) {
        self.init()
        for agent in agents {
            switch PaneActivity(agent.pane.status) {
            case .blocked: waiting += 1
            case .working: working += 1
            case .idle: idle += 1
            case nil: break
            }
        }
    }
}

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
                             terminalTitle: row.pane.terminalTitle, agentIdentity: row.pane.agentIdentity,
                             workspaceLabelBytes: row.workspace.label.utf8.count, paneTitle: row.pane.displayPaneTitle)
        }
        // Save attention requests before applying the per-session cache limit.
        let ordered = rows.filter { $0.status == "blocked" } + rows.filter { $0.status != "blocked" }
        self.init(seenAt: seenAt,
                  blocked: view.panes.filter { $0.status == "blocked" }.count,
                  working: view.panes.filter { $0.status == "working" }.count,
                  hasIdle: view.panes.contains { $0.status == "idle" || $0.status == "done" },
                  location: [workspace?.label, tab?.label, pane?.label].compactMap { $0 }, agents: ordered,
                  agentCounts: AgentCounts(agents: agents))
    }
}
