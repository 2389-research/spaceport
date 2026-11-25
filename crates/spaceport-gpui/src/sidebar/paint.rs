//! What a row painter in `spaceport-sidebar` needs and cannot own: the element
//! its text is, and the window's layer for a mark that moves. Each is three
//! lines here, because each already exists — `label_text` below (which the
//! layout tests swap for `ProbeText`), and the mark layer the tab strip and
//! activity bar share.
//!
//! This is the app's one cfg-aware text path: every word the sidebar paints,
//! whoever paints it, is an element built here.

#[cfg(any(test, feature = "integration-test"))]
use super::layout_tests;
use gpui::{AnyElement, IntoElement, SharedString};
use spaceport_sidebar::paint::Painters;
use std::rc::Rc;

/// The crate's seams, as this window fills them. A `const` of `fn` items, so a
/// row's context can take it with no construction and no allocation.
pub(crate) const PAINTERS: Painters = Painters {
    text: label_element,
    mark_layer: window_mark,
};

/// A word as this app paints it: the string itself, or under the layout tests,
/// the probe element they read the laid-out text back from.
pub(crate) fn label_text(text: &str) -> impl IntoElement {
    painted_text(SharedString::from(text))
}

/// A moving mark drawn in the window's mark layer, so its frames never rebuild
/// the sidebar, faded to the opacity of the row around it.
fn window_mark(size: f32, fade: f32, build: Rc<dyn Fn() -> AnyElement>) -> AnyElement {
    crate::mark_layer::layered(size, fade, move || build()).into_any_element()
}

fn label_element(text: SharedString) -> AnyElement {
    painted_text(text).into_any_element()
}

#[cfg(not(any(test, feature = "integration-test")))]
fn painted_text(text: SharedString) -> SharedString {
    text
}

#[cfg(any(test, feature = "integration-test"))]
fn painted_text(text: SharedString) -> layout_tests::ProbeText {
    layout_tests::ProbeText(text)
}
