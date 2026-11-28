// ABOUTME: Fits configured terminal keys and the keyboard toggle to the available width.
// ABOUTME: Docks controls above the keyboard and preserves armed modifiers across key pages.
import SwiftUI

struct SessionToolbar: View {
    static let rowHeight: CGFloat = 46

    let width: CGFloat
    let keyboard: Bool
    @Binding var modifiers: UInt8
    let sendKey: (String, UInt8) -> Void
    let toggleKeyboard: () -> Void
    var back: (() -> Void)?
    var forward: (() -> Void)?
    @AppStorage(TerminalPreferences.toolbarKeys) private var keys = TerminalKeyPages.standard
    @State private var keyPage = 0

    var body: some View {
        Group {
            if #available(iOS 26.0, *) {
                GlassEffectContainer(spacing: 0) { controls }
            } else {
                controls
            }
        }
        .padding(.horizontal, 8)
        .buttonStyle(.plain)
        .foregroundStyle(.primary)
        .background {
            if keyboard { SessionChrome.keyboard.ignoresSafeArea(edges: .bottom) }
        }
        .overlay(alignment: .top) { if keyboard { Divider() } }
        .onChange(of: keys) { _, updated in keyPage = min(keyPage, updated.pages.count - 1) }
    }

    private var controls: some View {
        let allKeys = Array(keys.pages.joined())
        return HStack(spacing: 6) {
            if let back { trailButton("Previous pane", symbol: "chevron.backward", action: back) }
            if let forward { trailButton("Next pane in history", symbol: "chevron.forward", action: forward) }
            Group {
                switch SessionLayout.keyRow(width: width - (back == nil ? 0 : 46) - (forward == nil ? 0 : 46), keyCount: allKeys.count) {
                case .compact:
                    compactKeyStrip
                case .paged:
                    keyStrip
                case .flat:
                    HStack(spacing: 0) {
                        ForEach(Array(allKeys.enumerated()), id: \.offset) { _, key in
                            keycap(key).frame(maxWidth: .infinity)
                        }
                    }
                }
            }
            .frame(maxWidth: .infinity, minHeight: Self.rowHeight, maxHeight: Self.rowHeight, alignment: .leading)
            .terminalCapsule(docked: keyboard)
            Button(action: toggleKeyboard) {
                Image(systemName: keyboard ? "keyboard.chevron.compact.down" : "keyboard")
                    .frame(width: 46, height: Self.rowHeight).contentShape(Rectangle())
            }
            .accessibilityLabel(keyboard ? "Hide keyboard" : "Show keyboard")
            .accessibilityIdentifier("terminal.keyboard.toggle")
            .terminalCapsule(docked: keyboard, interactive: true)
        }
    }

    private func trailButton(_ title: LocalizedStringKey, symbol: String, action: @escaping () -> Void) -> some View {
        Button(action: action) {
            Image(systemName: symbol).frame(width: 40, height: Self.rowHeight).contentShape(Rectangle())
        }
        .accessibilityLabel(Text(title))
        .terminalCapsule(docked: keyboard, interactive: true)
    }

    private var compactKeyStrip: some View {
        HStack(spacing: 0) {
            keycap(.esc)
            if !keys.compactMenuKeys.isEmpty {
                Menu {
                    ForEach(keys.compactMenuKeys, id: \.self) { key in
                        Button { activate(key) } label: {
                            if isArmed(key) { Label(key.title, systemImage: "checkmark") }
                            else { Text(key.title) }
                        }
                    }
                } label: {
                    Image(systemName: "ellipsis").frame(width: 44, height: 44)
                }
                .accessibilityLabel("More terminal keys")
                .accessibilityIdentifier("terminal.keys.more")
            }
        }
    }

    private var keyStrip: some View {
        let page = keys.pages[min(keyPage, keys.pages.count - 1)]
        return TabView(selection: $keyPage) {
            ForEach(Array(keys.pages.enumerated()), id: \.offset) { index, keys in
                HStack(spacing: 0) {
                    ForEach(Array(keys.enumerated()), id: \.offset) { _, key in
                        keycap(key).accessibilityAction(named: "Next key page") {
                            withAnimation { keyPage = (keyPage + 1) % self.keys.pages.count }
                        }
                    }
                }.tag(index)
            }
        }
        .tabViewStyle(.page(indexDisplayMode: .never))
        .frame(width: 132, height: Self.rowHeight)
        .padding(.horizontal, 4)
        .overlay(alignment: .bottom) {
            if keys.pages.count > 1 && !page.contains(where: isArmed) {
                HStack(spacing: 3) {
                    ForEach(keys.pages.indices, id: \.self) { i in
                        Circle().fill(.primary.opacity(i == keyPage ? 0.75 : 0.34)).frame(width: 5, height: 5)
                    }
                }.padding(.bottom, 3).allowsHitTesting(false).accessibilityHidden(true)
            }
        }
    }

    private func isArmed(_ key: TerminalKey) -> Bool {
        if case let .arm(flag) = key.action { return modifiers & flag != 0 }
        return false
    }

    private func activate(_ key: TerminalKey) {
        switch key.action {
        case let .arm(flag): modifiers ^= flag
        case let .key(name, flags): sendKey(name, flags)
        }
    }

    private func keycap(_ key: TerminalKey) -> some View {
        let armed = isArmed(key)
        return Button { activate(key) } label: {
            Text(key.keycap.lowercased().replacingOccurrences(of: "⇧ ", with: "⇧"))
                .font(.system(size: 12, weight: .medium, design: .monospaced))
                .minimumScaleFactor(0.7)
                .lineLimit(1)
                .foregroundStyle(armed ? Color(uiColor: .systemBackground) : .primary)
                .frame(width: 32, height: 32)
                .background(armed ? Color(uiColor: .label) : Color.clear, in: Circle())
                .frame(width: 44, height: 44).contentShape(Rectangle())
        }
        .accessibilityLabel(key.title)
        .accessibilityAddTraits(armed ? .isSelected : [])
        .accessibilityIdentifier("terminal.key." + key.rawValue)
    }
}

private extension View {
    @ViewBuilder func terminalCapsule(docked: Bool, interactive: Bool = false) -> some View {
        if #available(iOS 26.0, *) {
            glassEffect(docked ? .identity : .regular.interactive(interactive), in: Capsule())
        } else if docked {
            self
        } else {
            background(Color(uiColor: .secondarySystemBackground), in: Capsule())
                .overlay { Capsule().strokeBorder(.primary.opacity(0.14), lineWidth: 0.5).allowsHitTesting(false) }
        }
    }
}
