//! The `[keybindings]` and `[pane_keys]` tables a settings file carries,
//! before the app resolves them into keystrokes.
use serde::Deserialize;
use std::collections::BTreeMap;

/// One entry of the `[keybindings]` table: a keystroke, or a list of them. An
/// empty string or list leaves the command unbound.
#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum Binding {
    One(String),
    Many(Vec<String>),
}

impl Binding {
    /// The keystrokes this entry names, trimmed, without the empty ones.
    pub fn keystrokes(&self) -> impl Iterator<Item = &str> {
        let keystrokes = match self {
            Self::One(keystroke) => std::slice::from_ref(keystroke),
            Self::Many(keystrokes) => keystrokes.as_slice(),
        };
        keystrokes
            .iter()
            .map(|keystroke| keystroke.trim())
            .filter(|keystroke| !keystroke.is_empty())
    }
}

/// The `[pane_keys]` table: a keystroke, and the keystroke or `text:` the
/// focused pane receives for it instead.
pub type PaneKeys = BTreeMap<String, String>;
