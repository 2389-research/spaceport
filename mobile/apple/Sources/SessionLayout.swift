// ABOUTME: Fits the agent sidebar and keyboard controls to the usable session width.
// ABOUTME: Reserves enough space for terminal input as the window changes size.
import Foundation

enum SessionLayout {
    static func sidebarWidth(in width: CGFloat, regular: Bool) -> CGFloat? {
        guard regular, width >= 620 else { return nil }
        return min(280, max(248, width * 0.30))
    }

    static func usesCompactKeys(width: CGFloat) -> Bool { width < 360 }
}
