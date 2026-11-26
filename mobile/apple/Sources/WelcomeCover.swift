// ABOUTME: Uses the Spaceport paperback cover to introduce the first SSH connection.
// ABOUTME: Keeps setup usable on narrow screens and with larger text.
import SwiftUI

struct WelcomeCover: View {
    @Environment(\.colorScheme) private var scheme
    @Environment(\.dynamicTypeSize) private var dynamicTypeSize
    @ScaledMetric private var titleSize: CGFloat = 34
    let addHost: () -> Void

    var body: some View {
        let navy = Color(red: 23/255, green: 19/255, blue: 46/255)
        let cream = Color(red: 246/255, green: 236/255, blue: 214/255)
        let flame = Color(red: 244/255, green: 116/255, blue: 28/255)
        GeometryReader { proxy in
            let artHeight = proxy.size.height * (dynamicTypeSize.isAccessibilitySize ? 0.18 : 0.34)
            let artSize = max(0, min(236, proxy.size.width - 48, artHeight))
            ScrollView {
                VStack(alignment: .leading, spacing: 14) {
                    Text("Spaceport").textCase(.uppercase)
                        .font(.system(size: titleSize, weight: .heavy)).fontWidth(.expanded)
                        .lineLimit(1).minimumScaleFactor(0.45)
                        .accessibilityAddTraits(.isHeader).accessibilityIdentifier("home.welcome.title")
                    if let painting = UIImage(named: "rocket-560.webp") {
                        Image(uiImage: painting).resizable().scaledToFit()
                            .frame(width: artSize, height: artSize)
                            .overlay { Rectangle().strokeBorder(flame, lineWidth: 2) }
                            .accessibilityHidden(true)
                    }
                    Text("Add your first host").font(.title2.bold())
                    Text("Connect over SSH to a host with a POSIX shell and Herdr installed.")
                        .font(.system(.body, design: .serif))
                    Button("Add Host", systemImage: "plus", action: addHost)
                        .buttonStyle(.borderedProminent).buttonBorderShape(.capsule).controlSize(.large)
                        .tint(flame).foregroundStyle(navy)
                        .accessibilityIdentifier("home.empty.addHost")
                }
                .frame(maxWidth: 430, alignment: .leading).padding(24)
                .frame(maxWidth: .infinity, minHeight: proxy.size.height, alignment: .center)
            }
            .foregroundStyle(scheme == .dark ? cream : navy)
            .background((scheme == .dark ? navy : cream).ignoresSafeArea())
        }
    }
}
