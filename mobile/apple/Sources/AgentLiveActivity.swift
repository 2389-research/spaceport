// ABOUTME: Publishes the selected SSH presentation’s observed agent counts to ActivityKit.
// ABOUTME: Coalesces terminal updates and marks the last observation stale when its feed pauses.
import ActivityKit
import OSLog
import UIKit

struct AgentActivityFeed {
    typealias Content = AgentStatusAttributes.ContentState
    private(set) var selected: UUID?
    private(set) var content: Content?
    private var lastObservation: Date?

    mutating func select(_ id: UUID) {
        guard selected != id else { return }
        self = Self()
        selected = id
    }

    mutating func end(_ id: UUID) {
        if selected == id { self = Self() }
    }

    mutating func observe(_ counts: AgentCounts, at date: Date, from id: UUID) {
        guard selected == id else { return }
        lastObservation = date
        if let content, !content.paused, content.counts == counts,
           date.timeIntervalSince(content.observedAt) < 60 { return }
        content = Content(counts: counts, observedAt: date, paused: false)
    }

    mutating func pause(_ id: UUID) {
        guard selected == id, let counts = content?.counts, let lastObservation else { return }
        content = Content(counts: counts, observedAt: lastObservation, paused: true)
    }
}

@MainActor
final class AgentLiveActivity: ObservableObject {
    private typealias StatusActivity = Activity<AgentStatusAttributes>
    static let shared = AgentLiveActivity()

    private static let preference = "agentLiveActivityEnabled"
    private let defaults: UserDefaults
    @Published private(set) var isEnabled: Bool

    private var feed = AgentActivityFeed()
    private var owner: UUID?
    private var activity: StatusActivity?
    private var worker: Task<Void, Never>?
    private var pending = false

    init(defaults: UserDefaults = .standard) {
        self.defaults = defaults
        isEnabled = defaults.object(forKey: Self.preference) as? Bool ?? true
        // A new process has no SSH presentation to update activities left by its predecessor.
        let orphaned = StatusActivity.activities
        worker = Task {
            for orphan in orphaned { await orphan.end(nil, dismissalPolicy: .immediate) }
            await drain()
        }
    }

    @discardableResult
    func setEnabled(_ enabled: Bool) -> Task<Void, Never>? {
        if enabled != isEnabled {
            defaults.set(enabled, forKey: Self.preference)
            isEnabled = enabled
            schedule()
        }
        return worker
    }

    func select(_ id: UUID, view: MobileView?, observedAt: Date?) {
        guard feed.selected != id else { return }
        feed.select(id)
        // A focused iPad scene can already have a quiet, verified foreground connection.
        if let view, let observedAt {
            feed.observe(AgentCounts(view: view), at: observedAt, from: id)
        }
        schedule()
    }

    func end(_ id: UUID?) {
        guard let id, feed.selected == id else { return }
        feed.end(id)
        schedule()
    }

    func pause(_ id: UUID?) {
        guard let id else { return }
        let previous = feed.content
        feed.pause(id)
        if feed.content != previous { schedule() }
    }

    func observe(_ view: MobileView, previous: MobileView?, at date: Date, from id: UUID?) {
        guard let id, feed.selected == id else { return }
        let prior = feed.content
        let counts: AgentCounts
        if let prior, previous?.panes == view.panes, previous?.tabs == view.tabs,
           previous?.workspaces == view.workspaces {
            counts = prior.counts
        } else {
            counts = AgentCounts(view: view)
        }
        feed.observe(counts, at: date, from: id)
        if feed.content != prior { schedule() }
    }

    private func schedule() {
        guard isEnabled || activity != nil else { return }
        pending = true
        if worker == nil { worker = Task { await drain() } }
    }

    private func drain() async {
        while pending {
            pending = false
            if !isEnabled || owner != feed.selected {
                if let activity { await activity.end(nil, dismissalPolicy: .immediate) }
                activity = nil
                owner = feed.selected
            }
            guard isEnabled, let content = feed.content else { continue }
            let update = ActivityContent(state: content, staleDate: content.staleDate)
            if let activity {
                // Keep the dismissed handle until this presentation ends, so a swipe stays dismissed.
                if activity.activityState == .active || activity.activityState == .stale {
                    await activity.update(update)
                }
            } else if !content.paused, content.counts != AgentCounts(),
                      ActivityAuthorizationInfo().areActivitiesEnabled,
                      UIApplication.shared.applicationState == .active {
                do {
                    activity = try StatusActivity.request(attributes: AgentStatusAttributes(), content: update)
                } catch {
                    Logger(subsystem: "ai.2389.spaceport.mobile", category: "LiveActivity")
                        .notice("Could not start agent status: \(error.localizedDescription, privacy: .private)")
                }
            }
        }
        worker = nil
    }
}
