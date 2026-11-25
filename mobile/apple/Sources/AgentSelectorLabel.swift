// ABOUTME: Shows a reported terminal title alongside the agent identity and location.
// ABOUTME: Uses the same labels for live Go To rows and last-seen Home rows.
import SwiftUI

struct AgentSelectorLabel: View {
    let name: String
    let terminalTitle: String?
    let workspace: String
    let tab: String
    var host = ""
    var session = ""
    var detail = ""
    let font: Font
    var isAgent = true

    var title: String? {
        guard let terminalTitle, !terminalTitle.isEmpty, terminalTitle != name else { return nil }
        return terminalTitle
    }

    func subtitle(includingName: Bool) -> AttributedString {
        var result = AttributedString()
        func append(_ text: String, color: Color, separator: String = " · ") {
            guard !text.isEmpty else { return }
            if !result.characters.isEmpty {
                var divider = AttributedString(separator)
                divider.swiftUI.foregroundColor = .secondary
                result += divider
            }
            var part = AttributedString(text)
            part.swiftUI.foregroundColor = color
            result += part
        }
        append(workspace, color: Self.directoryColor)
        append(tab, color: Self.tabColor, separator: " › ")
        append(host, color: Self.hostColor)
        append(session, color: Self.sessionColor)
        append(detail, color: .secondary)
        if includingName { append(name, color: Self.agentColor) }
        return result
    }

    private static let directoryColor = ink(light: 0x166353, dark: 0x9DDAC9)
    private static let tabColor = ink(light: 0x805100, dark: 0xF3CB82)
    private static let hostColor = ink(light: 0x944261, dark: 0xF4B0C3)
    private static let sessionColor = ink(light: 0x345D91, dark: 0xA5C8FF)
    private static let agentColor = ink(light: 0x67418D, dark: 0xD6B8F6)

    private static func ink(light: UInt32, dark: UInt32) -> Color {
        Color(uiColor: UIColor { traits in
            let rgb = traits.userInterfaceStyle == .dark ? dark : light
            return UIColor(red: CGFloat(rgb >> 16) / 255, green: CGFloat((rgb >> 8) & 255) / 255,
                           blue: CGFloat(rgb & 255) / 255, alpha: 1)
        })
    }

    var body: some View {
        VStack(alignment: .leading, spacing: 3) {
            Text(verbatim: title ?? name).font(font).lineLimit(2)
                .foregroundStyle(title == nil && isAgent ? Self.agentColor : Color.primary)
            Text(subtitle(includingName: title != nil))
                .font(.footnote)
        }
    }
}

struct AgentRowBadge: View {
    let identity: String?
    let activity: PaneActivity?
    let background: Color
    let statusColor: Color

    var body: some View {
        ZStack(alignment: .bottomTrailing) {
            AgentBadge(identity: identity)
                .frame(width: 20, height: 20)
                .padding([.bottom, .trailing], 5)
            if let activity {
                activity.agentDot(in: statusColor)
                    .padding(1).background(background, in: Circle())
            }
        }
        .accessibilityHidden(true)
    }
}
