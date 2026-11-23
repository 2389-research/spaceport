//! The row layouts the config can pick. Each one is a unit struct assembled
//! from the shared pieces in `parts`, so adding a layout is a new file and a
//! `RowStyle` variant. `layout_for` names the one a window's mode draws.

mod herdr;
mod minimal;
mod orca;
mod parts;
mod superset;

pub use {herdr::Herdr, minimal::Minimal, orca::Orca, superset::Superset};

use spaceport_settings::LayoutMode;

use crate::cell::RowLayout;

/// The rows a layout draws: Herdr's for every density, or a design of its own.
pub fn layout_for(mode: LayoutMode) -> &'static dyn RowLayout {
    match mode {
        LayoutMode::Classic { .. } => &Herdr,
        LayoutMode::Superset => &Superset,
        LayoutMode::Orca => &Orca,
        LayoutMode::Minimal => &Minimal,
        // Herdr's own rows, nested under each device's header.
        LayoutMode::Devices => &Herdr,
    }
}
