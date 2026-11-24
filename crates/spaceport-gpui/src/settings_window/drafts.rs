//! The one global the unsaved Settings drafts live in.
//!
//! `spaceport-settings` decides what a draft does to a load; this window keeps
//! the drafts every window reads — the Settings pages, the theme picker and
//! the main windows a preview reaches — so a choice made in one is fenced in
//! the others' reloads.

use crate::config::{Config, Theme};
use gpui::{App, Global};
use spaceport_settings::Drafts;

#[derive(Default)]
struct UnsavedDrafts(Drafts);
impl Global for UnsavedDrafts {}

/// The drafts, or none before the first edit of this process.
pub(super) fn drafts(cx: &App) -> Option<&Drafts> {
    cx.try_global::<UnsavedDrafts>().map(|global| &global.0)
}

/// The drafts, created on the first edit.
pub(super) fn drafts_mut(cx: &mut App) -> &mut Drafts {
    &mut cx.default_global::<UnsavedDrafts>().0
}

pub(crate) fn theme_pending(cx: &App) -> bool {
    drafts(cx).is_some_and(Drafts::theme_pending)
}

pub(crate) fn theme_load_revision(cx: &App) -> u64 {
    drafts(cx).map_or(0, Drafts::theme_load_revision)
}

pub(crate) fn apply_loaded_theme(config: &mut Config, theme: &mut Theme, revision: u64, cx: &App) {
    if let Some(drafts) = drafts(cx) {
        drafts.apply_loaded_theme(config, theme, revision);
    }
}

pub(crate) fn layout_load_revision(cx: &App) -> u64 {
    drafts(cx).map_or(0, Drafts::layout_load_revision)
}

pub(crate) fn apply_loaded_layout(config: &mut Config, revision: u64, cx: &App) {
    if let Some(drafts) = drafts(cx) {
        drafts.apply_loaded_layout(config, revision);
    }
}

pub(crate) fn sidebar_load_revision(cx: &App) -> u64 {
    drafts(cx).map_or(0, Drafts::sidebar_load_revision)
}

pub(crate) fn apply_loaded_sidebar(config: &mut Config, revision: u64, cx: &App) {
    if let Some(drafts) = drafts(cx) {
        drafts.apply_loaded_sidebar(config, revision);
    }
}
