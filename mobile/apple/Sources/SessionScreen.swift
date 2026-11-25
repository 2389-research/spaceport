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
    @Environment(\.scenePhase) private var scenePhase
    @Environment(\.horizontalSizeClass) private var horizontalSizeClass
    @State private var sidebarHidden = false
    @State private var sidebarRequest: GoToSheet.Jump?
    @Environment(\.displayScale) private var displayScale
    @State private var keyboard = false
    @State private var measured = false
    @State private var navigation: GoToSection?
    @State private var keyboardBeforeNavigation: Bool?
    @State private var requestedNavigation: GoToSection?
    @State private var composing = false
    @State private var modifiers: UInt8 = 0
    @State private var selectedText: String?

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
        .navigationTitle(showsTerminal ? connection.profile.session : "Connect")
        .navigationBarTitleDisplayMode(.inline)
        .toolbar(showsTerminal ? .hidden : .visible, for: .navigationBar)
        .preferredColorScheme(showsTerminal ? .dark : nil)
        .sheet(isPresented: $selectedText.presented) {
            NavigationStack {
                SelectableTerminalText(text: selectedText ?? "")
                    .navigationTitle("Terminal text")
                    .navigationBarTitleDisplayMode(.inline)
                    .toolbar { Button("Done") { selectedText = nil } }
            }
        }
        .sheet(item: $navigation, onDismiss: finishNavigation) {
            GoToSheet(connection: connection, presentation: presentation, initialSection: $0, disconnect: disconnect)
        }
        .sheet(isPresented: $composing) { DraftComposer(draft: $connection.draft, connection: connection, presentation: presentation) }
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
                navigation = nil
                composing = false
                selectedText = nil
                keyboard = false
                modifiers = 0
            }
        }
        .overlay {
            if scenePhase != .active {
                Rectangle().fill(.background).overlay { Label("Spaceport", systemImage: "terminal").foregroundStyle(.secondary) }
                    .ignoresSafeArea()
            }
        }
    }

    private var transition: some View {
        VStack(spacing: 12) {
            ProgressView()
            if let agent = connection.openingAgent {
                Text(verbatim: agent.name).font(.headline)
                Text(verbatim: agent.location).font(.subheadline).foregroundStyle(.secondary)
            } else {
                Text(connection.hasParkedSession ? "Resuming session…" : "Connecting to Herdr…").foregroundStyle(.secondary)
            }
            if showsTerminal { Button("Home", action: home) }
        }
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
                    Text(connection.phase == .closed ? "Disconnected" : "Sign in to \(connection.profile.hostname)")
                        .font(.title2.bold()).multilineTextAlignment(.center)
                    Text(verbatim: connection.profile.destination)
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
    @AppStorage(TerminalPreferences.ghostty) private var ghostty = false
    // Direct-mode updates carry a cell frame only when Herdr is displaying a popup.
    private var showsDirectPane: Bool { ghostty && connection.view?.terminal == nil }
    private var metrics: TerminalMetrics { TerminalMetrics(textSize: TerminalPreferences.textSize(saved: savedTextSize, ghostty: ghostty)) }

    private var terminal: some View {
        GeometryReader { geometry in
            let sidebarWidth = SessionLayout.sidebarWidth(in: geometry.size.width, regular: horizontalSizeClass == .regular)
            let visibleWidth = sidebarHidden ? nil : sidebarWidth
            HStack(spacing: 0) {
                // Keep an outstanding selection alive while folding removes its column.
                if visibleWidth != nil || sidebarRequest != nil {
                    GoToSheet(connection: connection, presentation: presentation, initialSection: .agents,
                              disconnect: disconnect, sidebar: .init(hide: { sidebarHidden = true },
                                                                    openPicker: { openNavigation(.agents) }, keyboard: $keyboard,
                                                                    request: $sidebarRequest, isVisible: visibleWidth != nil))
                        .frame(width: visibleWidth ?? 248)
                        .frame(width: visibleWidth ?? 0, alignment: .leading)
                        .clipped()
                        .accessibilityHidden(visibleWidth == nil)
                        .allowsHitTesting(visibleWidth != nil)
                        .disabled(connection.phase != .ready)
                    if visibleWidth != nil { Divider() }
                }
                terminalContent(width: geometry.size.width - (visibleWidth ?? 0), offersSidebar: sidebarWidth != nil)
            }
        }
    }

    private func terminalContent(width: CGFloat, offersSidebar: Bool) -> some View {
        VStack(spacing: 0) {
            GeometryReader { proxy in
                ZStack(alignment: .topLeading) {
                    TerminalDisplay.background
                    if connection.phase == .connecting && (connection.reconnectFrame != nil || connection.hasParkedSession) {
                        if let lastFrame = connection.reconnectFrame {
                            TerminalDisplay(terminal: lastFrame, metrics: metrics)
                                .opacity(0.4).accessibilityHidden(true)
                        }
                        ProgressView("Reconnecting…")
                            .padding(.horizontal, 16).padding(.vertical, 10)
                            .background(.regularMaterial, in: Capsule())
                            .frame(maxWidth: .infinity, maxHeight: .infinity)
                    } else if connection.phase != .ready {
                        transition
                    } else if showsDirectPane {
                        if let pane = connection.directPane {
                            DirectPaneDisplay(pane: pane, keyboard: $keyboard, modifiers: $modifiers,
                                              retry: connection.retryDirectPane)
                                .id(pane.id)
                        } else {
                            ProgressView("Waiting for the terminal…")
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
                                TerminalGestures(tap: showKeyboard, scroll: { start, delta in
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
                            .font(.callout).foregroundStyle(.white.opacity(0.8)).padding()
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
            SessionToolbar(view: connection.view, location: connection.openingAgent?.name ?? connection.draftDestination?.label ?? connection.profile.session,
                           compactKeys: SessionLayout.usesCompactKeys(width: width), offersSidebar: offersSidebar,
                           keyboard: keyboard, modifiers: $modifiers, sendKey: { key, flags in
                guard connection.acceptsInput(from: presentation) else { return }
                guard let target = connection.view?.inputTarget else { connection.inputUnavailable(); return }
                send(.key(key, flags), target: target)
            }, navigate: { section in
                if offersSidebar { sidebarHidden.toggle() }
                else { openNavigation(section) }
            }, home: home, compose: {
                keyboard = false
                modifiers = 0
                connection.draft.open(at: connection.draftDestination)
                composing = true
            }, toggleKeyboard: {
                if keyboard { keyboard = false } else { showKeyboard() }
            })
            .disabled(connection.phase != .ready)
        }
        .background(TerminalDisplay.background.ignoresSafeArea())
        .onChange(of: connection.editorTarget) { _, _ in modifiers = 0 }
    }

    private func viewport(in size: CGSize) -> Viewport? {
        guard !showsDirectPane, connection.phase == .ready, keyboardBeforeNavigation == nil, size.width > 0, size.height > 0 else { return nil }
        return metrics.viewport(size: size, scale: displayScale)
    }

    private func openNavigation(_ section: GoToSection) {
        keyboardBeforeNavigation = keyboard
        keyboard = false
        modifiers = 0
        navigation = section
    }

    private func finishNavigation() {
        let restore = keyboardBeforeNavigation == true
        keyboardBeforeNavigation = nil
        guard restore, scenePhase == .active, connection.acceptsInput(from: presentation) else { return }
        keyboard = true
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
