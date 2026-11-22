// ABOUTME: Shows a reported terminal title alongside the agent identity and location.
// ABOUTME: Uses the same labels for live Go To rows and last-seen Home rows.
import SwiftUI

struct AgentSelectorLabel: View {
    let name: String
    let terminalTitle: String?
    let location: String
    let font: Font

    var title: String? {
        guard let terminalTitle, !terminalTitle.isEmpty, terminalTitle != name else { return nil }
        return terminalTitle
    }

    var body: some View {
        VStack(alignment: .leading, spacing: 3) {
            Text(verbatim: title ?? name).font(font).lineLimit(2)
            Text(verbatim: title == nil ? location : [name, location].filter { !$0.isEmpty }.joined(separator: " · "))
                .font(.footnote).foregroundStyle(.secondary)
        }
    }
}
