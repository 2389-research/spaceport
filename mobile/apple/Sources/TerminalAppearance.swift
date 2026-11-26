// ABOUTME: Couples the app's appearance to the default background of its visible terminal.
// ABOUTME: Keeps the user's terminal palette separate from colors reported by a running program.
import GhosttyTerminal
import SwiftUI

enum TerminalAppearance: String {
    case dark, light

    static let preference = "terminalAppearance"
    var scheme: ColorScheme { self == .light ? .light : .dark }

    var theme: TerminalTheme {
        let palette: TerminalConfiguration = self == .light ? .alabaster : .afterglow
        // UIKit reports the app's appearance back to Ghostty. Use the selected palette in both
        // slots so a program changing its background cannot trigger a second palette change.
        return TerminalTheme(light: palette, dark: palette)
    }

    init(background: TerminalColor) {
        func linear(_ channel: UInt8) -> Double {
            let value = Double(channel) / 255
            return value <= 0.04045 ? value / 12.92 : pow((value + 0.055) / 1.055, 2.4)
        }
        let luminance = 0.2126 * linear(background.red) + 0.7152 * linear(background.green) + 0.0722 * linear(background.blue)
        // Above this point black text has more contrast than white text.
        self = luminance > 0.179 ? .light : .dark
    }
}

@MainActor
final class WindowTerminalAppearance: ObservableObject {
    @Published var reported: TerminalAppearance?
}
