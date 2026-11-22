// ABOUTME: Computes the cell size shared by terminal drawing, gestures and host resize.
// ABOUTME: Fits the current projection into the visible grid while a resize settles.
import Foundation

struct TerminalMetrics {
    let pointSize: CGFloat
    let cell: CGSize

    init(textSize: Int) {
        let range = TerminalPreferences.textRange
        let points = CGFloat(max(range.lowerBound, min(range.upperBound, textSize)))
        pointSize = points
        cell = CGSize(width: ceil(points * 8 / 13), height: ceil(points * 17 / 13))
    }

    func fittedCell(in size: CGSize, columns: Int, rows: Int) -> CGSize {
        CGSize(width: min(cell.width, size.width / CGFloat(max(1, columns))),
               height: min(cell.height, size.height / CGFloat(max(1, rows))))
    }

    func viewport(size: CGSize, scale: CGFloat) -> Viewport {
        Viewport(columns: UInt16(max(1, min(512, size.width / cell.width))),
                 rows: UInt16(max(1, min(512, size.height / cell.height))),
                 cellWidth: UInt32(max(1, (cell.width * scale).rounded())),
                 cellHeight: UInt32(max(1, (cell.height * scale).rounded())))
    }
}
