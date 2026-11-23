//! A sidebar row's paint, under the path the app has always named it by. What
//! a row is, the inputs every row of one render shares, its two lines and its
//! configured token rows all live in `spaceport-sidebar`; what is left here is
//! the one element the app alone decides — the text a label is painted as, so
//! the layout probes can record every word a row draws.

#[cfg(any(test, feature = "integration-test"))]
use super::layout_tests;
use gpui::{IntoElement, SharedString};

/// A word as this app paints it: the string itself, or under the layout
/// tests, the probe element they read the laid-out text back from.
pub(crate) fn label_text(text: &str) -> impl IntoElement {
    label_element(SharedString::from(text))
}

#[cfg(not(any(test, feature = "integration-test")))]
fn label_element(text: SharedString) -> SharedString {
    text
}

#[cfg(any(test, feature = "integration-test"))]
fn label_element(text: SharedString) -> layout_tests::ProbeText {
    layout_tests::ProbeText(text)
}

pub(super) use spaceport_sidebar::row::{
    Elbow, RowBadge, RowIcon, RowKind, RowLift, RowTree, TokenLook, configured_status,
    configured_status_style, first_text, guide, leading_status, removing_dot, row, row_text,
    token_column, tree_gutter, trunk_below,
};
