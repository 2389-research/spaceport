// ABOUTME: Reconstructs Herdr’s popup and reconnect cell snapshots for a read-only Ghostty grid.
// ABOUTME: Only client-generated positioning and styling can enter the terminal control stream.
import Foundation

extension MobileCell {
    var displayGlyph: String {
        guard !skip, modifier & 128 == 0, !symbol.isEmpty else { return " " }
        // One source cell owns one grapheme. Reject control bytes, direction overrides and malformed cells.
        guard symbol.count == 1, symbol.unicodeScalars.allSatisfy({ scalar in
            switch scalar.value {
            case 0...31, 127...159, 0x202A...0x202E, 0x2066...0x2069: false
            default: true
            }
        }) else { return "�" }
        return symbol
    }

    fileprivate var spansTwoColumns: Bool {
        let scalars = symbol.unicodeScalars
        guard let first = scalars.first, symbol.count == 1 else { return false }
        if first.properties.isEmoji {
            if scalars.contains("\u{fe0e}") { return false }
            if first.properties.isEmojiPresentation || scalars.contains("\u{fe0f}") { return true }
        }
        guard !first.isASCII else { return false }
        // Herdr treats voiced halfwidth kana as wide in addition to Unicode W/F graphemes.
        if (0xff66...0xff9d).contains(first.value), let last = scalars.last,
           last == "\u{ff9e}" || last == "\u{ff9f}" { return true }
        return symbol.range(of: #"^[\p{ea=W}\p{ea=F}]"#, options: .regularExpression) != nil
    }
}

extension MobileTerminal {
    func continuesGlyph(at index: Int) -> Bool {
        let cell = cells[index]
        guard width > 0, index % Int(width) > 0, cell.skip || cell.symbol.isEmpty else { return false }
        return cells[index - 1].spansTwoColumns
    }

    func text(in range: Range<Int>) -> String {
        range.lazy.filter { !cells[$0].skip && !continuesGlyph(at: $0) }.map { cells[$0].displayGlyph }.joined()
    }

    var visibleText: String {
        let width = Int(self.width)
        guard width > 0 else { return "" }
        var rows = [String]()
        let count = min(cells.count, width * Int(height))
        for start in stride(from: 0, to: count, by: width) {
            let line = text(in: start..<min(count, start + width))
            let end = line.lastIndex { !$0.isWhitespace }.map { line.index(after: $0) } ?? line.startIndex
            rows.append(String(line[..<end]))
        }
        return rows.joined(separator: "\n").trimmingCharacters(in: .newlines)
    }

    func replay(columns: Int, rows: Int) -> Data {
        // Disable auto-wrap so a glyph at the right edge never moves or scrolls the next row.
        var data = Data("\u{1b}[?7l\u{1b}[?25l\u{1b}[0m\u{1b}[2J".utf8)
        let width = Int(self.width)
        let visibleColumns = min(width, columns)
        let visibleRows = min(Int(height), rows)
        guard visibleColumns > 0, visibleRows > 0 else { return data }
        var previous: CellStyle?
        for row in 0..<visibleRows {
            for col in 0..<visibleColumns {
                let index = row * width + col
                guard index < cells.count else { break }
                let cell = cells[index]
                guard !continuesGlyph(at: index) else { continue }
                let glyph = cell.displayGlyph
                if glyph == " " && cell.bg == 0 && cell.modifier & (8 | 64 | 256) == 0 { continue }
                let style = CellStyle(foreground: cell.fg, background: cell.bg, flags: cell.modifier)
                if style != previous {
                    data.append(contentsOf: style.sgr.utf8)
                    previous = style
                }
                data.append(contentsOf: "\u{1b}[\(row + 1);\(col + 1)H".utf8)
                data.append(contentsOf: glyph.utf8)
            }
        }
        if let x = cursorX, let y = cursorY, Int(x) < visibleColumns, Int(y) < visibleRows {
            data.append(contentsOf: "\u{1b}[\(Int(y) + 1);\(Int(x) + 1)H\u{1b}[?25h".utf8)
        }
        return data
    }
}

private struct CellStyle: Equatable {
    let foreground: UInt32
    let background: UInt32
    let flags: UInt16

    var sgr: String {
        var codes = ["0"]
        for (mask, code): (UInt16, String) in [(1, "1"), (2, "2"), (4, "3"), (8, "4"), (64, "7"), (256, "9")] where flags & mask != 0 {
            codes.append(code)
        }
        for (color, prefix) in [(foreground, "38"), (background, "48")] {
            if color >> 24 == 2 {
                codes.append("\(prefix);2;\((color >> 16) & 255);\((color >> 8) & 255);\(color & 255)")
            } else if color >> 24 == 1 {
                codes.append("\(prefix);5;\(color & 255)")
            } else if (1...16).contains(color) {
                codes.append("\(prefix);5;\(color - 1)")
            }
        }
        return "\u{1b}[\(codes.joined(separator: ";"))m"
    }
}
