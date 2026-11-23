//! The row layouts the config can pick. Each one is a unit struct assembled
//! from the shared pieces in `parts`, so adding a layout is a new file and a
//! `RowStyle` variant.

mod herdr;
mod minimal;
mod orca;
mod parts;
mod superset;

pub(super) use {herdr::Herdr, minimal::Minimal, orca::Orca, superset::Superset};
