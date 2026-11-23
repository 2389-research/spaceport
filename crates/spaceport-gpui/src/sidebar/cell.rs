//! Sidebar rows as swappable layouts, under the path the app's render has
//! always named them by. What a row shows, the cell that carries its state, and
//! the layouts' trait live in `spaceport-sidebar`; what is left here is which
//! layout a window's mode picks.

use crate::config::LayoutMode;

pub(crate) use spaceport_sidebar::cell::{AgentRow, Cell, Fold, RowData, RowLayout, WorkspaceRow};
pub(super) use spaceport_sidebar::row::{RowContext, RowState};

/// The rows a layout draws: Herdr's for every density, or a design of its own.
pub(super) fn layout_for(mode: LayoutMode) -> &'static dyn RowLayout {
    match mode {
        LayoutMode::Classic { .. } => &super::layouts::Herdr,
        LayoutMode::Superset => &super::layouts::Superset,
        LayoutMode::Orca => &super::layouts::Orca,
        LayoutMode::Minimal => &super::layouts::Minimal,
        // Herdr's own rows, nested under each device's header.
        LayoutMode::Devices => &super::layouts::Herdr,
    }
}
