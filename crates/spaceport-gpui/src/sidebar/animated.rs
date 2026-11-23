//! The status sets drawn rather than loaded from assets — Superset's wave
//! grid, the braille cells, and Orca's marks — under the path the app has
//! always drawn them by. They live in `spaceport-sidebar` with the rest of a
//! row's model; `status_mark` in `sidebar::agents` picks between them and the
//! sets the icons crate serves.

pub(super) use spaceport_sidebar::animated::mark;
