// ABOUTME: Draws the bundled desktop agent mark for the identity Herdr reports.
// ABOUTME: Keeps original raster colors and lets vector marks inherit the row's text color.
import SwiftUI

struct AgentBadge: View {
    let identity: String?
    var fallbackName: String? = nil

    static func artwork(for identity: String?) -> UIImage? {
        // Herdr sends canonical lowercase identifiers. Only search this app's agent asset namespace.
        guard let identity, !identity.isEmpty, identity.utf8.count <= 32,
              identity.allSatisfy({ $0.isASCII && ($0.isLowercase || $0.isNumber || $0 == "-") }) else {
            return nil
        }
        return UIImage(named: "agent-" + identity, in: .main, compatibleWith: nil)
    }

    var body: some View {
        Group {
            if let mark = Self.artwork(for: identity) {
                Image(uiImage: mark).resizable().scaledToFit()
            } else if let initial = fallbackName?.first {
                Text(verbatim: initial.uppercased()).font(.system(size: 16, weight: .semibold))
            } else {
                Image(uiImage: UIImage(resource: .agentGeneric)).resizable().scaledToFit()
            }
        }.accessibilityHidden(true)
    }
}
