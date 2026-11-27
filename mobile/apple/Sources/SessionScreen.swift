// ABOUTME: Attaches an SSH profile and renders one live Herdr pane on the phone.
// ABOUTME: Keeps navigation, login decisions and approvals scoped to this session.
import SwiftUI
import UIKit

struct SessionScreen: View {
    @ObservedObject private var connection: MobileConnection
    private let presentation: UUID
    private let hostDetails: () -> Void
    private let home: () -> Void
    private let disconnect: () -> Void
    @Environment(\.dismiss) private var dismiss
    @Environment(\.accessibilityReduceMotion) private var reduceMotion
    @Environment(\.scenePhase) private var scenePhase
    @Environment(\.horizontalSizeClass) private var horizontalSizeClass
    @Environment(\.verticalSizeClass) private var verticalSizeClass
    @Environment(\.usesVerticalSessionToolbar) private var verticalToolbar
    @State private var revealAgentsID: UUID?
    @State private var sidebarCollapsed = false
    @State private var expandedWhileTyping = false
    @State private var interrupt = WaitingInterrupt()
    @State private var continuity: Continuity?
    @State private var historyTarget: InputTarget?

    private struct Continuity: Identifiable {
        let id = UUID()
        let message: String
    }
    @State private var sidebarRequest: GoToSheet.Jump?
    @State private var shortcutLayout: UUID?
    @State private var showEmptyAgents = false
    @Environment(\.displayScale) private var displayScale
    @State private var keyboard = false
    @State private var measured = false
    @State private var navigation: GoToSection?
    @State private var navigationExit: NavigationExit?
    @State private var keyboardBeforeNavigation: Bool?
    @State private var keyboardReturnTarget: InputTarget?
    @State private var requestedNavigation: GoToSection?
    @State private var composing = false
    @State private var modifiers: UInt8 = 0
    @State private var selectedText: String?
    private enum NavigationExit { case home, disconnect }

    init(presentation: SessionPresentation, initialNavigation: GoToSection? = nil,
         hostDetails: @escaping () -> Void, home: @escaping () -> Void, disconnect: @escaping () -> Void) {
        self.presentation = presentation.id
        self.hostDetails = hostDetails
        self.home = home
        self.disconnect = disconnect
        _requestedNavigation = State(initialValue: initialNavigation)
        _connection = ObservedObject(wrappedValue: presentation.connection)
    }

    private var showsTerminal: Bool {
        connection.phase == .ready || connection.phase == .locatingAgent || connection.hasParkedSession
            || (connection.phase == .connecting && connection.reconnectFrame != nil)
    }

    var body: some View {
        VStack(spacing: 0) {
            switch connection.phase {
            case .credentials, .closed: loginForm
            case .trust:
                if let challenge = connection.challenge {
                    VerifyHostView(address: connection.profile.address, challenge: challenge,
                                   trustLabel: "Trust and Connect", trust: { if connection.presentationID == presentation { connection.trust() } },
                                   cancel: { dismiss() }, openHostDetails: hostDetails)
                }
            case .connecting where !showsTerminal: transition
            case .ready, .locatingAgent, .connecting:
                terminal
            }
        }
        .navigationTitle(showsTerminal ? (verticalToolbar ? "" : connection.profile.session) : "Connect")
        .navigationBarTitleDisplayMode(.inline)
        .navigationBarBackButtonHidden(showsTerminal && verticalToolbar)
        .toolbar(showsTerminal && (!verticalToolbar || composing) ? .hidden : .visible, for: .navigationBar)
        .sheet(isPresented: $selectedText.presented) {
            NavigationStack {
                SelectableTerminalText(text: selectedText ?? "")
                    .navigationTitle("Terminal text")
                    .navigationBarTitleDisplayMode(.inline)
                    .toolbar { Button("Done") { selectedText = nil } }
            }
        }
        .sheet(item: $navigation, onDismiss: finishNavigation) {
            GoToSheet(connection: connection, presentation: presentation, initialSection: $0, showEmptyAgents: showEmptyAgents,
                      disconnect: { leaveNavigation(.disconnect) }, home: { leaveNavigation(.home) }, onArrival: {
                keyboardReturnTarget = connection.view?.focusedPaneTarget
            }).id(shortcutLayout)
        }
        .sheet(isPresented: Binding(get: { historyTarget != nil }, set: { if !$0 { historyTarget = nil } }), onDismiss: finishNavigation) {
            if let target = historyTarget {
                NavigationStack {
                    TerminalHistoryView(connection: connection, presentation: presentation, target: target, done: { historyTarget = nil })
                }
            }
        }
        .alert("Session", isPresented: $connection.message.presented) {
            Button("OK") { connection.message = nil }
        } message: { Text(connection.message ?? "") }
        .onChange(of: connection.phase, initial: true) { _, phase in
            if phase == .ready, let requestedNavigation {
                openNavigation(requestedNavigation)
                self.requestedNavigation = nil
            }
        }
        .overlay(alignment: .top) {
            if showsTerminal, let notice = connection.enrollmentNotice {
                HStack {
                    Text(notice).font(.footnote)
                    Button { connection.enrollmentNotice = nil } label: {
                        Image(systemName: "xmark.circle.fill")
                    }.accessibilityLabel("Dismiss Phone Key notice")
                }
                .padding().background(.regularMaterial, in: .rect(cornerRadius: 16)).padding()
            }
        }
        .onDisappear { keyboard = false }
        .onChange(of: scenePhase) { _, phase in
            if phase == .background {
                keyboardBeforeNavigation = nil
                keyboardReturnTarget = nil
                navigationExit = nil
                navigation = nil
                historyTarget = nil
                continuity = nil
                interrupt.dismiss()
                composing = false
                selectedText = nil
                keyboard = false
                modifiers = 0
            }
        }
        .background {
            SessionPrivacyWindow(concealed: scenePhase != .active).allowsHitTesting(false)
        }
    }

    private var transition: some View {
        ConnectionWait {
            VStack(spacing: 8) {
                Text(connection.phase == .locatingAgent ? "Opening agent…" : "Connecting to Herdr…")
                    .font(.headline)
                if let agent = connection.openingAgent {
                    let path = agent.locationLabels
                    AgentSelectorLabel(name: agent.name, terminalTitle: agent.terminalTitle, workspace: path.workspace,
                                       tab: path.tab, font: .subheadline, paneTitle: agent.paneTitle, address: connection.profile.address)
                }
                if showsTerminal { Button("Home", action: home) }
            }
        }
        .padding(.horizontal)
        .frame(maxWidth: .infinity, maxHeight: .infinity)
        .background(Color(uiColor: .systemBackground))
    }

    private var loginForm: some View {
        Form {
            Section {
                VStack(spacing: 10) {
                    Image(systemName: connection.phase == .closed ? "bolt.slash" : "lock.fill")
                        .font(.title).foregroundStyle(.tint)
                        .frame(width: 56, height: 56).background(.tint.opacity(0.12), in: .rect(cornerRadius: 16))
                    Text(connection.phase == .closed ? "Disconnected from \(connection.profile.address.endpoint)" : "Sign in to \(connection.profile.address.endpoint)")
                        .font(.title2.bold()).multilineTextAlignment(.center)
                    Text("Account \(connection.profile.username)")
                        .font(.callout.monospaced()).foregroundStyle(.secondary)
                    if connection.phase == .closed {
                        Text("Herdr keeps your agents running.").font(.footnote).foregroundStyle(.secondary)
                    }
                }
                .frame(maxWidth: .infinity)
            }.listRowBackground(Color.clear)
            Section {
                Picker("Login method", selection: Binding(get: { connection.method }, set: { connection.chooseMethod($0, from: presentation) })) {
                    ForEach(LoginMethod.allCases, id: \.self) { Text($0.title).tag($0) }
                }.pickerStyle(.segmented)
                if connection.method == .password {
                    SecureField("SSH password", text: $connection.password)
                        .textContentType(.password).onSubmit(signIn)
                    Toggle("Authorize This Phone", isOn: $connection.authorizeThisPhone)
                        .disabled(!SecurePhoneKey.isAvailable)
                }
            } header: { Text("Sign In With") } footer: {
                if connection.method == .phoneKey {
                    Text("This host must trust your Secure Enclave public key. Install it from Settings → Phone Key.")
                } else if !SecurePhoneKey.isAvailable {
                    Text("Phone Key requires a device with a Secure Enclave. You can sign in with your password.")
                } else if connection.authorizeThisPhone {
                    Text("Install this phone’s public key for this SSH account and verify a key-only login. Future sign-ins will use Phone Key.")
                } else {
                    Text("Sign in with your password for this session.")
                }
            }
            Section {
                Button(action: signIn) {
                    Text(connection.phase == .closed ? "Reconnect" : "Sign In").frame(maxWidth: .infinity, minHeight: 44)
                }
                .buttonStyle(.borderedProminent).buttonBorderShape(.capsule)
                .disabled(connection.method == .password && connection.password.isEmpty)
                if connection.phase == .closed {
                    Button("Copy connection diagnostics", systemImage: "doc.on.doc") {
                        UIPasteboard.general.string = connection.diagnosticReport()
                    }
                    Button("Back to Sessions", action: home).frame(maxWidth: .infinity)
                }
            }.listRowBackground(Color.clear)
        }
    }

    private func signIn() {
        guard connection.presentationID == presentation,
              connection.method == .phoneKey || !connection.password.isEmpty else { return }
        connection.connect()
    }

    @AppStorage(TerminalPreferences.textSize) private var savedTextSize: Int?
    private var showsDirectPane: Bool { connection.view?.terminal == nil }
    private var metrics: TerminalMetrics { TerminalMetrics(textSize: savedTextSize ?? TerminalPreferences.defaultTextSize) }

    private var terminal: some View {
        GeometryReader { geometry in
            let width = geometry.size.width
            let regular = horizontalSizeClass == .regular && verticalSizeClass != .compact
            let expandedWidth = SessionLayout.sidebarWidth(in: width, regular: regular)
            let wide = expandedWidth != nil
            let sidebar = SessionLayout.sidebar(width: width, regular: regular,
                                               keyboard: keyboard && !expandedWhileTyping,
                                               composing: composing, userCollapsed: sidebarCollapsed)
            let sidebarAnimation = sidebar.transitionAnimation(reduceMotion: reduceMotion)
            let column = composing && wide ? SessionLayout.composerWidth : sidebar.width
            let useBottomInset = wide && !keyboard && !composing
            ZStack(alignment: .bottomLeading) {
                HStack(spacing: 0) {
                    if sidebar != .hidden || sidebarRequest != nil {
                        GoToSheet(connection: connection, presentation: presentation, initialSection: .agents,
                                  disconnect: disconnect, home: home,
                                  sidebar: .init(hide: { sidebarCollapsed = true; expandedWhileTyping = false },
                                                 expand: expandSidebar, hostDetails: hostDetails, rail: sidebar == .rail,
                                                 expandedWidth: expandedWidth,
                                                 keyboard: $keyboard, request: $sidebarRequest, revealAgentsID: revealAgentsID, isVisible: sidebar != .hidden))
                            .id(shortcutLayout)
                            .frame(width: sidebar.width, alignment: .leading)
                            .background(SessionChrome.sidebar).clipped()
                            .padding(.bottom, keyboard && !composing ? SessionToolbar.rowHeight : 0)
                            .accessibilityHidden(sidebar == .hidden).allowsHitTesting(sidebar != .hidden)
                            .disabled(connection.phase != .ready)
                    }
                    if composing && wide { Color.clear.frame(width: column) }
                    if column > 0 { Divider() }
                    terminalContent(wide: wide)
                        .padding(.bottom, composing ? 0 : SessionToolbar.rowHeight)
                        .disabled(composing)
                }
                .animation(sidebarAnimation, value: sidebar == .rail)
                .overlay(alignment: .bottomTrailing) {
                    if !composing {
                        let inset = keyboard || column == 0 ? 0 : column + 1
                        SessionToolbar(width: width - inset, keyboard: keyboard, modifiers: $modifiers, sendKey: { key, flags in
                            guard connection.acceptsInput(from: presentation) else { return }
                            guard let target = connection.view?.inputTarget else { connection.inputUnavailable(); return }
                            send(.key(key, flags), target: target)
                        }, toggleKeyboard: {
                            if keyboard { keyboard = false } else { showKeyboard() }
                        }, back: historyAction(forward: false), forward: historyAction(forward: true))
                        .animation(sidebarAnimation) { toolbar in
                            toolbar.frame(width: width - inset)
                        }
                        .disabled(connection.phase != .ready)
                    }
                }
                .accessibilityHidden(composing && !wide)
                // One editor owns marked text, keyboard focus, and an in-flight image across display changes.
                if composing {
                    DraftComposer(draft: $connection.draft, connection: connection, presentation: presentation,
                                  inline: wide, close: { composing = false })
                        .frame(width: wide ? column : width)
                        .padding(.top, wide ? 0 : 8)
                        .background(.black.opacity(wide ? 0 : 0.45))
                        .accessibilityAddTraits(wide ? [] : .isModal)
                }
            }
            .overlay(alignment: .bottom) {
                if let continuity {
                    Label(continuity.message, systemImage: "checkmark")
                        .font(.caption).padding(.horizontal, 12).padding(.vertical, 6)
                        .background(.regularMaterial, in: Capsule())
                        .padding(.bottom, 54).allowsHitTesting(false)
                }
            }
            .padding(.bottom, useBottomInset ? min(12, geometry.safeAreaInsets.bottom) : 0)
            .ignoresSafeArea(.container, edges: useBottomInset ? .bottom : [])
            .onChange(of: scenePhase == .active ? connection.readyShortcutID : nil, initial: true) { _, request in
                if let request { presentShortcut(request, wide: wide) }
            }
            .onChange(of: wide) { _, _ in
                guard connection.phase == .ready, connection.view?.focusedPaneTarget != nil else { return }
                let message = connection.draft.isEmpty ? "Same pane" : "Draft kept"
                continuity = Continuity(message: message)
            }
            .task(id: continuity?.id) {
                guard continuity != nil else { return }
                do { try await Task.sleep(for: .seconds(2)) } catch { return }
                continuity = nil
            }
        }
        .onChange(of: connection.view?.version, initial: true) { _, _ in updateInterrupt() }
        .onChange(of: keyboard) { _, active in
            if !active { expandedWhileTyping = false; interrupt.dismiss() }
        }
    }

    private func expandSidebar() {
        sidebarCollapsed = false
        expandedWhileTyping = keyboard
    }

    private func revealAgents() {
        revealAgentsID = UUID()
        expandSidebar()
    }

    private func presentShortcut(_ id: UUID, wide: Bool) {
        guard connection.readyShortcutID == id,
              let action = connection.takeShortcut(from: presentation), let view = connection.view else { return }
        requestedNavigation = nil
        keyboardBeforeNavigation = nil
        keyboardReturnTarget = nil
        navigationExit = nil
        historyTarget = nil
        selectedText = nil
        composing = false
        keyboard = false
        modifiers = 0
        interrupt.dismiss()
        sidebarRequest = nil
        switch action {
        case .revealTerminal:
            shortcutLayout = id
            navigation = nil
        case .compose:
            shortcutLayout = id
            navigation = nil
            composing = true
        case .nextWaiting:
            let agents = GoToPane.agents(workspaces: view.workspaces, tabs: view.tabs, panes: view.panes)
            if let next = TerminalHeader.nextWaiting(agents, focused: view.focusedPane) {
                navigation = nil
                select(next, in: view, generation: connection.generation)
            } else if agents.contains(where: { $0.isFocused(in: view) && PaneActivity($0.pane.status) == .blocked }) {
                navigation = nil
            } else if wide {
                shortcutLayout = id
                navigation = nil
                revealAgents()
            } else {
                shortcutLayout = id
                openNavigation(.agents, showEmptyAgents: true)
            }
        }
    }

    private func openComposer() {
        keyboard = false
        modifiers = 0
        connection.draft.open(at: connection.draftDestination)
        composing = true
    }

    private func historyAction(forward: Bool) -> (() -> Void)? {
        guard let view = connection.view, let id = forward ? view.forwardPane : view.backPane,
              let pane = view.panes.first(where: { $0.id == id }),
              let tab = view.tabs.first(where: { $0.id == pane.tabId }),
              let workspace = view.workspaces.first(where: { $0.id == tab.workspaceId }) else { return nil }
        let row = GoToPane(workspace: workspace, tab: tab, pane: pane)
        let token = connection.generation
        return { select(row, in: view, generation: token, historyForward: forward) }
    }

    private func select(_ row: GoToPane, in view: MobileView?, generation: UUID, historyForward: Bool? = nil) {
        guard sidebarRequest == nil, let view, view.bootId == connection.view?.bootId,
              let jump = GoToSheet.Jump.begin(to: row, view: view, connection: connection,
                                              presentation: presentation, generation: generation,
                                              expectedAgent: historyForward == nil ? row.pane.agentName : nil, restoreKeyboard: keyboard, historyForward: historyForward) else { return }
        sidebarRequest = jump
        keyboard = false
        modifiers = 0
        interrupt.dismiss()
    }

    private func updateInterrupt() {
        guard let view = connection.view, connection.phase == .ready else {
            interrupt = WaitingInterrupt()
            return
        }
        var waiting: [String: UInt64] = [:]
        for pane in view.panes where pane.agentName != nil && PaneActivity(pane.status) == .blocked {
            waiting[pane.id] = pane.stateChangeSeq ?? 0
        }
        let previous = interrupt.pane
        interrupt.update(boot: view.bootId, waiting: waiting, focused: view.focusedPane,
                         typing: keyboard && !composing && sidebarRequest == nil && scenePhase == .active)
        if previous != interrupt.pane, let row = interruptRow {
            UIAccessibility.post(notification: .announcement, argument: "\(row.pane.agentName ?? row.pane.label) needs you")
        }
    }

    private var interruptRow: GoToPane? {
        guard let id = interrupt.pane, let view = connection.view else { return nil }
        return GoToPane.agents(workspaces: view.workspaces, tabs: view.tabs, panes: view.panes).first { $0.id == id }
    }

    private func terminalContent(wide: Bool) -> some View {
        VStack(spacing: 0) {
            let view = connection.view
            let generation = connection.generation
            let openAgents = {
                guard connection.acceptsInput(from: presentation), generation == connection.generation, !composing else { return }
                if wide { revealAgents() } else { openNavigation(.agents) }
            }
            TerminalHeader(view: view, location: connection.draftDestination?.name ?? connection.profile.session,
                           address: connection.profile.address, folded: !wide, composing: composing, navigate: openAgents,
                           select: { select($0, in: view, generation: generation) }, find: {
                guard generation == connection.generation, let target = view?.focusedPaneTarget,
                      target == connection.view?.focusedPaneTarget else { return }
                keyboardBeforeNavigation = keyboard
                keyboardReturnTarget = target
                keyboard = false
                modifiers = 0
                historyTarget = target
            }, home: home, newTerminal: {
                guard let view, let workspace = view.focusedWorkspace, let tab = view.focusedTab,
                      generation == connection.generation else { return }
                connection.createTab(workspace: workspace, sourceTab: tab, boot: view.bootId, from: presentation)
            }, compose: openComposer)
            .disabled(connection.phase != .ready)
            if keyboard, let row = interruptRow {
                HStack {
                    VStack(alignment: .leading) {
                        Text(verbatim: "\(row.pane.agentName ?? row.pane.label) needs you").font(.subheadline.bold())
                        Text(verbatim: "\(row.location) · Your typing stays here").font(.caption)
                    }
                    Spacer()
                    Button("Later") { interrupt.dismiss() }
                        .frame(minHeight: 44).accessibilityIdentifier("session.interrupt.later")
                    Button("Jump") { select(row, in: view, generation: generation) }
                        .buttonStyle(.borderedProminent).accessibilityIdentifier("session.interrupt.jump")
                }
                .padding(.horizontal, 10)
                .background(SessionChrome.waitingBackground, in: RoundedRectangle(cornerRadius: 12))
                .padding(.horizontal, 6)
            }
            GeometryReader { proxy in
                ZStack(alignment: .topLeading) {
                    TerminalDisplay.background
                    if connection.phase == .connecting && (connection.reconnectFrame != nil || connection.hasParkedSession) {
                        if let lastFrame = connection.reconnectFrame {
                            TerminalDisplay(terminal: lastFrame, metrics: metrics)
                                .opacity(0.4).accessibilityHidden(true)
                        }
                        ConnectionWait { Text("Reconnecting…").font(.headline) }
                            .padding(.horizontal, 16).padding(.vertical, 10)
                            .background(.regularMaterial, in: .rect(cornerRadius: 16))
                            .frame(maxWidth: .infinity, maxHeight: .infinity)
                    } else if connection.phase != .ready {
                        transition
                    } else if showsDirectPane {
                        if let pane = connection.directPane {
                            DirectPaneDisplay(pane: pane, keyboard: $keyboard, modifiers: $modifiers,
                                              retry: connection.retryDirectPane, openAgents: openAgents)
                                .id(pane.id)
                        } else {
                            ConnectionWait { Text("Waiting for the terminal…").font(.headline) }
                                .frame(maxWidth: .infinity, maxHeight: .infinity)
                        }
                    } else if let surface = connection.view?.terminal {
                        let target = connection.view?.inputTarget
                        let display = TerminalDisplay(terminal: surface, metrics: metrics)
                        let arrow: (String) -> Void = { key in
                            guard let target else { connection.inputUnavailable(); return }
                            send(.key(key, 0), target: target)
                        }
                        display
                            .overlay {
                                TerminalGestures(tap: showKeyboard, openAgents: openAgents, scroll: { start, delta in
                                    scroll(surface, size: proxy.size, target: target, start: start, delta: delta)
                                }, arrow: arrow, actions: { point in
                                    let word = display.word(at: point, in: proxy.size)
                                    let copy = UIAction(title: "Copy", image: UIImage(systemName: "doc.on.doc"),
                                                        attributes: word == nil ? .disabled : []) { _ in
                                        if let word { UIPasteboard.general.string = word.text }
                                    }
                                    let paste = UIAction(title: "Paste", image: UIImage(systemName: "doc.on.clipboard")) { _ in
                                        self.paste(to: target)
                                    }
                                    let all = UIAction(title: "Select All", image: UIImage(systemName: "selection.pin.in.out")) { _ in
                                        selectedText = display.visibleText
                                    }
                                    return (UIMenu(children: [copy, paste, all]), word?.rect)
                                })
                                // Retire the gesture along with its pane, including during a projection gap.
                                .id(target)
                                .accessibilityHidden(true)
                            }
                            .accessibilityAction { showKeyboard() }
                            .accessibilityAction(named: "Arrow up") { arrow("Up") }
                            .accessibilityAction(named: "Arrow down") { arrow("Down") }
                            .accessibilityAction(named: "Arrow left") { arrow("Left") }
                            .accessibilityAction(named: "Arrow right") { arrow("Right") }
                            .accessibilityAction(named: "Select terminal text") { selectedText = display.visibleText }
                            .accessibilityAction(named: "Paste") { paste(to: target) }
                            .accessibilityAction(named: "Scroll up") { if connection.acceptsInput(from: presentation) { connection.scroll(target: target, lines: 8) } }
                            .accessibilityAction(named: "Scroll down") { if connection.acceptsInput(from: presentation) { connection.scroll(target: target, lines: -8) } }
                    } else {
                        Text(connection.view?.panes.isEmpty == true ? "No terminal panes in this session." : "Waiting for the terminal…")
                            .font(.callout).foregroundStyle(.secondary).padding()
                    }
                }
                .contentShape(Rectangle())
                .overlay(alignment: .bottom) {
                    if let status = connection.inputStatus {
                        Text(status).font(.footnote).multilineTextAlignment(.center)
                            .padding(.horizontal, 12).padding(.vertical, 8)
                            .background(.regularMaterial, in: Capsule()).padding(8)
                            .accessibilityAddTraits(.isStaticText)
                    }
                }
                .task(id: viewport(in: proxy.size)) {
                    guard let size = viewport(in: proxy.size) else { return }
                    if measured {
                        // As on desktop, let a keyboard or rotation animation settle before resizing the host tab.
                        do { try await Task.sleep(for: .milliseconds(150)) } catch { return }
                    }
                    measured = true
                    if connection.acceptsInput(from: presentation) { connection.resize(size) }
                }
            }
            if !showsDirectPane {
                let editorTarget = connection.editorTarget
                TerminalInput(target: editorTarget, keyboard: keyboard && connection.phase == .ready, pointSize: metrics.pointSize, ready: connection.view?.inputTarget != nil,
                              send: { input, target in send(input, target: target) },
                              dismissed: { if editorTarget == connection.editorTarget { keyboard = false } })
                    .id(editorTarget)
                    .frame(height: keyboard ? metrics.cell.height + 5 : 1)
                    .background(TerminalDisplay.background)
                    .accessibilityHidden(!keyboard || connection.phase != .ready)
            }
        }
        .background(TerminalDisplay.background.ignoresSafeArea())
        .onChange(of: connection.editorTarget) { _, _ in modifiers = 0 }
    }

    private func viewport(in size: CGSize) -> Viewport? {
        guard !showsDirectPane, connection.phase == .ready, keyboardBeforeNavigation == nil, size.width > 0, size.height > 0 else { return nil }
        return metrics.viewport(size: size, scale: displayScale)
    }

    private func openNavigation(_ section: GoToSection, showEmptyAgents: Bool = false) {
        self.showEmptyAgents = showEmptyAgents
        keyboardBeforeNavigation = keyboard
        keyboardReturnTarget = connection.view?.inputTarget
        keyboard = false
        modifiers = 0
        navigation = section
    }

    private func finishNavigation() {
        let restore = keyboardBeforeNavigation == true && keyboardReturnTarget != nil
            && keyboardReturnTarget == connection.view?.inputTarget
        keyboardBeforeNavigation = nil
        keyboardReturnTarget = nil
        if let exit = navigationExit {
            navigationExit = nil
            guard connection.presentationID == presentation else { return }
            switch exit {
            case .home: home()
            case .disconnect: disconnect()
            }
            return
        }
        guard restore, scenePhase == .active, connection.acceptsInput(from: presentation) else { return }
        keyboard = true
    }

    private func leaveNavigation(_ exit: NavigationExit) {
        // Finish dismissing Go To before removing the session that presented it.
        navigationExit = exit
        navigation = nil
    }

    private func showKeyboard() {
        guard connection.acceptsInput(from: presentation) else { return }
        guard connection.editorTarget != nil else { connection.inputUnavailable(); return }
        if !showsDirectPane { connection.claimSize() }
        keyboard = true
    }

    private func paste(to target: InputTarget?) {
        guard connection.acceptsInput(from: presentation) else { return }
        guard let target, target == connection.view?.inputTarget else { connection.inputUnavailable(); return }
        if let text = UIPasteboard.general.string { send(.paste(text), target: target) }
    }

    private func scroll(_ surface: MobileTerminal, size: CGSize, target: InputTarget?, start: CGPoint, delta: CGSize) {
        guard connection.acceptsInput(from: presentation) else { return }
        guard abs(delta.height) > abs(delta.width), surface.width > 0, surface.height > 0 else { return }
        let fitted = metrics.fittedCell(in: size, columns: Int(surface.width), rows: Int(surface.height))
        let w = fitted.width
        let h = fitted.height
        guard w > 0, h > 0 else { return }
        let column = UInt16(min(CGFloat(surface.width - 1), max(0, start.x / w)))
        let row = UInt16(min(CGFloat(surface.height - 1), max(0, start.y / h)))
        let distance = max(1, min(32, Int(abs(delta.height) / h)))
        connection.scroll(target: target, lines: Int16(delta.height > 0 ? distance : -distance), column: column, row: row)
    }

    private func send(_ input: MobileConnection.Input, target: InputTarget) {
        switch input {
        case let .text(text) where modifiers != 0 && text.count == 1:
            connection.send(.key(text, modifiers), to: target, from: presentation)
        case let .key(key, existing): connection.send(.key(key, modifiers | existing), to: target, from: presentation)
        default: connection.send(input, to: target, from: presentation)
        }
        modifiers = 0
    }
}

private struct SelectableTerminalText: UIViewRepresentable {
    let text: String
    func makeUIView(context: Context) -> UITextView {
        let view = UITextView()
        view.isEditable = false
        view.isSelectable = true
        view.font = .monospacedSystemFont(ofSize: 15, weight: .regular)
        view.textContainerInset = UIEdgeInsets(top: 12, left: 12, bottom: 12, right: 12)
        return view
    }
    func updateUIView(_ view: UITextView, context: Context) {
        if view.text != text {
            view.text = text
            view.selectedRange = NSRange(location: 0, length: (text as NSString).length)
        }
    }
}
