// ABOUTME: Shows the last session observation in the small and medium Home Screen widgets.
// ABOUTME: WidgetKit refreshes the observation age while the app supplies new counts.
import SwiftUI
import WidgetKit

@main
struct SpaceportWidgets: WidgetBundle {
    var body: some Widget {
        AgentHomeWidget()
        AgentStatusWidget()
    }
}

private struct AgentEntry: TimelineEntry {
    let date: Date
    let snapshot: AgentWidgetSnapshot?

    static func gallerySample() -> Self {
        let date = Date.now
        let sample = AgentWidgetSnapshot(
            destinationID: UUID(), counts: AgentCounts(waiting: 1, working: 3, idle: 2),
            observedAt: date.addingTimeInterval(-120)
        )
        return Self(date: date, snapshot: sample)
    }
}

private struct AgentStatusProvider: TimelineProvider {
    func placeholder(in context: Context) -> AgentEntry {
        .gallerySample()
    }

    func getSnapshot(in context: Context, completion: @escaping (AgentEntry) -> Void) {
        completion(context.isPreview ? .gallerySample() : AgentEntry(date: .now, snapshot: .load()))
    }

    func getTimeline(in context: Context, completion: @escaping (Timeline<AgentEntry>) -> Void) {
        let entry = AgentEntry(date: .now, snapshot: .load())
        completion(Timeline(entries: [entry], policy: .never))
    }
}

private struct AgentHomeWidget: Widget {
    var body: some WidgetConfiguration {
        StaticConfiguration(kind: AgentWidgetSnapshot.widgetKind, provider: AgentStatusProvider()) { entry in
            AgentHomeView(snapshot: entry.snapshot)
                .widgetURL(entry.snapshot?.destinationURL)
                .containerBackground(.background, for: .widget)
        }
        .configurationDisplayName("Agent Status")
        .description("See the last agent counts and open the next waiting agent.")
        .supportedFamilies([.systemSmall, .systemMedium])
    }
}

private struct AgentHomeView: View {
    @Environment(\.widgetFamily) private var family
    let snapshot: AgentWidgetSnapshot?

    var body: some View {
        VStack(alignment: .leading, spacing: 4) {
            Label("Agent Status", systemImage: "terminal")
                .font(.subheadline.weight(.semibold))
                .accessibilityAddTraits(.isHeader)
            Spacer(minLength: 2)
            if let snapshot {
                let compact = family == .systemSmall
                let layout = compact ? AnyLayout(VStackLayout(spacing: 2)) : AnyLayout(HStackLayout(spacing: 8))
                layout {
                    metric(snapshot.counts.waiting, "Waiting", compact: compact)
                    metric(snapshot.counts.working, "Working", compact: compact)
                    metric(snapshot.counts.idle, "Idle", compact: compact)
                }
                Spacer(minLength: 2)
                Text("Checked \(snapshot.observedAt, style: .relative) ago")
                    .font(.caption2)
                    .foregroundStyle(.secondary)
                    .accessibilityLabel(Text("Last checked \(snapshot.observedAt, style: .date) at \(snapshot.observedAt, style: .time)"))
                Label(snapshot.counts.waiting > 0 ? "Next waiting agent" : "Check agents", systemImage: "arrow.right")
                    .font(.caption.weight(.semibold))
                    .lineLimit(1)
                    .minimumScaleFactor(0.8)
            } else {
                Text("Open a session to see agent status.")
                    .font(.subheadline)
                    .foregroundStyle(.secondary)
                Spacer(minLength: 2)
            }
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .topLeading)
    }

    private func metric(_ count: Int, _ label: LocalizedStringKey, compact: Bool) -> some View {
        Group {
            if compact {
                HStack {
                    Text(label)
                    Spacer(minLength: 4)
                    Text(count, format: .number).fontWeight(.semibold)
                }
                .font(.subheadline)
            } else {
                VStack(alignment: .leading, spacing: 0) {
                    Text(count, format: .number).font(.largeTitle.weight(.semibold))
                    Text(label).font(.subheadline).foregroundStyle(.secondary)
                }
                .frame(maxWidth: .infinity, alignment: .leading)
            }
        }
        .lineLimit(1)
        .minimumScaleFactor(0.7)
        .monospacedDigit()
        .accessibilityElement(children: .ignore)
        .accessibilityLabel(Text("\(count) \(Text(label))"))
    }
}
