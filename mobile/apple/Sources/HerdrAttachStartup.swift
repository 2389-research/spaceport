// ABOUTME: Handles Herdr’s outer terminal capability probe before the pane stream begins.
// ABOUTME: Leaves application output and typed input untouched once the known preamble ends.
import Foundation

struct HerdrAttachStartup {
    // Captured from Herdr 0.9.5 terminal attach. Match from byte zero so a pane can issue
    // identical queries or print the entire preamble later and still get normal Ghostty replies.
    private static let modes: Data = {
        let reset = "\u{1b}[?1006l\u{1b}[?1016l\u{1b}[?1015l\u{1b}[?1005l\u{1b}[?1003l\u{1b}[?1002l\u{1b}[?1000l\u{1b}[?9l"
        return Data((reset + "\u{1b}[?1049h" + reset + "\u{1b}[?2031l").utf8)
    }()
    private static let preamble = modes + Data("\u{1b}[?1016$p\u{1b}[c".utf8)
    private var matched: Int? = 0
    private(set) var hasPaneOutput = false

    mutating func receive(_ bytes: Data) -> Data {
        guard !bytes.isEmpty else { return bytes }
        guard let offset = matched else {
            hasPaneOutput = true
            return bytes
        }
        let count = min(bytes.count, Self.preamble.count - offset)
        let end = offset + count
        guard Self.preamble[offset..<end].elementsEqual(bytes.prefix(count)) else {
            hasPaneOutput = true
            return finish() + bytes
        }
        // Mode changes flow immediately. Withhold only the incomplete query pair.
        var output = Data(bytes.prefix(min(count, max(0, Self.modes.count - offset))))
        if end == Self.preamble.count {
            matched = nil
            hasPaneOutput = bytes.count > count
            output.append(bytes.dropFirst(count))
        } else {
            matched = end
        }
        return output
    }

    mutating func finish() -> Data {
        defer { matched = nil }
        guard let matched, matched > Self.modes.count else { return Data() }
        return Data(Self.preamble[Self.modes.count..<matched])
    }
}
