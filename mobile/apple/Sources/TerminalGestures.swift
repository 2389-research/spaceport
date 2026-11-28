// ABOUTME: Routes terminal taps, scroll swipes and held drags to distinct actions.
// ABOUTME: A stationary hold opens the terminal's selection and paste actions.
import SwiftUI
import UIKit

struct TerminalGestures: UIViewRepresentable {
    let tap: () -> Void
    let openAgents: () -> Void
    let scroll: (CGPoint, CGSize) -> Void
    let arrow: (String) -> Void
    let actions: (CGPoint) -> (menu: UIMenu, selection: CGRect?)

    func makeCoordinator() -> Coordinator { Coordinator(self) }

    func makeUIView(context: Context) -> UIView {
        let view = UIView()
        view.isAccessibilityElement = false
        let coordinator = context.coordinator
        let editMenu = UIEditMenuInteraction(delegate: coordinator)
        coordinator.editMenu = editMenu
        view.addInteraction(editMenu)
        let tap = UITapGestureRecognizer(target: coordinator, action: #selector(Coordinator.tapped))
        let pan = UIPanGestureRecognizer(target: coordinator, action: #selector(Coordinator.panned))
        let hold = UILongPressGestureRecognizer(target: coordinator, action: #selector(Coordinator.held))
        pan.maximumNumberOfTouches = 1
        pan.require(toFail: hold)
        let swipe = AgentListSwipe()
        swipe.openAgents = { [weak coordinator] in coordinator?.callbacks.openAgents() }
        swipe.require(toFail: hold)
        view.addGestureRecognizer(swipe)
        tap.require(toFail: hold)
        view.addGestureRecognizer(tap)
        view.addGestureRecognizer(pan)
        view.addGestureRecognizer(hold)
        return view
    }

    func updateUIView(_ uiView: UIView, context: Context) { context.coordinator.callbacks = self }

    @MainActor
    final class Coordinator: NSObject, @MainActor UIEditMenuInteractionDelegate {
        var callbacks: TerminalGestures
        weak var editMenu: UIEditMenuInteraction?
        private var menu: UIMenu?
        private let highlight = CALayer()
        private var trackpad: TerminalTrackpad?
        private var holdCallbacks: TerminalGestures?
        private var scrollFromStart: ((CGPoint, CGSize) -> Void)?

        init(_ callbacks: TerminalGestures) { self.callbacks = callbacks }

        func editMenuInteraction(_ interaction: UIEditMenuInteraction, menuFor configuration: UIEditMenuConfiguration,
                                 suggestedActions: [UIMenuElement]) -> UIMenu? { menu }

        func editMenuInteraction(_ interaction: UIEditMenuInteraction, willDismissMenuFor configuration: UIEditMenuConfiguration,
                                 animator: UIEditMenuInteractionAnimating) {
            menu = nil
            highlight.removeFromSuperlayer()
        }

        @objc func tapped() { callbacks.tap() }

        @objc func panned(_ recognizer: UIPanGestureRecognizer) {
            if recognizer.state == .began { scrollFromStart = callbacks.scroll }
            if recognizer.state == .cancelled || recognizer.state == .failed { scrollFromStart = nil }
            guard recognizer.state == .ended else { return }
            defer { scrollFromStart = nil }
            let delta = recognizer.translation(in: recognizer.view)
            let point = recognizer.location(in: recognizer.view)
            let start = CGPoint(x: point.x - delta.x, y: point.y - delta.y)
            scrollFromStart?(start, CGSize(width: delta.x, height: delta.y))
        }

        @objc func held(_ recognizer: UILongPressGestureRecognizer) {
            let point = recognizer.location(in: recognizer.view)
            switch recognizer.state {
            case .began:
                holdCallbacks = callbacks
                trackpad = TerminalTrackpad(origin: point)
                UISelectionFeedbackGenerator().selectionChanged()
            case .changed, .ended:
                guard let holdCallbacks else { return }
                trackpad?.move(to: point).forEach(holdCallbacks.arrow)
                if recognizer.state == .ended {
                    if trackpad?.moved == false {
                        let choices = holdCallbacks.actions(point)
                        menu = choices.menu
                        if let selection = choices.selection {
                            highlight.frame = selection
                            highlight.backgroundColor = UIColor.systemPink.withAlphaComponent(0.32).cgColor
                            recognizer.view?.layer.addSublayer(highlight)
                        }
                        editMenu?.presentEditMenu(with: UIEditMenuConfiguration(identifier: nil, sourcePoint: point))
                    }
                    trackpad = nil
                    self.holdCallbacks = nil
                }
            default:
                trackpad = nil
                holdCallbacks = nil
            }
        }
    }
}
