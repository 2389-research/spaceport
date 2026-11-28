// ABOUTME: Covers the terminal while its scene is inactive.
// ABOUTME: Draws Spaceport’s ASCII rocket on an opaque, static orbital chart.
import SwiftUI

struct SessionPrivacyCover: View {
    @Environment(\.colorScheme) private var scheme

    var body: some View {
        let dark = scheme == .dark
        let ink = dark ? Color(red: 0.97, green: 0.94, blue: 0.86) : Color(red: 0.12, green: 0.11, blue: 0.2)
        let paper = dark ? Color(red: 0.07, green: 0.065, blue: 0.14) : Color(red: 0.97, green: 0.95, blue: 0.89)
        GeometryReader { proxy in
            let diameter = min(224, proxy.size.width * 0.60, proxy.size.height * 0.43)
            VStack(spacing: 0) {
                ZStack {
                    Orbit().foregroundStyle(ink.opacity(dark ? 0.35 : 0.32))
                    ConnectionFlight.Frame(step: 0, fontSize: diameter / 14)
                        .environment(\.layoutDirection, .leftToRight)
                }
                .frame(width: diameter, height: diameter)
                Text(verbatim: "SPACEPORT")
                    .font(.system(size: min(28, diameter / 7), weight: .heavy, design: .monospaced))
                    .tracking(3.5)
                    .padding(.top, 12)
            }
            .foregroundStyle(ink)
            .frame(maxWidth: .infinity, maxHeight: .infinity)
        }
        .background(paper)
        .ignoresSafeArea()
        .accessibilityElement(children: .ignore)
        .accessibilityLabel("Spaceport")
    }

    private struct Orbit: View {
        var body: some View {
            ZStack {
                Circle().strokeBorder(lineWidth: 0.75)
                Circle().inset(by: 5).stroke(style: StrokeStyle(lineWidth: 1.5, dash: [1, 8]))
                GeometryReader { proxy in
                    let d = proxy.size.width
                    // Tilt the flight path around the rocket and mark one point on its orbit.
                    ZStack(alignment: .trailing) {
                        Ellipse().stroke(lineWidth: 0.75)
                        Circle().fill(.orange).frame(width: 5, height: 5).offset(x: 2.5)
                    }
                    .frame(width: d * 1.23, height: d * 0.36)
                    .rotationEffect(.degrees(-24))
                    .position(x: d / 2, y: d / 2)
                }
            }
            .accessibilityHidden(true)
        }
    }
}
