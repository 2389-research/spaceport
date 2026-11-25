//! What a keystroke sends a pane: the key event Herdr names it as, and which
//! keystrokes have one. A `[pane_keys]` target without one would send nothing.

use gpui::Keystroke;
use spaceport_client::protocol::ClientKeyCode;

/// Whether a pane can receive `keystroke` as a key event, so `[pane_keys]`
/// never names one that would silently send nothing.
pub fn reaches_pane(keystroke: &Keystroke) -> bool {
    key_code(keystroke, true).is_some()
}

/// The key `keystroke` is sent as, or `None` when it reaches the pane as
/// typed text instead. `alt_keys` claims Alt-modified characters as shortcuts,
/// so a terminal can tell them from the characters a layout commits.
pub fn key_code(key: &Keystroke, alt_keys: bool) -> Option<ClientKeyCode> {
    use ClientKeyCode::*;
    if key.modifiers.platform {
        return None;
    }
    Some(match key.key.as_str() {
        "enter" => Enter,
        "backspace" | "back" => Backspace,
        "escape" => Esc,
        "tab" if key.modifiers.shift => BackTab,
        "tab" => Tab,
        "up" => Up,
        "down" => Down,
        "left" => Left,
        "right" => Right,
        "home" => Home,
        "end" => End,
        "pageup" => PageUp,
        "pagedown" => PageDown,
        "delete" => Delete,
        "insert" => Insert,
        name if name.starts_with('f')
            && name[1..].parse::<u8>().is_ok_and(|n| (1..=24).contains(&n)) =>
        {
            F(name[1..].parse().ok()?)
        }
        "space" if key.modifiers.control => Char(' '),
        name if key.modifiers.control && name.chars().count() == 1 => Char(name.chars().next()?),
        "space" if key.modifiers.alt && alt_keys => Char(' '),
        // GPUI reports Shift-letter as the lowercase key plus Shift; a shifted
        // symbol arrives as the symbol itself. Herdr expects the typed letter.
        name if key.modifiers.alt && alt_keys && name.chars().count() == 1 => {
            let ch = name.chars().next()?;
            Char(if key.modifiers.shift {
                ch.to_ascii_uppercase()
            } else {
                ch
            })
        }
        _ => return None,
    })
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests;
