// ABOUTME: Turns travel during a terminal or space-bar hold into semantic arrow keys.
// ABOUTME: Limits the number of keys produced by a single touch sample.
import Foundation

struct TerminalTrackpad {
    private var anchor: CGPoint
    private(set) var moved = false

    init(origin: CGPoint) { anchor = origin }

    mutating func move(to point: CGPoint) -> [String] {
        let dx = (point.x - anchor.x) / 10
        let dy = (point.y - anchor.y) / 18
        let horizontal = abs(dx) >= abs(dy)
        let distance = horizontal ? dx : dy
        guard abs(distance) >= 1 else { return [] }

        moved = true
        let steps = distance.rounded(.towardZero)
        if horizontal {
            anchor = CGPoint(x: anchor.x + steps * 10, y: point.y)
        } else {
            anchor = CGPoint(x: point.x, y: anchor.y + steps * 18)
        }
        let key = horizontal ? (steps > 0 ? "Right" : "Left") : (steps > 0 ? "Down" : "Up")
        return Array(repeating: key, count: Int(min(8, abs(steps))))
    }
}
