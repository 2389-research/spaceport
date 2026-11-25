//! The two keymap tests the app keeps while the rest live in
//! `spaceport-keymap`: the keys it reserves for its own menus, and how a
//! `[pane_keys]` target reaches a pane.

use super::*;
use gpui::Keystroke;
use spaceport_keymap::PaneKey;
use std::collections::{BTreeMap, HashMap};

mod platform_defaults;
mod shift_enter;

fn overrides(entries: &[(&str, Binding)]) -> BTreeMap<String, Binding> {
    entries
        .iter()
        .map(|(name, binding)| ((*name).to_owned(), binding.clone()))
        .collect()
}

fn keystroke(text: &str) -> Keystroke {
    parse_keystroke(text).unwrap()
}

/// A daemon config that binds nothing, isolating the GUI layers.
fn no_keys() -> DaemonKeys {
    DaemonKeys {
        prefixes: vec![keystroke("ctrl-b")],
        bindings: Vec::new(),
        navigate_up: Vec::new(),
        navigate_down: Vec::new(),
    }
}

fn with(entries: &[(&str, Binding)]) -> spaceport_keymap::Result<Keymap> {
    Keymap::with_overrides(&overrides(entries), &PaneKeys::new(), &no_keys())
}

fn sent<'a>(keymap: &'a Keymap, typed: &str) -> Option<&'a PaneKey> {
    keymap.pane_key(&keystroke(typed))
}

fn key(text: &str) -> PaneKey {
    PaneKey::Key(keystroke(text))
}
