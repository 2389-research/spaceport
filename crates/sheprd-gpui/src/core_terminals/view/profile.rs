// ABOUTME: Reads projected Core state for the explicitly enabled private profile observer.
// ABOUTME: Keeps projected content separate from capture-based visible-content qualification.
use crate::{SheprdWindow, browser::TabId, working_profile::report::Core};
use gpui::App;

impl SheprdWindow {
    pub(crate) fn profile_core_tab(&self, _tab: TabId, _cx: &App) -> Option<Core> {
        None
    }
}

#[cfg(test)]
mod tests;
