// ABOUTME: Paints the Herdr cell grid and cursor at the current device scale.
// ABOUTME: Exposes the visible terminal text as one accessible reading surface.
import SwiftUI

struct TerminalDisplay: View {
    let terminal: MobileTerminal
    let metrics: TerminalMetrics
    static let background = Color(red: 28 / 255, green: 28 / 255, blue: 30 / 255)
    var body: some View {
        Canvas { context, size in
            let columns = Int(terminal.width)
            let rows = Int(terminal.height)
            guard columns > 0, rows > 0 else { return }
            let fitted = metrics.fittedCell(in: size, columns: columns, rows: rows)
            let width = fitted.width
            let height = fitted.height
            let font = metrics.pointSize * min(width / metrics.cell.width, height / metrics.cell.height)
            for (index, cell) in terminal.cells.prefix(columns * rows).enumerated() {
                let x = index % columns
                let y = index / columns
                let rect = CGRect(x: CGFloat(x) * width, y: CGFloat(y) * height, width: width, height: height)
                let reversed = cell.modifier & 64 != 0
                let foreground = Self.color(reversed ? cell.bg : cell.fg, foreground: !reversed)
                let background = Self.color(reversed ? cell.fg : cell.bg, foreground: reversed)
                if cell.bg != 0 || reversed {
                    context.fill(Path(rect), with: .color(background))
                }
                if !cell.skip && cell.modifier & 128 == 0 && cell.symbol != " " && !cell.symbol.isEmpty {
                    let text = Text(verbatim: cell.symbol)
                        .font(.system(size: font, weight: cell.modifier & 1 != 0 ? .bold : .regular, design: .monospaced))
                        .italic(cell.modifier & 4 != 0)
                        .foregroundStyle(foreground.opacity(cell.modifier & 2 != 0 ? 0.6 : 1))
                    context.draw(text, at: rect.origin, anchor: .topLeading)
                }
                if cell.modifier & 8 != 0 || cell.modifier & 256 != 0 {
                    if cell.modifier & 8 != 0 {
                        context.fill(Path(CGRect(x: rect.minX, y: rect.maxY - 2, width: width, height: 1)), with: .color(foreground))
                    }
                    if cell.modifier & 256 != 0 {
                        context.fill(Path(CGRect(x: rect.minX, y: rect.midY, width: width, height: 1)), with: .color(foreground))
                    }
                }
            }
            if let x = terminal.cursorX, let y = terminal.cursorY, Int(x) < columns, Int(y) < rows {
                let rect = CGRect(x: CGFloat(x) * width, y: CGFloat(y) * height, width: width, height: height)
                context.stroke(Path(rect.insetBy(dx: 0.5, dy: 0.5)), with: .color(.white.opacity(0.8)), lineWidth: 1)
            }
        }
        .background(Self.background)
        .accessibilityElement(children: .ignore)
        .accessibilityLabel("Terminal display")
        .accessibilityValue(visibleText)
        .accessibilityIdentifier("Terminal display")
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
        while index > start && terminal.cells[index].skip { index -= 1 }
        func belongs(_ position: Int) -> Bool {
            let glyph = terminal.cells[position]
            return glyph.modifier & 128 == 0 && (glyph.skip || glyph.symbol.contains { !$0.isWhitespace })
        }
        guard belongs(index) else { return nil }
        var left = index, right = index + 1
        while left > start && belongs(left - 1) { left -= 1 }
        while right < end && belongs(right) { right += 1 }
        let text = terminal.cells[left..<right].filter { !$0.skip }.map(\.symbol).joined()
        return (text, CGRect(x: CGFloat(left - start) * cell.width, y: CGFloat(row) * cell.height,
                             width: CGFloat(right - left) * cell.width, height: cell.height))
    }

    // Herdr's wire palette numbers named colors from one and tags indexed/RGB colors.
    static func color(_ value: UInt32, foreground: Bool) -> Color {
        let rgb: UInt32
        let palette: [UInt32] = [0x121212, 0xcc5555, 0x64b66a, 0xc7a74e,
                                 0x598ad1, 0xb877c5, 0x55b8b6, 0xc8c8c8,
                                 0x777777, 0xf98080, 0x90df96, 0xf2d878,
                                 0x8db6ed, 0xd89ee8, 0x84dedb, 0xffffff]
        if value >> 24 == 2 { rgb = value & 0xffffff }
        else if value >> 24 == 1 {
            let index = Int(value & 255)
            if index < 16 { rgb = palette[index] }
            else if index < 232 {
                let i = index - 16
                let levels: [UInt32] = [0, 95, 135, 175, 215, 255]
                rgb = levels[i / 36] << 16 | levels[i / 6 % 6] << 8 | levels[i % 6]
            } else { rgb = UInt32(8 + 10 * (index - 232)) * 0x010101 }
        } else if (1...16).contains(value) { rgb = palette[Int(value) - 1] }
        else { rgb = foreground ? 0xe5e5e7 : 0x1c1c1e }
        return Color(red: Double((rgb >> 16) & 255) / 255, green: Double((rgb >> 8) & 255) / 255,
                     blue: Double(rgb & 255) / 255)
    }
}

extension MobileTerminal {
    var visibleText: String {
        let width = Int(self.width)
        guard width > 0 else { return "" }
        var rows = [String]()
        for start in stride(from: 0, to: cells.count, by: width) {
            let line = cells[start..<min(cells.count, start + width)]
                .filter { !$0.skip }.map { $0.modifier & 128 == 0 ? $0.symbol : " " }.joined()
            let end = line.lastIndex { !$0.isWhitespace }.map { line.index(after: $0) } ?? line.startIndex
            rows.append(String(line[..<end]))
        }
        return rows.joined(separator: "\n").trimmingCharacters(in: .newlines)
    }
}
