// ABOUTME: Shows Herdr popup and reconnect snapshots on an isolated Ghostty surface.
// ABOUTME: Keeps accessible text and word selection aligned to Herdr’s source cells.
import GhosttyTerminal
import SwiftUI

struct TerminalDisplay: View {
    let terminal: MobileTerminal
    let metrics: TerminalMetrics
    static let background = SessionChrome.terminal

    var body: some View {
        SnapshotDisplay(terminal: terminal, metrics: metrics)
            .allowsHitTesting(false)
            .background(Self.background)
            .accessibilityElement(children: .ignore)
            .accessibilityLabel("Terminal popup")
            .accessibilityValue(visibleText)
            .accessibilityIdentifier("terminal.popup")
    }

    var visibleText: String { terminal.visibleText }

    func word(at point: CGPoint, in size: CGSize) -> (text: String, rect: CGRect)? {
        let columns = Int(terminal.width)
        let rows = Int(terminal.height)
        guard columns > 0, rows > 0, point.x >= 0, point.y >= 0 else { return nil }
        let cell = metrics.fittedCell(in: size, columns: columns, rows: rows)
        guard cell.width > 0, cell.height > 0 else { return nil }
        let column = Int(point.x / cell.width), row = Int(point.y / cell.height)
        guard column < columns, row < rows else { return nil }
        let start = row * columns
        let end = min(start + columns, terminal.cells.count)
        var index = start + column
        guard index < end else { return nil }
        while index > start && terminal.continuesGlyph(at: index) { index -= 1 }
        func belongs(_ position: Int) -> Bool {
            let glyph = terminal.cells[position]
            return terminal.continuesGlyph(at: position)
                || (glyph.modifier & 128 == 0 && glyph.displayGlyph.contains { !$0.isWhitespace })
        }
        guard belongs(index) else { return nil }
        var left = index, right = index + 1
        while left > start && belongs(left - 1) { left -= 1 }
        while right < end && belongs(right) { right += 1 }
        let text = terminal.text(in: left..<right)
        return (text, CGRect(x: CGFloat(left - start) * cell.width, y: CGFloat(row) * cell.height,
                             width: CGFloat(right - left) * cell.width, height: cell.height))
    }
}

private struct SnapshotDisplay: UIViewRepresentable {
    let terminal: MobileTerminal
    let metrics: TerminalMetrics

    func makeUIView(context: Context) -> SnapshotViewport { SnapshotViewport() }

    func updateUIView(_ view: SnapshotViewport, context: Context) {
        view.update(terminal, metrics: metrics)
    }

    static func dismantleUIView(_ view: SnapshotViewport, coordinator: ()) {
        view.terminalView.setSurfaceVisible(false)
    }
}

@MainActor
private final class SnapshotViewport: UIView {
    let terminalView = UITerminalView(frame: .zero)
    private var frameData: MobileTerminal?
    private var metrics = TerminalMetrics(textSize: 12)
    private var grid: InMemoryTerminalViewport?
    private var fontSize: Float?
    private lazy var session = InMemoryTerminalSession(write: { _ in }, resize: { [weak self] value in
        Task { @MainActor in self?.didResize(value) }
    })

    init() {
        super.init(frame: .zero)
        clipsToBounds = true
        terminalView.isUserInteractionEnabled = false
        terminalView.accessibilityElementsHidden = true
        terminalView.layer.anchorPoint = .zero
        addSubview(terminalView)
    }

    required init?(coder: NSCoder) { nil }

    func update(_ frame: MobileTerminal, metrics: TerminalMetrics) {
        let changed = frameData != frame
        frameData = frame
        self.metrics = metrics
        setNeedsLayout()
        if changed { repaint() }
    }

    override func layoutSubviews() {
        super.layoutSubviews()
        guard let frameData, bounds.width > 0, bounds.height > 0 else { return }
        let cols = max(1, Int(frameData.width)), rows = max(1, Int(frameData.height))
        let fit = metrics.fittedCell(in: bounds.size, columns: cols, rows: rows)
        let font = Float(max(1, (metrics.pointSize * min(fit.width / metrics.cell.width, fit.height / metrics.cell.height)).rounded(.down)))
        if fontSize != font {
            fontSize = font
            grid = nil
            terminalView.configuration = TerminalSurfaceOptions(backend: .inMemory(session), fontSize: font)
            terminalView.controller = GhosttyPaneSurface.controller
        }
        guard let grid, grid.cellWidthPixels > 0, grid.cellHeightPixels > 0 else {
            terminalView.transform = .identity
            terminalView.frame = bounds
            return
        }
        let scale = window?.screen.nativeScale ?? traitCollection.displayScale
        guard scale > 0 else { return }
        let naturalWidth = CGFloat(grid.cellWidthPixels) / scale
        let naturalHeight = CGFloat(grid.cellHeightPixels) / scale
        // A subpixel margin avoids rounding away the last column or row. Bound the backing texture
        // to the device for oversized daemon frames and clip the replay to the actual Ghostty grid.
        terminalView.bounds = CGRect(x: 0, y: 0, width: min(bounds.width * 2, naturalWidth * CGFloat(cols) + 1 / scale),
                                     height: min(bounds.height * 2, naturalHeight * CGFloat(rows) + 1 / scale))
        terminalView.layer.position = .zero
        terminalView.transform = CGAffineTransform(scaleX: fit.width / naturalWidth, y: fit.height / naturalHeight)
    }

    private func didResize(_ value: InMemoryTerminalViewport) {
        guard value != grid else { return }
        let previous = grid
        grid = value
        setNeedsLayout()
        if previous?.columns != value.columns || previous?.rows != value.rows { repaint() }
    }

    private func repaint() {
        guard let grid, let frameData else { return }
        session.discardPendingOutput()
        session.receiveReplay(frameData.replay(columns: Int(grid.columns), rows: Int(grid.rows)))
    }
}
