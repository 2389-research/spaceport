// ABOUTME: Defines the aggregate Herdr status that the app sends to ActivityKit.
// ABOUTME: The widget receives only counts, observation time and whether updates paused.
import ActivityKit
import Foundation

struct AgentCounts: Codable, Hashable {
    var waiting = 0
    var working = 0
    var idle = 0
}

struct AgentStatusAttributes: ActivityAttributes {
    struct ContentState: Codable, Hashable {
        let counts: AgentCounts
        let observedAt: Date
        let paused: Bool

        var staleDate: Date { paused ? observedAt : observedAt.addingTimeInterval(120) }
    }
}
