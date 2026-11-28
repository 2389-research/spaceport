// ABOUTME: Draws a small terminal rocket while the native client waits for a connection.
// ABOUTME: Keeps the status readable when space is tight or Reduce Motion is enabled.
import SwiftUI

struct ConnectionWait<Label: View>: View {
    @ViewBuilder let label: () -> Label

    var body: some View {
        ViewThatFits(in: .vertical) {
            VStack(spacing: 12) { ConnectionFlight(); label() }
            label()
        }
    }
}

struct ConnectionFlight: View {
    @Environment(\.accessibilityReduceMotion) private var reduceMotion
    @Environment(\.scenePhase) private var scenePhase

    var body: some View {
        Group {
            if reduceMotion || scenePhase != .active {
                Frame(step: 0)
            } else {
                TimelineView(.periodic(from: .now, by: 0.4)) { moment in
                    Frame(step: Int(moment.date.timeIntervalSinceReferenceDate / 0.4))
                }
            }
        }
        .fixedSize()
        .environment(\.layoutDirection, .leftToRight)
        .accessibilityHidden(true)
        .allowsHitTesting(false)
    }

    struct Frame: View {
        let step: Int
        var fontSize: CGFloat = 12
        private static let stars = ["   +  ", "      ", " ·    ", "      ", "     .", "      ", "      "]
        private static let hull = [#"   /\   "#, #"  /  \  "#, "  |()|  ", #" /|  |\ "#, #"/_|__|_\"#]

        var body: some View {
            let plume = step.isMultiple(of: 2) ? ["  '::'  ", "   ::   "] : ["   ::   ", "   ''   "]
            let rocket = Self.hull + plume
            VStack(spacing: -1) {
                ForEach(rocket.indices, id: \.self) { row in
                    let stars = Self.stars
                    let sky = (row + stars.count - step % stars.count) % stars.count
                    HStack(spacing: 0) {
                        Text(verbatim: stars[sky]).foregroundStyle(.secondary)
                        let hullInk: Color = row == 2 ? .cyan : .primary
                        Text(verbatim: rocket[row]).foregroundStyle(row < Self.hull.count ? hullInk : .orange)
                        Text(verbatim: stars[(sky + 3) % stars.count]).foregroundStyle(.secondary)
                    }
                }
            }
            .font(.system(size: fontSize, weight: .bold, design: .monospaced))
            .fixedSize()
        }
    }
}
