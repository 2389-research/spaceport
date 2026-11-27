//! The artwork the app draws: each agent's mark, each status set, the UI
//! glyphs, and the provider logos the usage views show.
//!
//! Every file is embedded at build time from `assets/icons` at the repository
//! root, so nothing is read from disk at run time and no asset has to be
//! installed beside the binary. `Icons` is the `gpui::AssetSource` the app
//! registers once, and it is also the only list of what can be drawn, so
//! loading and listing cannot drift.
//!
//! The crate holds artwork and the shapes that draw it (`icon`, and the tinted
//! `note`/`muted` marks with `spoken_muted`'s labelled wrapper). It holds no app
//! state, names no message, and leaves to the app which mark a row shows and
//! when.

mod brand;
mod icons;
mod providers;

pub use icons::{
    AgentIcon, Icons, MUTED, STARRED, UNSTARRED, icon, muted, note, spoken_muted, status_icon,
};
pub use providers::{provider_logo, provider_paths};
