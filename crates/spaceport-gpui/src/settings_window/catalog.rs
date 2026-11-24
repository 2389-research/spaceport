// ABOUTME: Declares every static Settings row once, with the state that makes it available.
// ABOUTME: Pages build their rows from these entries and search matches the same text.
//! Every static Settings row, at the paths the pages, the search and their
//! tests already use. `spaceport-settings` declares them; search matches
//! through this window's two wordings, as the palette matches commands.

pub(in crate::settings_window) use spaceport_settings::catalog::*;

use crate::i18n;

/// Indices into [`CATALOG`] of the entries whose title, description or page
/// name contains `query`, ignoring case, in catalog order: each entry matched
/// in this window's language and in English.
pub(in crate::settings_window) fn search(query: &str) -> Vec<usize> {
    spaceport_settings::catalog::search(
        query,
        |id| i18n::message(id, &[]),
        |id| i18n::english(id, &[]),
    )
}

/// The row's title in this window's language.
pub(in crate::settings_window) fn title(entry: &Entry) -> String {
    i18n::message(entry.title, &[])
}

/// The row's description in this window's language, empty when it has none.
pub(in crate::settings_window) fn description(entry: &Entry) -> String {
    if entry.description.is_empty() {
        return String::new();
    }
    i18n::message(entry.description, &[])
}
