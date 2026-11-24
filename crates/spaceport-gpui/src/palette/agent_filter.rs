//! Go To's agent status filter: the key that chooses one, and the words its
//! chips say. The filter, its ranking and its headings are
//! `spaceport_palette`'s; only the keystroke (GPUI's) and the sentences are the
//! app's.

use super::Filter;
use crate::SpaceportWindow;
use gpui::{Context, Keystroke, Modifiers};

pub(super) use spaceport_palette::AgentFilter;

/// The filter Alt and exactly one navigator letter choose. Any other
/// modifier leaves the keystroke to the search field and the keymap.
pub(super) fn from_keystroke(keystroke: &Keystroke) -> Option<AgentFilter> {
    if keystroke.modifiers != Modifiers::alt() {
        return None;
    }
    AgentFilter::ALL
        .into_iter()
        .find(|filter| filter.key() == keystroke.key)
}

impl SpaceportWindow {
    /// Chooses a status and shows Go To, the only filter that applies one.
    pub(super) fn set_palette_agent_filter(
        &mut self,
        agent_filter: AgentFilter,
        cx: &mut Context<Self>,
    ) {
        if let Some(palette) = &mut self.menu.palette {
            palette.filter = Filter::Navigation;
            palette.agent_filter = agent_filter;
            palette.restore = None;
        }
        self.rank_palette(super::Selection::First, cx);
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests;
