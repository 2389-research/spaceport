//! The theme browser: the two libraries a Theme page lists, the grid of cards
//! that draws them, and the naming that keeps a theme following the system
//! appearance.
//!
//! The window keeps what only it can do — the search field it owns, the theme
//! it previews in its other windows, the file it saves a choice to and the
//! palettes it reads off a worker — and hands this crate the values and the
//! answers. The words are the crate's, as everywhere else in the kit.
// i18n: migrated

pub mod appearance;
pub mod grid;
pub mod system;

use spaceport_settings::herdr_settings;
use spaceport_settings::theme::{Scope, filter_names};

/// The height of one row of cards.
pub const ROW_HEIGHT: f32 = 80.;

/// One row of the browser: a theme name from one of the two libraries. The
/// same name in both is two rows, each drawn with its own library's palette.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Choice {
    pub scope: Scope,
    pub name: String,
}

/// Which libraries the browser lists. One is always listed, so the grid never
/// ends up with nothing to show.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Sources {
    pub ghostty: bool,
    pub herdr: bool,
}

impl Default for Sources {
    fn default() -> Self {
        Self::ALL
    }
}

impl Sources {
    /// Both libraries, as the browser starts.
    pub const ALL: Self = Self {
        ghostty: true,
        herdr: true,
    };

    /// Lists the libraries asked for, keeping the ones already listed when
    /// neither is asked for. Whether the sources are now the ones asked for.
    pub fn set(&mut self, ghostty: bool, herdr: bool) -> bool {
        if !ghostty && !herdr {
            return false;
        }
        *self = Self { ghostty, herdr };
        true
    }

    /// Whether this library's names are listed.
    pub fn lists(self, scope: Scope) -> bool {
        match scope {
            Scope::App => self.ghostty,
            Scope::Herdr => self.herdr,
        }
    }
}

/// The rows `names` and Herdr's own library resolve to under `query`: the
/// app's themes first, then Herdr's, each in case-insensitive name order.
pub fn listing(names: &[String], query: &str, sources: Sources) -> Vec<Choice> {
    let mut listing = Vec::new();
    for scope in [Scope::App, Scope::Herdr] {
        if !sources.lists(scope) {
            continue;
        }
        let matched = match scope {
            Scope::App => filter_names(names.iter().map(String::as_str), query),
            Scope::Herdr => filter_names(herdr_settings::THEME_NAMES.iter().copied(), query),
        };
        listing.extend(matched.into_iter().map(|name| Choice { scope, name }));
    }
    listing.sort_by_cached_key(|Choice { scope, name }| {
        (name.to_lowercase(), *scope == Scope::Herdr, name.clone())
    });
    listing
}

/// Where in `listing` the row `preserve` names stands, else the first row;
/// none when nothing is listed. A search keeps the choice it had when that
/// theme still matches, so the grid does not jump to another one.
pub fn chosen(listing: &[Choice], preserve: Option<&Choice>) -> Option<usize> {
    preserve
        .and_then(|choice| listing.iter().position(|row| row == choice))
        .or_else(|| (!listing.is_empty()).then_some(0))
}

/// `names` with `extra` added, each once, in case-insensitive name order.
pub fn merged(names: &mut Vec<String>, extra: impl IntoIterator<Item = String>) {
    for name in extra {
        if !names.contains(&name) {
            names.push(name);
        }
    }
    names.sort_by_cached_key(|name| (name.to_lowercase(), name.clone()));
}

#[cfg(test)]
mod tests;
