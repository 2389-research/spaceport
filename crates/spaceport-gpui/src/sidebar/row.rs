//! A sidebar row, under the path the app has always named it by. What a row is,
//! the inputs every row of one render shares, its two lines, its configured
//! token rows and the layouts that arrange them all live in
//! `spaceport-sidebar`; the app paints one of them — the element a word is —
//! in `sidebar::paint`, beside the rest of the seams it fills.

pub(super) use spaceport_sidebar::row::{
    RowBadge, RowIcon, RowLift, RowTree, first_text, removing_dot,
};
#[cfg(any(test, feature = "integration-test"))]
pub(super) use spaceport_sidebar::row::{RowKind, row, row_text};
