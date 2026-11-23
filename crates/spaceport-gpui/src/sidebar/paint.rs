//! The three things a row painter in `spaceport-sidebar` needs and cannot own:
//! the element its text is, the window's layer for a mark that moves, and the
//! copy for a word. Each is three lines here, because each already exists —
//! `label_text` (which the layout tests swap for `ProbeText`), the mark layer
//! the tab strip and activity bar share, and the Fluent catalog.

use super::label_text;
use crate::i18n::fl;
use gpui::{AnyElement, IntoElement, SharedString};
use spaceport_sidebar::paint::{Painters, Word};
use std::rc::Rc;

/// The crate's seams, as this window fills them. A `const` of `fn` items, so a
/// row's context can take it with no construction and no allocation.
pub(super) const PAINTERS: Painters = Painters {
    text: label_element,
    mark_layer: window_mark,
    words: word,
};

/// A label as the app paints it, through the one seam that swaps in the probes
/// the layout tests read: every painted word goes through `label_text`.
fn label_element(text: SharedString) -> AnyElement {
    label_text(text.as_ref()).into_any_element()
}

/// A moving mark drawn in the window's mark layer, so its frames never rebuild
/// the sidebar, faded to the opacity of the row around it.
fn window_mark(size: f32, fade: f32, build: Rc<dyn Fn() -> AnyElement>) -> AnyElement {
    crate::mark_layer::layered(size, fade, move || build()).into_any_element()
}

/// A word the crate names, in the reader's language.
fn word(word: Word) -> SharedString {
    match word {
        Word::Primary => fl!("sidebar-primary").into(),
    }
}
