//! What a closing popup draws from while it goes. Closing resets the menu's
//! state in the same frame; a page that animates out has the fields it reads
//! taken out first, and drawn again from here until it is gone. Taking them
//! out ends their effect before the leaving animation finishes.

use super::super::{MenuState, Page, git};
use crate::dialog_input::DialogInput;

/// Whether `page` animates out after it closes: All Devices into the
/// footer and the Git popover.
pub(in crate::menu) fn leaves(page: Page) -> bool {
    matches!(page, Page::Devices | Page::Git)
}

/// The fields a leaving page reads that closing clears.
#[derive(Default)]
pub(in crate::menu) struct Kept {
    error: Option<String>,
    git_selected: Option<git::Row>,
    input: Option<DialogInput>,
    /// The device All Devices showed unfolded, so it leaves as it was.
    unfolded_device: Option<String>,
}

impl Kept {
    /// Takes what the open page reads out of `menu`, before its reset.
    pub(in crate::menu) fn take(menu: &mut MenuState) -> Self {
        let mut kept = Self::default();
        kept.swap(menu);
        kept
    }

    /// Trades places with `menu`'s fields: once to draw the leaving page,
    /// and again to put the menu's own back.
    pub(super) fn swap(&mut self, menu: &mut MenuState) {
        std::mem::swap(&mut self.error, &mut menu.error);
        std::mem::swap(&mut self.git_selected, &mut menu.git_selected);
        std::mem::swap(&mut self.input, &mut menu.input);
        std::mem::swap(&mut self.unfolded_device, &mut menu.unfolded_device);
    }
}
