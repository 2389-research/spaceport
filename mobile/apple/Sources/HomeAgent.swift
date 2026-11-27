// ABOUTME: Groups saved agents using Spaceport’s shared idle age bands.
// ABOUTME: Orders undated agents by Herdr sequence within each saved session.
import Foundation

enum HomeAgentSection: Int, CaseIterable {
    case pinned, waiting, working, finished, recentIdle, idleHours, oldIdle, idle, quiet, other, snoozed

    var title: String {
        switch self {
        case .pinned: "Pinned"
        case .snoozed: "Snoozed"
        case .waiting: "Waiting for you"
        case .finished: "Finished for review"
        case .quiet: "Quiet"
        case .working: "Working"
        case .recentIdle: "Idle · under 1 hour"
        case .idleHours: "Idle · 1–5 hours"
        case .oldIdle: "Idle · 5+ hours"
        case .idle: "Idle"
        case .other: "Other agents"
        }
    }
}

struct HomeAgent: Identifiable {
    struct ID: Hashable {
        let sessionID: UUID
        let bootId: String
        let paneId: String
    }
    let host: SavedHost
    let profile: HostProfile
    let agent: SeenAgent
    let seenAt: Date
    let section: HomeAgentSection
    let ageLabel: String?
    fileprivate(set) var sortValue: Double
    var id: ID { ID(sessionID: profile.id, bootId: agent.bootId, paneId: agent.paneId) }

    private init(agent: SeenAgent, seenAt: Date, host: SavedHost, profile: HostProfile) {
        self.host = host
        self.profile = profile
        self.agent = agent
        self.seenAt = seenAt
        sortValue = seenAt.timeIntervalSince1970
        switch PaneActivity(agent.status) {
        case .blocked: section = .waiting
        case .working: section = .working
        case nil: section = .other
        case .idle:
            if agent.status == "done" {
                section = .finished
                ageLabel = nil
                return
            }
            let confirmed = Self.milliseconds(seenAt)
            let first = agent.changeObservedAt.map(Self.milliseconds)
            // Old catalogs have a local sort date but no reliable onset provenance.
            let knownFirst = agent.idleOnsetObserved == nil ? nil : first
            if let knownFirst, confirmed >= knownFirst, confirmed - knownFirst >= 86_400_000 {
                section = .quiet
            } else {
                switch idleAgeGroup(firstObservedMs: knownFirst, onsetObserved: agent.idleOnsetObserved == true,
                                   confirmedAtMs: confirmed) {
                case .recent: section = .recentIdle
                case .oneToFiveHours: section = .idleHours
                case .fiveHoursOrMore: section = .oldIdle
                case .unplaced: section = .idle
                }
            }
            if section != .idle, let first {
                let elapsed = confirmed >= first ? confirmed - first : 0
                ageLabel = Self.idleLabel(elapsedMs: elapsed, observedOnset: agent.idleOnsetObserved == true)
                sortValue = -Double(elapsed)
                return
            }
            sortValue = agent.changeObservedAt?.timeIntervalSince1970 ?? sortValue
        }
        ageLabel = nil
    }

    static func idleLabel(elapsedMs: UInt64, observedOnset: Bool) -> String {
        let seconds = elapsedMs / 1_000
        let value: String
        switch seconds {
        case ..<60: value = "<1m"
        case ..<3_600: value = "\(seconds / 60)m"
        default: value = "\(seconds / 3_600)h"
        }
        return observedOnset ? value : value + "+"
    }

    static func milliseconds(_ date: Date) -> UInt64 {
        let ms = (date.timeIntervalSince1970 * 1_000).rounded()
        guard ms > 0, ms < Double(UInt64.max) else { return 0 }
        return UInt64(ms)
    }

    static func rows(from snapshot: SessionStatusSnapshot, host: SavedHost, profile: HostProfile) -> [Self] {
        let rows = (snapshot.agents ?? []).map { Self(agent: $0, seenAt: snapshot.seenAt, host: host, profile: profile) }
        let undated = rows.filter { $0.section == .idle }.sorted {
            ($0.agent.stateChangeSeq ?? 0) > ($1.agent.stateChangeSeq ?? 0)
        }
        // Sequence numbers belong to one daemon. Keep its order while interleaving
        // sessions on their own observation dates.
        var newestPossible = Double.infinity
        return rows.filter { $0.section != .idle } + undated.map { row in
            var row = row
            newestPossible = min(newestPossible, row.sortValue)
            row.sortValue = newestPossible
            return row
        }
    }
}
