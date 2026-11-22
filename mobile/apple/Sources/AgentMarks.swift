// ABOUTME: Keeps Home pins and snoozes on the phone for the lifetime of a Herdr agent.
// ABOUTME: Releases snoozes at their deadline or when new work needs attention.
import Foundation

enum AgentMarkAction {
    case pin(Bool), snooze(Date), wake
}

struct AgentMarks: Hashable, Codable {
    var pinned = false
    var snoozeUntil: Date?
    var heldStatus: String?

    mutating func apply(_ action: AgentMarkAction, status: String) {
        switch action {
        case .pin(let value): pinned = value
        case .snooze(let date):
            snoozeUntil = date
            heldStatus = status
        case .wake:
            snoozeUntil = nil
            heldStatus = nil
        }
    }

    func observed(status: String, at time: Date) -> Self? {
        var marks = self
        if let deadline = snoozeUntil {
            let activity = PaneActivity(status)
            if deadline <= time || (PaneActivity(heldStatus ?? "") != activity && (activity == .blocked || activity == .working)) {
                marks.apply(.wake, status: status)
            }
        }
        return marks.pinned || marks.snoozeUntil != nil ? marks : nil
    }
}

extension HomeAgent {
    func displaySection(at date: Date) -> HomeAgentSection {
        if let deadline = agent.marks?.snoozeUntil, deadline > date { return .snoozed }
        return agent.marks?.pinned == true ? .pinned : section
    }
}
