//! The Keyboard Shortcuts page: the sections a catalog's commands list under,
//! every way to reach one, and the button that edits it.
//!
//! The window lists the rows — a description, the keystrokes the keymap gave
//! it, and why a daemon command's binding cannot run — and this narrows them
//! to a query and paints them. A row carries no message: the app words every
//! label, every heading and the note a binding's reach earns.

pub mod render;

/// One way to reach a row's command.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Binding {
    /// The keystrokes as the keymap spells them, a prefix chord's two in turn.
    pub keys: String,
    /// Why this binding does not run, beneath its keycaps: nothing for one
    /// that does.
    pub note: Option<String>,
}

/// One row: what a person calls a command, every way to reach it, and the
/// button that edits its shortcut.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Row {
    /// What the row says, and the name its selectors are built from.
    pub description: String,
    pub bindings: Vec<Binding>,
    /// The catalog name of the command whose shortcut the row's Edit button
    /// edits, which also names that button and its debug selector. Nothing
    /// for a row no keymap binding reaches — the paste row, and a daemon's own
    /// commands.
    pub edit: Option<&'static str>,
}

impl Row {
    /// Whether `query` finds this row by its description, by one of its keys,
    /// or by the heading it lists under.
    pub fn matches(&self, query: &str, section: &str) -> bool {
        matches(query, "", &self.description, section)
            || self
                .bindings
                .iter()
                .any(|binding| matches(query, &binding.keys, &self.description, section))
    }
}

/// The rows listing under one heading.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Section {
    pub heading: String,
    pub rows: Vec<Row>,
}

/// How many rows `sections` list.
pub fn total(sections: &[Section]) -> usize {
    sections.iter().map(|section| section.rows.len()).sum()
}

/// Keeps the rows `query` matches under their headings, drops every heading
/// whose rows are all gone, and answers how many rows are left.
pub fn filter(sections: &mut Vec<Section>, query: &str) -> usize {
    for Section { heading, rows } in sections.iter_mut() {
        rows.retain(|row| row.matches(query, heading));
    }
    sections.retain(|section| !section.rows.is_empty());
    total(sections)
}

/// Whether `query` finds a row whose keys are `keys`, which it says is called
/// `description` and lists under `section`.
pub fn matches(query: &str, keys: &str, description: &str, section: &str) -> bool {
    let query = query.to_lowercase().replace(['-', '+'], " ");
    if query
        .split_whitespace()
        .next()
        .is_some_and(|token| matches!(token, "cmd" | "ctrl" | "alt" | "shift"))
    {
        // A key combination should match keycaps, not letters in an action's name.
        let keys = keys.to_lowercase();
        return query
            .split_whitespace()
            .all(|token| keys.split(['-', '+', ' ']).any(|key| key == token));
    }
    let text = format!("{keys} {description} {section}")
        .to_lowercase()
        .replace(['-', '+'], " ");
    query.split_whitespace().all(|token| text.contains(token))
}
