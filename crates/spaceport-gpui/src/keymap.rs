//! The two keymap tests the app keeps while the rest live in
//! `spaceport-keymap`: the keys it reserves for its own menus (`crate::actions`)
//! and how a `[pane_keys]` target reaches a pane (`crate::terminal`). Every
//! other file names the crate's paths directly.

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests;
