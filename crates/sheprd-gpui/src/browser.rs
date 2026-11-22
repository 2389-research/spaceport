// ABOUTME: Connects native tab presentation to shared browser state.
// ABOUTME: Keeps saved paths and localized labels at the UI boundary.
//! Client-owned native tabs and shared terminal/chat split layouts.
//! The selected Herdr runtime owns terminal processes; views only arrange them.
//! A tab, its saved files and the group algebra are `sheprd-browser`'s; what
//! stays here is the window's own part: the strips it draws, the groups it
//! splits, and the words its language gives a tab.
// i18n: migrated

mod closed;
mod group_settle;
mod groups_view;
mod hosts;
mod strip;
mod tab_move;
#[cfg(test)]
mod tests;
mod view;
#[cfg(test)]
mod view_tests;

#[cfg(test)]
pub(crate) use sheprd_browser::GroupIds;
pub(crate) use sheprd_browser::{
    Browser, BrowserHosts, Cells, CloseRun, ClosedHerdrTab, ClosedTab, CoreGroupKey, Fold, GroupId,
    Layout, Layouts, Leaving, List, Listed, Location, Pick, Scope, Shown, Slot, Span, Store, Tab,
    TabId, Thumb, ThumbDrag, WebUrl,
};

/// The file the app keeps its browser tabs in, or none when it has no state
/// folder to save them in.
pub(crate) fn tabs_path() -> Option<std::path::PathBuf> {
    crate::preferences::state_dir().map(|dir| dir.join("browser-tabs.json"))
}

/// The file the app keeps each workspace's saved editor groups in.
pub(crate) fn layouts_path() -> Option<std::path::PathBuf> {
    crate::preferences::state_dir().map(|dir| dir.join("editor-groups.json"))
}
pub(crate) use view::{PageOpened, new_browser, scope, tab_open_failure};

/// The title the window shows for `tab`: what its content names itself, or
/// the words this window's language gives a tab that has not. A terminal tab
/// shows its program's title once the program sets one.
pub(crate) fn drawn_label(tab: &Tab) -> String {
    match &tab.location {
        None => crate::i18n::fl!("browser-tab-new"),
        Some(location @ Location::CoreTerminal { .. })
            if !tab.title.is_empty() && tab.title != location.default_title() =>
        {
            tab.title.clone()
        }
        Some(location) => drawn_title(location),
    }
}

/// The title of a tab the app draws, in the window's language.
fn drawn_title(location: &Location) -> String {
    match location {
        Location::Devices => crate::i18n::fl!("browser-tab-devices"),
        Location::CoreTerminal { .. } => crate::i18n::fl!("browser-tab-terminal"),
    }
}
