// ABOUTME: Identifies the pane above its terminal and exposes session navigation.
// ABOUTME: Shows a waiting agent on the folded display and pane controls on the open display.
import SwiftUI

extension EnvironmentValues {
    var usesVerticalSessionToolbar: Bool {
        if #available(iOS 27.1, *) { return toolbarVerticalEdge != nil }
        return false
    }
}

struct TerminalHeader: View {
    @Environment(\.usesVerticalSessionToolbar) private var verticalToolbar
    @Environment(\.isEnabled) private var enabled
    let view: MobileView?
    let location: String
    let address: SSHAddress
    let folded: Bool
    let composing: Bool
    let navigate: () -> Void
    let select: (GoToPane) -> Void
    let find: () -> Void
    let home: () -> Void
    let newTerminal: () -> Void
    let compose: () -> Void

    nonisolated static func nextWaiting(_ agents: [GoToPane], focused: String?) -> GoToPane? {
        let waiting = agents.filter { PaneActivity($0.pane.status) == .blocked }
        guard let index = waiting.firstIndex(where: { $0.id == focused }) else { return waiting.first }
        guard waiting.count > 1 else { return nil }
        return waiting[(index + 1) % waiting.count]
    }

    var body: some View {
        let workspace = view?.workspaces.first { $0.id == view?.focusedWorkspace }
        let tab = view?.tabs.first { $0.id == view?.focusedTab && $0.workspaceId == workspace?.id }
        let homeButton = action("Home", symbol: "house", id: "session.header.home", perform: home)
        let newButton = action("New Terminal", symbol: "plus", id: "session.header.newTerminal", perform: newTerminal)
            .disabled(workspace == nil || tab == nil || view?.inputTarget == nil)
            .accessibilityHint("Opens a terminal tab in this workspace.")
        let findButton = action("Find in Terminal", symbol: "magnifyingglass", id: "session.header.find", perform: find)
            .disabled(view?.focusedPaneTarget == nil)
        let composeButton = action("Compose", symbol: "square.and.pencil", id: "session.compose", perform: compose)
        HStack(spacing: 2) {
            let pane = view?.panes.first { $0.id == view?.focusedPane && $0.tabId == tab?.id }
            let first = pane?.displayPaneTitle ?? tab?.label ?? ""
            let path = [workspace?.label, tab?.label == first ? nil : tab?.label].compactMap { $0 }.joined(separator: " › ")
            locationButton(pane, first: first, path: path)
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
                    if !verticalToolbar { findButton }
                }
                if !verticalToolbar {
                    homeButton
                    newButton
                    composeButton
                }
            }
        }
        .labelStyle(HeaderIcon())
        .buttonStyle(.plain)
        .padding(.horizontal, 12)
        .frame(minHeight: folded ? 50 : 44)
        .background(SessionChrome.terminal)
        .overlay(alignment: .bottom) { Divider() }
        .toolbar {
            if verticalToolbar && !composing {
                ToolbarItemGroup(placement: .topBarTrailing) {
                    homeButton
                    newButton
                }
                ToolbarItemGroup(placement: .bottomBar) {
                    findButton
                    composeButton
                }
            }
        }
    }

    private func action(_ title: LocalizedStringKey, symbol: String, id: String, perform: @escaping () -> Void) -> some View {
        Button(action: perform) { Label(title, systemImage: symbol) }
            .accessibilityLabel(Text(title))
            .accessibilityIdentifier(id)
            .disabled(!enabled)
    }

    private struct HeaderIcon: LabelStyle {
        func makeBody(configuration: Configuration) -> some View {
            configuration.icon.frame(width: 44, height: 44).contentShape(Rectangle())
        }
    }

    private func locationButton(_ pane: MobilePane?, first: String, path: String) -> some View {
        let name = pane?.agentName ?? pane?.label ?? location
        let activity = pane.flatMap { PaneActivity($0.status) }
        let agentTitle = folded ? nil : AgentSelectorLabel.displayTitle(pane?.terminalTitle, address: address)
        return Button(action: navigate) {
            HStack(spacing: 7) {
                if let activity { activity.sessionDot }
                let title = Text(AgentSelectorLabel.heading(pane: first, title: name)).font(.subheadline.weight(.semibold))
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
                } else if let agentTitle {
                    Text(verbatim: agentTitle).font(.subheadline.weight(.semibold))
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
        .accessibilityValue(Text(verbatim: [agentTitle, first, first == name || name == agentTitle ? nil : name, activity?.title, path].compactMap { $0 }.filter { !$0.isEmpty }.joined(separator: ", ")))
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
            .background(SessionChrome.waitingBackground, in: Capsule())
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
                                .strokeBorder(focused ? activity?.sessionColor ?? .primary : SessionChrome.field)
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
