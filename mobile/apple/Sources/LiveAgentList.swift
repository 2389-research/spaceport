// ABOUTME: Updates the live agent picker at a bounded rate during bursts of SSH roster changes.
// ABOUTME: Keeps only navigation rows between updates and reads the current roster when the list opens.
import Combine
import SwiftUI

struct LiveAgentList<Content: View>: View, Equatable {
    let connection: MobileConnection
    let presentation: UUID
    let focus: String?
    let actions: MobilePaneActions?
    let ready: Bool
    let navigating: Bool
    @ViewBuilder let render: ([AgentRail.Section]) -> Content
    @State var sections: [AgentRail.Section]?

    nonisolated static func == (lhs: Self, rhs: Self) -> Bool {
        lhs.connection === rhs.connection && lhs.presentation == rhs.presentation &&
            lhs.focus == rhs.focus && lhs.actions == rhs.actions && lhs.ready == rhs.ready && lhs.navigating == rhs.navigating
    }

    var body: some View {
        VStack(spacing: 0) { render(sections ?? AgentRail.groups(in: connection.view)) }
            .task {
                for await view in connection.$view.throttle(for: .milliseconds(500), scheduler: RunLoop.main, latest: true).values {
                    guard !Task.isCancelled else { return }
                    sections = AgentRail.groups(in: view)
                }
            }
    }
}
