// ABOUTME: Shows the selected Herdr session’s agent counts on the Lock Screen and Dynamic Island.
// ABOUTME: Every presentation signals when a displayed observation is out of date.
import ActivityKit
import SwiftUI
import WidgetKit

struct AgentStatusWidget: Widget {
    var body: some WidgetConfiguration {
        ActivityConfiguration(for: AgentStatusAttributes.self) { context in
            let stale = context.isStale || context.state.paused
            VStack(alignment: .leading, spacing: 8) {
                Label("Spaceport agents", systemImage: "terminal").font(.headline)
                AgentTotals(counts: context.state.counts)
                ObservationTime(date: context.state.observedAt, stale: stale)
            }
            .frame(maxWidth: .infinity, alignment: .leading)
            .padding()
        } dynamicIsland: { context in
            let stale = context.isStale || context.state.paused
            let counts = context.state.counts
            return DynamicIsland {
                DynamicIslandExpandedRegion(.leading) {
                    Label("Agents", systemImage: "terminal").font(.headline)
                }
                DynamicIslandExpandedRegion(.bottom) {
                    VStack(alignment: .leading, spacing: 6) {
                        AgentTotals(counts: counts)
                        ObservationTime(date: context.state.observedAt, stale: stale)
                    }
                    .frame(maxWidth: .infinity, alignment: .leading)
                }
            } compactLeading: {
                Image(systemName: stale ? "clock" : "terminal").accessibilityLabel(stale ? "Last seen" : "Spaceport agents")
            } compactTrailing: {
                IslandCount(counts: counts, stale: stale)
            } minimal: {
                if stale {
                    Image(systemName: "clock").accessibilityLabel("Last seen agent status")
                } else {
                    IslandCount(counts: counts, stale: false)
                }
            }
        }
    }
}

private struct AgentTotals: View {
    let counts: AgentCounts

    var body: some View {
        ViewThatFits(in: .horizontal) {
            HStack(spacing: 16) { labels }
            VStack(alignment: .leading) { labels }
        }
        .font(.subheadline.weight(.semibold))
        .monospacedDigit()
    }

    @ViewBuilder private var labels: some View {
        Text("\(counts.waiting) waiting")
        Text("\(counts.working) working")
        Text("\(counts.idle) idle")
    }
}

private struct ObservationTime: View {
    let date: Date
    let stale: Bool

    var body: some View {
        Label {
            if stale { Text("Last seen \(date, style: .time)") }
            else { Text("Seen \(date, style: .time)") }
        } icon: {
            Image(systemName: stale ? "clock" : "checkmark.circle")
        }
        .font(.caption)
        .foregroundStyle(.secondary)
        .accessibilityLabel(stale ? Text("Last seen \(date, style: .time), updates paused") : Text("Seen \(date, style: .time)"))
    }
}

private struct IslandCount: View {
    let counts: AgentCounts
    let stale: Bool

    private var status: (count: Int, name: String, symbol: String) {
        if counts.waiting > 0 { return (counts.waiting, "waiting", "exclamationmark") }
        if counts.working > 0 { return (counts.working, "working", "bolt.fill") }
        return (counts.idle, "idle", "moon.fill")
    }

    var body: some View {
        HStack(spacing: 1) {
            Image(systemName: status.symbol).imageScale(.small)
            Text(status.count.formatted()).monospacedDigit()
        }
        .lineLimit(1)
        .minimumScaleFactor(0.7)
        .accessibilityElement(children: .ignore)
        .accessibilityLabel("\(stale ? "Last seen, " : "")\(status.count) \(status.name)")
    }
}
