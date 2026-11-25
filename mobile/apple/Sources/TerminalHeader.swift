// ABOUTME: Identifies the pane above its terminal and exposes session navigation.
// ABOUTME: Shows a waiting agent on the folded display and pane controls on the open display.
import SwiftUI

struct TerminalHeader: View {
    let view: MobileView?
    let location: String
    let folded: Bool
    let composing: Bool
    let navigate: () -> Void
    let select: (GoToPane) -> Void
    let find: () -> Void
    let home: () -> Void
    let compose: () -> Void

    nonisolated static func nextWaiting(_ agents: [GoToPane], focused: String?) -> GoToPane? {
        agents.first { $0.id != focused && PaneActivity($0.pane.status) == .blocked }
    }

    var body: some View {
        HStack(spacing: 2) {
            let workspace = view?.workspaces.first { $0.id == view?.focusedWorkspace }
            let tab = view?.tabs.first { $0.id == view?.focusedTab && $0.workspaceId == workspace?.id }
            let pane = view?.panes.first { $0.id == view?.focusedPane && $0.tabId == tab?.id }
            locationButton(pane, path: [workspace?.label, tab?.label].compactMap { $0 }.joined(separator: " › "))
            if composing {
                Text("Live").font(.caption).foregroundStyle(SessionChrome.secondary)
            } else {
                if let view, folded {
                    let agents = GoToPane.agents(workspaces: view.workspaces, tabs: view.tabs, panes: view.panes)
                    if let next = Self.nextWaiting(agents, focused: view.focusedPane) { waitingButton(next) }
                } else if !folded {
                    if let view, let workspace, let tab, let map = view.paneMap, map.panes.count >= 2 {
                        let panes = Dictionary(view.panes.filter { $0.tabId == tab.id }.map { ($0.id, $0) },
                                               uniquingKeysWith: { first, _ in first })
                        paneTiles(map.panes.compactMap { panes[$0.id] }, in: workspace, tab: tab)
                    }
                    Button(action: find) {
                        Image(systemName: "magnifyingglass").frame(width: 44, height: 44).contentShape(Rectangle())
                    }
                    .disabled(view?.focusedPaneTarget == nil)
                    .accessibilityLabel("Find in Terminal")
                    .accessibilityIdentifier("session.header.find")
                }
                Button(action: home) {
                    Image(systemName: "house").frame(width: 44, height: 44).contentShape(Rectangle())
                }
                .accessibilityLabel("Home")
                .accessibilityIdentifier("session.header.home")
                Button(action: compose) {
                    Image(systemName: "square.and.pencil").frame(width: 44, height: 44).contentShape(Rectangle())
                }
                .accessibilityLabel("Compose")
                .accessibilityIdentifier("session.compose")
            }
        }
        .buttonStyle(.plain)
        .padding(.horizontal, 12)
        .frame(minHeight: folded ? 50 : 44)
        .background(SessionChrome.terminal)
        .overlay(alignment: .bottom) { Divider() }
    }

    private func locationButton(_ pane: MobilePane?, path: String) -> some View {
        let name = pane?.agentName ?? pane?.label ?? location
        let activity = pane.flatMap { PaneActivity($0.status) }
        return Button(action: navigate) {
            HStack(spacing: 7) {
                if let activity { activity.sessionDot }
                let title = Text(verbatim: name).font(.subheadline.weight(.semibold))
                if folded {
                    VStack(alignment: .leading, spacing: 2) {
                        title
                        HStack(spacing: 3) {
                            if let activity { Text(activity.title).foregroundStyle(activity.sessionTextColor) }
                            if activity != nil && !path.isEmpty { Text("·") }
                            if !path.isEmpty { Text(verbatim: path) }
                        }.font(.caption).foregroundStyle(SessionChrome.secondary)
                    }
                    Image(systemName: "chevron.down").font(.caption2)
                } else {
                    title.layoutPriority(1)
                    if !path.isEmpty {
                        Text(verbatim: path).font(.caption).foregroundStyle(SessionChrome.secondary)
                            .truncationMode(.middle)
                    }
                }
                Spacer(minLength: 0)
            }
            .lineLimit(1)
            .frame(maxWidth: .infinity, minHeight: 44, alignment: .leading)
            .contentShape(Rectangle())
        }
        .accessibilityLabel("Agents")
        .accessibilityValue(Text(verbatim: [name, activity?.title, path].compactMap { $0 }.filter { !$0.isEmpty }.joined(separator: ", ")))
        .accessibilityHint(folded ? "Opens Go To." : "Opens the agents sidebar.")
        .accessibilityIdentifier("session.navigation")
    }

    private func waitingButton(_ next: GoToPane) -> some View {
        let name = next.pane.agentName ?? next.pane.label
        return Button { select(next) } label: {
            HStack(spacing: 4) {
                PaneActivity.blocked.sessionDot
                Text(verbatim: name).font(.caption.weight(.semibold)).lineLimit(1)
                Image(systemName: "chevron.right").font(.caption2)
            }
            .padding(.horizontal, 9).frame(height: 30)
            .foregroundStyle(PaneActivity.blocked.sessionTextColor)
            .background(Color(red: 58/255, green: 26/255, blue: 38/255), in: Capsule())
            .frame(maxWidth: 112, minHeight: 44).contentShape(Rectangle())
        }
        .accessibilityLabel(Text(verbatim: "\(name) needs you"))
        .accessibilityHint("Opens this agent’s pane.")
        .accessibilityIdentifier("session.header.waiting")
    }

    private func paneTiles(_ panes: [MobilePane], in workspace: MobileWorkspace, tab: MobileTab) -> some View {
        ScrollViewReader { reader in
            ScrollView(.horizontal) {
                HStack(spacing: 0) {
                    ForEach(panes, id: \.id) { pane in
                        let focused = pane.id == view?.focusedPane
                        let activity = PaneActivity(pane.status)
                        Button { select(GoToPane(workspace: workspace, tab: tab, pane: pane)) } label: {
                            RoundedRectangle(cornerRadius: 4)
                                .fill(focused ? SessionChrome.selected : SessionChrome.sidebar)
                                .strokeBorder(focused ? activity?.sessionColor ?? .white : SessionChrome.field)
                                .frame(width: 22, height: 30).frame(width: 44, height: 44)
                                .contentShape(Rectangle())
                        }
                        .accessibilityLabel(Text(verbatim: pane.label))
                        .accessibilityAddTraits(focused ? .isSelected : [])
                        .accessibilityIdentifier("session.header.pane.\(pane.id)")
                        .id(pane.id)
                    }
                }
            }
            .scrollIndicators(.hidden)
            .frame(width: min(CGFloat(panes.count) * 44, 132))
            .onChange(of: view?.focusedPane, initial: true) { _, pane in
                if let pane { reader.scrollTo(pane, anchor: .center) }
            }
        }
    }
}
