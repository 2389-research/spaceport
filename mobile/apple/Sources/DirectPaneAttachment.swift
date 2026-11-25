// ABOUTME: Connects a Ghostty surface to one input-fenced SSH terminal attachment.
// ABOUTME: Closes the channel when the selection leaves and preserves failures for explicit retry.
import SwiftUI

@MainActor
final class DirectPaneAttachment: ObservableObject, Identifiable {
    let id = UUID()
    let target: InputTarget
    private let channel: MobilePty?
    private(set) var surface: GhosttyPaneSurface?
    private var startup = HerdrAttachStartup()
    private var closed = false
    @Published private(set) var inputWarning: String?
    @Published private(set) var failure: String?

    init(session: MobileSession, target: InputTarget, size: Viewport) {
        self.target = target
        do {
            let pty = try session.attachTerminal(target: target, cols: size.columns, rows: size.rows)
            channel = pty
            let stored = UserDefaults.standard.object(forKey: TerminalPreferences.textSize) as? Int
            surface = GhosttyPaneSurface(write: { [weak self] data in
                do { try pty.write(bytes: data); return true }
                catch {
                    Task { @MainActor in self?.warnInput(error) }
                    return false
                }
            }, resize: { viewport in
                try? pty.resize(cols: viewport.columns, rows: viewport.rows)
            }, textSize: stored)
        } catch {
            channel = nil
            surface = nil
            failure = Self.describe(error)
        }
    }

    func readOutput() async {
        guard let channel, surface != nil, failure == nil, !closed else { return }
        let reader = Task.detached(priority: .userInitiated) { [weak self] in
            do {
                while !Task.isCancelled {
                    let bytes = try channel.readWaiting()
                    guard !Task.isCancelled else { return }
                    if !bytes.isEmpty { await self?.display(bytes) }
                }
            } catch {
                if !Task.isCancelled { await self?.finish(error) }
            }
        }
        await withTaskCancellationHandler {
            await reader.value
        } onCancel: { reader.cancel() }
    }

    private func display(_ bytes: Data) {
        guard !closed else { return }
        let output = startup.receive(bytes)
        if !output.isEmpty { surface?.receive(output) }
    }

    private func finish(_ error: Error) {
        guard !closed else { return }
        let tail = startup.finish()
        if !tail.isEmpty { surface?.receive(tail) }
        failure = Self.describe(error)
        close()
    }

    func send(_ input: MobileConnection.Input) -> Bool {
        guard failure == nil, let surface else { return false }
        switch input {
        case let .key(key, flags): return surface.sendKey(key, modifiers: flags)
        case let .paste(text, pressReturn): return surface.paste(text, pressReturn: pressReturn)
        case .text: return false  // Ghostty handles native typing directly.
        }
    }

    private static func describe(_ error: Error) -> String {
        switch error as? MobileError {
        case .StaleView: return "The selected pane changed."
        case .TerminalUnavailable: return "The SSH host couldn’t start Herdr direct attach."
        case .TerminalLookup: return "Herdr couldn’t find the selected terminal in this session."
        case .TerminalTimeout: return "The SSH host didn’t answer the attach request."
        case .TerminalEnded: return "Herdr closed this terminal attachment."
        case .TerminalStarting: return "The terminal is still opening. Type again when it appears."
        case .TerminalQueueFull: return "The terminal input queue is full. Try again."
        default: return error.localizedDescription
        }
    }

    private func warnInput(_ error: Error) {
        guard !closed else { return }
        inputWarning = Self.describe(error)
    }

    func clearWarning() { inputWarning = nil }

    func close() {
        closed = true
        channel?.close()
    }
}

struct DirectPaneDisplay: View {
    @ObservedObject var pane: DirectPaneAttachment
    @Binding var keyboard: Bool
    @Binding var modifiers: UInt8
    let retry: () -> Void

    var body: some View {
        Group {
            if let failure = pane.failure {
                ContentUnavailableView {
                    Label("Couldn’t attach to this terminal", systemImage: "terminal")
                } description: {
                    Text(failure)
                } actions: {
                    Button("Try Again", action: retry)
                }
            } else if let surface = pane.surface {
                GhosttyPaneView(surface: surface, keyboard: $keyboard, modifiers: $modifiers)
            }
        }
        .overlay(alignment: .bottom) {
            if let warning = pane.inputWarning, pane.failure == nil {
                Text(warning).font(.footnote).padding(8).background(.regularMaterial, in: Capsule())
                    .task(id: warning) {
                        do { try await Task.sleep(for: .seconds(3)) } catch { return }
                        pane.clearWarning()
                    }
            }
        }
        .task(id: pane.id) { await pane.readOutput() }
    }
}
