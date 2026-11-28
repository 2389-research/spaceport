// ABOUTME: Presents the privacy artwork above a terminal scene’s navigation, sheets, and alerts.
// ABOUTME: Ties the cover to its session view without taking keyboard focus.
import SwiftUI

struct SessionPrivacyWindow: UIViewRepresentable {
    let concealed: Bool
    @Environment(\.colorScheme) private var scheme

    func makeUIView(context: Context) -> Anchor { Anchor() }

    func updateUIView(_ anchor: Anchor, context: Context) {
        anchor.update(concealed: concealed, style: scheme == .dark ? .dark : .light)
    }

    static func dismantleUIView(_ anchor: Anchor, coordinator: ()) {
        anchor.update(concealed: false, style: .unspecified)
    }

    final class Anchor: UIView {
        private var concealed = false
        private var style = UIUserInterfaceStyle.unspecified
        private var cover: UIWindow?

        func update(concealed: Bool, style: UIUserInterfaceStyle) {
            self.concealed = concealed
            self.style = style
            updateWindow()
        }

        override func didMoveToWindow() {
            super.didMoveToWindow()
            updateWindow()
        }

        private func updateWindow() {
            guard concealed, let scene = window?.windowScene else {
                cover?.isHidden = true
                cover = nil
                return
            }
            if let cover, cover.windowScene === scene {
                cover.overrideUserInterfaceStyle = style
                return
            }
            cover?.isHidden = true
            let shield = UIWindow(windowScene: scene)
            shield.windowLevel = .alert + 1
            shield.overrideUserInterfaceStyle = style
            shield.backgroundColor = .systemBackground
            shield.rootViewController = UIHostingController(rootView: SessionPrivacyCover())
            cover = shield
            shield.isHidden = false
            shield.layoutIfNeeded()
        }
    }
}
