// ABOUTME: Shows delayed navigation progress and brief continuity feedback in a session.
// ABOUTME: Keeps each notice's timer and visibility local to its overlay.
import SwiftUI

struct GoToProgress: View {
    let name: String
    @State private var visible = false

    var body: some View {
        VStack {
            if visible {
                ProgressView("Opening \(name)…")
                    .lineLimit(1).truncationMode(.middle)
                    .padding(8).frame(maxWidth: .infinity).background(.bar)
            }
        }
        .task {
            do { try await Task.sleep(for: .milliseconds(180)) } catch { return }
            visible = true
        }
    }
}

struct SessionContinuityNotice: View {
    @Environment(\.scenePhase) private var scenePhase
    let wide: Bool
    let messageForTransition: () -> String?

    private struct Notice {
        let id = UUID()
        let message: String
    }
    @State private var notice: Notice?

    var body: some View {
        VStack {
            if let notice {
                Label(notice.message, systemImage: "checkmark")
                    .font(.caption).padding(.horizontal, 12).padding(.vertical, 6)
                    .background(.regularMaterial, in: Capsule())
                    .padding(.bottom, 54)
            }
        }
        .allowsHitTesting(false)
        .onChange(of: wide) { _, _ in
            if let message = messageForTransition() { notice = Notice(message: message) }
        }
        .onChange(of: scenePhase) { _, phase in
            if phase == .background { notice = nil }
        }
        .task(id: notice?.id) {
            guard notice != nil else { return }
            do { try await Task.sleep(for: .seconds(2)) } catch { return }
            notice = nil
        }
    }
}
