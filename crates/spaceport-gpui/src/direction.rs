//! The chrome's mirroring, from `spaceport-widgets`: the `Logical` trait every
//! styled element is given, which names a side by its start and end instead of
//! its left and right, and the direction those sides are read in. The rules
//! themselves are `spaceport-sidebar`'s, keyed there on a `bool` because a row's
//! painter cannot read a locale; kept at the path the app's painters have always
//! named it by.
//!
//! `i18n` publishes the shown locale's direction as it switches the catalog, so
//! a locale switch mirrors whatever is built after it. The terminal grid and its
//! content never mirror: panes, splits, and tab strips are the daemon's geometry.

pub(crate) use spaceport_widgets::direction::*;
