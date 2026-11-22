// ABOUTME: Keeps terminal keys, the focused pane and input controls within one row.
// ABOUTME: Opens Go To without replacing the live terminal or its SSH connection.
import SwiftUI

struct SessionToolbar: View {
    let view: MobileView?
    let location: String
    let keyboard: Bool
    @Binding var modifiers: UInt8
    let sendKey: (String, UInt8) -> Void
    let navigate: (GoToSection) -> Void
    let home: () -> Void
    let compose: () -> Void
    let toggleKeyboard: () -> Void
    @AppStorage(TerminalPreferences.toolbarKeys) private var keys = TerminalKeyPages.standard
    @State private var keyPage = 0

    var body: some View {
        HStack(spacing: 6) {
            keyStrip
            locationButton
            HStack(spacing: 1) {
                let needsYou = view?.panes.contains { $0.agentName != nil && $0.status == "blocked" && $0.id != view?.focusedPane } == true
                Button(action: home) {
                    Image(systemName: "house")
                        .overlay(alignment: .topTrailing) {
                            if needsYou { PaneActivity.blocked.dot.offset(x: 4, y: -3) }
                        }.frame(width: 36, height: 44).contentShape(Rectangle())
                }
                .accessibilityLabel("Home")
                .accessibilityValue(needsYou ? "An agent in another pane needs you" : "")
                .accessibilityIdentifier("session.home")
                icon("Compose", symbol: "square.and.pencil", id: "session.compose", action: compose)
                icon(keyboard ? "Hide keyboard" : "Show keyboard",
                     symbol: keyboard ? "keyboard.chevron.compact.down" : "keyboard",
                     id: "terminal.keyboard.toggle", action: toggleKeyboard)
            }
            .padding(.horizontal, 4).frame(height: 46).terminalCapsule()
        }
        .padding(.horizontal, 8)
        .buttonStyle(.plain)
        .foregroundStyle(.primary)
        .background(TerminalDisplay.background.ignoresSafeArea(edges: .bottom))
        .onChange(of: keys) { _, updated in keyPage = min(keyPage, updated.pages.count - 1) }
    }

    private func icon(_ label: String, symbol: String, id: String, action: @escaping () -> Void) -> some View {
        Button(action: action) {
            Image(systemName: symbol).frame(width: 36, height: 44).contentShape(Rectangle())
        }
        .accessibilityLabel(label).accessibilityIdentifier(id)
    }

    private var locationButton: some View {
        let pane = view?.panes.first { $0.id == view?.focusedPane }
        let tab = view?.tabs.first { $0.id == view?.focusedTab }
        let workspace = view?.workspaces.first { $0.id == view?.focusedWorkspace }
        let path = [workspace?.label, tab?.label].compactMap { $0 }.joined(separator: " › ")
        let name = pane?.agentName ?? pane?.label ?? location
        return Button { navigate(.agents) } label: {
            HStack(spacing: 4) {
                if let pane, let activity = PaneActivity(pane.status) { activity.dot }
                VStack(alignment: .leading, spacing: 0) {
                    Text(verbatim: name).font(.system(size: 14, weight: .semibold))
                    if !path.isEmpty { Text(verbatim: path).font(.system(size: 11)).foregroundStyle(.secondary) }
                }
                .lineLimit(1).truncationMode(.tail)
                Spacer(minLength: 0)
                Image(systemName: "chevron.down").font(.system(size: 9, weight: .semibold))
            }
            .padding(.horizontal, 9).frame(maxWidth: .infinity).frame(height: 46)
            .terminalCapsule().contentShape(Capsule())
        }
        .accessibilityLabel("Agents")
        .accessibilityValue([name, path].filter { !$0.isEmpty }.joined(separator: ", "))
        .accessibilityHint("Opens Go To.")
        .accessibilityIdentifier("session.navigation")
    }

    private var keyStrip: some View {
        let page = keys.pages[min(keyPage, keys.pages.count - 1)]
        return TabView(selection: $keyPage) {
            ForEach(Array(keys.pages.enumerated()), id: \.offset) { index, keys in
                HStack(spacing: 1) {
                    ForEach(Array(keys.enumerated()), id: \.offset) { _, key in keycap(key) }
                }.tag(index)
            }
        }
        .tabViewStyle(.page(indexDisplayMode: .never))
        .frame(width: 110, height: 46)
        .padding(.horizontal, 4)
        .overlay(alignment: .bottom) {
            if keys.pages.count > 1 && !page.contains(where: isArmed) {
                HStack(spacing: 3) {
                    ForEach(keys.pages.indices, id: \.self) { i in
                        Circle().fill(.white.opacity(i == keyPage ? 0.75 : 0.34)).frame(width: 5, height: 5)
                    }
                }.padding(.bottom, 3).allowsHitTesting(false).accessibilityHidden(true)
            }
        }
        .terminalCapsule()
    }

    private func isArmed(_ key: TerminalKey) -> Bool {
        if case let .arm(flag) = key.action { return modifiers & flag != 0 }
        return false
    }

    private func keycap(_ key: TerminalKey) -> some View {
        let armed = isArmed(key)
        return Button {
            switch key.action {
            case let .arm(flag): modifiers ^= flag
            case let .key(name, flags): sendKey(name, flags)
            }
        } label: {
            Text(key.keycap.lowercased().replacingOccurrences(of: "⇧ ", with: "⇧"))
                .font(.system(size: 12, weight: .medium, design: .monospaced))
                .minimumScaleFactor(0.7)
                .lineLimit(1)
                .foregroundStyle(armed ? .black : .white)
                .frame(width: 32, height: 32)
                .background(armed ? Color.white : Color.clear, in: Circle())
                .frame(width: 36, height: 44).contentShape(Rectangle())
        }
        .accessibilityLabel(key.title)
        .accessibilityAddTraits(armed ? .isSelected : [])
        .accessibilityIdentifier("terminal.key." + key.rawValue)
        .accessibilityAction(named: "Next key page") {
            withAnimation { keyPage = (keyPage + 1) % keys.pages.count }
        }
    }
}

private extension View {
    @ViewBuilder func terminalCapsule() -> some View {
        if #available(iOS 26.0, *) {
            glassEffect(.regular, in: Capsule())
        } else {
            background(Color(white: 0.23, opacity: 0.82), in: Capsule())
                .overlay { Capsule().strokeBorder(.white.opacity(0.14), lineWidth: 0.5).allowsHitTesting(false) }
        }
    }
}
