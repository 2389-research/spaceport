//! What a page on its way out draws of the draft its form asked for: the text
//! it held, round the field closing cleared.

use super::*;
use crate::dialog_input::DialogInput;
use crate::menu::state::MenuState;

#[gpui::test]
fn a_closing_page_draws_the_draft_as_the_text_it_held(cx: &mut gpui::TestAppContext) {
    cx.update(|cx| {
        let mut menu = MenuState::new(cx);
        menu.input = Some(DialogInput::new("agent launcher".into()));
        let mut kept = Kept::take(&mut menu);
        menu.reset();
        // The frame the page leaves in reads the draft back...
        kept.swap(&mut menu);
        assert_eq!(draft(&menu), Some("agent launcher"));
        // ...and gives the menu's own cleared field back once it has read it.
        kept.swap(&mut menu);
        assert_eq!(draft(&menu), None);
        // The text outlives that frame, so the next one draws the same draft.
        kept.swap(&mut menu);
        assert_eq!(draft(&menu), Some("agent launcher"));
    });
}

/// The text a menu's draft holds, which is all a page on its way out draws.
fn draft(menu: &MenuState) -> Option<&str> {
    menu.input.as_ref().map(|input| input.text.as_str())
}
