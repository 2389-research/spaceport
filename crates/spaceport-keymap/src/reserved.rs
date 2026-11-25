//! The keystrokes the app itself holds: the Edit menu's four, macOS's Hide,
//! Hide Others, and Minimize, and which of those system keys no command holds.
//! A daemon custom command never takes one (`crate::custom`).

use crate::keymap::Keymap;
use crate::parse_keystroke;
use gpui::Keystroke;
use spaceport_commands::CTRL_SHORTCUTS;

/// The Edit menu's keystrokes: cut, copy, paste, and select all. Windows
/// labels paste with plain Ctrl-V, which a pane takes as the app's paste
/// while the clipboard holds text, because Ctrl-Shift-V is often taken
/// system-wide there.
pub const EDIT_KEYS: [&str; 4] = if cfg!(target_os = "windows") {
    ["ctrl-shift-x", "ctrl-shift-c", "ctrl-v", "ctrl-shift-a"]
} else if CTRL_SHORTCUTS {
    [
        "ctrl-shift-x",
        "ctrl-shift-c",
        "ctrl-shift-v",
        "ctrl-shift-a",
    ]
} else {
    ["cmd-x", "cmd-c", "cmd-v", "cmd-a"]
};

/// macOS's Hide, Hide Others, and Minimize, bound there alone.
pub const SYSTEM_KEYS: [&str; 3] = ["cmd-h", "cmd-alt-h", "cmd-m"];

/// The system keystrokes the app keeps: each one `keymap` binds no command
/// to. No default is bound to any of them, so a command that holds one was
/// put there on purpose, in `[keybindings]` or the daemon's `[keys]`, and
/// runs instead.
#[cfg(any(target_os = "macos", feature = "test-support"))]
pub fn free_system_keys(keymap: &Keymap) -> impl Iterator<Item = &'static str> {
    let bound: Vec<Keystroke> = keymap
        .bindings()
        .filter_map(|(_, label)| parse_keystroke(label).ok())
        .collect();
    SYSTEM_KEYS.into_iter().filter(move |system| {
        parse_keystroke(system).is_ok_and(|system| {
            bound
                .iter()
                .all(|bound| (bound.modifiers, &bound.key) != (system.modifiers, &system.key))
        })
    })
}

/// Keystrokes a daemon custom command never takes: the Edit keys and the
/// macOS system keys. Its direct keystroke is matched before any binding, so
/// one of these would otherwise take Paste or Hide away from the app. A
/// system key the keymap gave to a command is held there instead, so a custom
/// command gets it neither way.
pub fn reserved_keystrokes() -> impl Iterator<Item = &'static str> {
    let system: &[&str] = if cfg!(target_os = "macos") {
        &SYSTEM_KEYS
    } else {
        &[]
    };
    EDIT_KEYS.into_iter().chain(system.iter().copied())
}
