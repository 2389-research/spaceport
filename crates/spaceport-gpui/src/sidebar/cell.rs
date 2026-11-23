//! Sidebar rows as swappable layouts, under the path the app's render has
//! always named them by. What a row shows, the cell that carries its state, the
//! layouts' trait, the four layouts and the mode that picks one all live in
//! `spaceport-sidebar`.

pub(crate) use spaceport_sidebar::cell::{AgentRow, Cell, Fold, RowData, RowLayout, WorkspaceRow};
pub(crate) use spaceport_sidebar::layouts::layout_for;
pub(super) use spaceport_sidebar::row::{RowContext, RowState};
