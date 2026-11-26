// ABOUTME: Recognizes a one-finger right swipe across the terminal to reopen Agents.
// ABOUTME: Leaves vertical scrolling, selection, and indirect pointer input to the terminal.
import UIKit

final class AgentListSwipe: UIPanGestureRecognizer, UIGestureRecognizerDelegate {
    var openAgents: (() -> Void)?
    var canStart: () -> Bool = { true }

    init() {
        super.init(target: nil, action: nil)
        maximumNumberOfTouches = 1
        allowedTouchTypes = [NSNumber(value: UITouch.TouchType.direct.rawValue)]
        delegate = self
        addTarget(self, action: #selector(finished))
    }

    private nonisolated static func rightward(_ vector: CGPoint) -> Bool {
        vector.x > 1.5 * abs(vector.y)
    }

    nonisolated static func completes(_ translation: CGPoint) -> Bool {
        translation.x >= 56 && rightward(translation)
    }

    func gestureRecognizerShouldBegin(_ gestureRecognizer: UIGestureRecognizer) -> Bool {
        openAgents != nil && canStart() && Self.rightward(velocity(in: view))
    }

    func gestureRecognizer(_ gestureRecognizer: UIGestureRecognizer,
                           shouldBeRequiredToFailBy other: UIGestureRecognizer) -> Bool {
        // Both terminal views already handle one-finger pans on this same view.
        other.view === view && other is UIPanGestureRecognizer
    }

    @objc private func finished() {
        if state == .ended, Self.completes(translation(in: view)) { openAgents?() }
    }
}
