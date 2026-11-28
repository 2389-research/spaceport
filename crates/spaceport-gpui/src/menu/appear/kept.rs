//! What a closing popup draws from while it goes. Closing resets the menu's
//! state in the same frame; a page that animates out has the fields it reads
//! taken out first, and drawn again from here until it is gone. Taking them
//! out ends their effect before the leaving animation finishes.
//!
//! Every field is a value the page reads rather than a field it edits: the
//! draft a form asked for is kept as the text it held, so a page on its way out
//! shows what was typed without the window's text field going with it.

use super::super::MenuState;
use crate::dialog_input::DialogInput;

/// The fields a leaving page reads that closing clears.
#[derive(Default)]
pub(in crate::menu) struct Kept {
    /// What the closing page's draft held, as the form drew it.
    draft: Option<String>,
    error: Option<String>,
    /// The device All Devices showed unfolded, so it leaves as it was.
    unfolded_device: Option<String>,
}

impl Kept {
    /// Takes what the open page reads out of `menu`, before its reset.
    pub(in crate::menu) fn take(menu: &mut MenuState) -> Self {
        Self {
            draft: menu.input.as_ref().map(|input| input.text.clone()),
            error: menu.error.take(),
            unfolded_device: menu.unfolded_device.take(),
        }
    }

    /// Trades places with `menu`'s fields: once to draw the leaving page,
    /// and again to put the menu's own back. The draft goes back as the text
    /// it held, which is all a page on its way out draws of it.
    pub(super) fn swap(&mut self, menu: &mut MenuState) {
        std::mem::swap(&mut self.error, &mut menu.error);
        std::mem::swap(&mut self.unfolded_device, &mut menu.unfolded_device);
        let field = menu.input.take();
        let draft = std::mem::replace(
            &mut self.draft,
            field.map(|mut input| std::mem::take(&mut input.text)),
        );
        menu.input = draft.map(DialogInput::new);
    }
}
