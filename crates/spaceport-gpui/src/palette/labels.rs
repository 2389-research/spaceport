//! The words the palette's filters say.
//!
//! The filters themselves are `spaceport_palette`'s; a crate below the app
//! says no sentence, so the app matches each variant to its catalog message
//! (`docs/i18n.md`).

use super::{AgentFilter, Filter};
use crate::i18n::fl;

pub(super) fn filter(filter: Filter) -> String {
    match filter {
        Filter::All => fl!("palette-filter-all"),
        Filter::Navigation => fl!("palette-filter-navigation"),
        Filter::Commands => fl!("palette-filter-commands"),
        Filter::Projects => fl!("palette-filter-projects"),
        Filter::Notes => fl!("palette-filter-notes"),
    }
}

pub(super) fn agent_filter(filter: AgentFilter) -> String {
    match filter {
        AgentFilter::All => fl!("palette-go-to-filter-all"),
        AgentFilter::Blocked => fl!("palette-go-to-filter-blocked"),
        AgentFilter::Working => fl!("palette-go-to-filter-working"),
        AgentFilter::Idle => fl!("palette-go-to-filter-idle"),
        AgentFilter::Done => fl!("palette-go-to-filter-done"),
    }
}
