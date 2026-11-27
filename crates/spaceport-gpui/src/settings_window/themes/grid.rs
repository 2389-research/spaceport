// ABOUTME: Loads the palettes the Theme page's card grid draws.
// ABOUTME: Reports where a card painted for the native theme checks.
//! The card grid's palettes: which ones a render needs, the bounded background
//! load that prepares them, and the bounds a native test clicks by. The grid
//! itself is the crate's; this window reads the themes off a worker.
// i18n: migrated
use super::{Scope, SettingsWindow};
use crate::herdr_settings;
use gpui::{prelude::*, *};
use std::ops::Range;

/// How many palettes one background batch prepares.
const BATCH: usize = 12;

/// The crate's grid geometry, at the path this window names it by.
pub(super) use spaceport_settings_window::themes::grid::{columns, navigate};

impl SettingsWindow {
    /// True once no palette load is pending and every visible card has one.
    #[cfg(all(feature = "integration-test", target_os = "macos"))]
    pub(in crate::settings_window) fn grid_ready(&self) -> bool {
        let grid = &self.themes.grid;
        !grid.running && !grid.queued && grid.covers(&self.themes.filtered)
    }

    pub(super) fn schedule_grid_palettes(&mut self, rows: Range<usize>, cx: &mut Context<Self>) {
        let columns = self.themes.grid.columns;
        self.themes.grid.visible =
            rows.start * columns..(rows.end * columns).min(self.themes.filtered.len());
        if self.themes.grid.covers(&self.themes.filtered) {
            return;
        }
        let grid = &mut self.themes.grid;
        if grid.queued || grid.running {
            return;
        }
        grid.queued = true;
        // Leave render before launching work, without depending on display frames
        // that an occluded native window may never receive.
        let weak = cx.entity().downgrade();
        cx.defer(move |cx| {
            let _ = weak.update(cx, |this, cx| {
                this.themes.grid.queued = false;
                this.load_grid_palettes(cx);
            });
        });
    }

    fn load_grid_palettes(&mut self, cx: &mut Context<Self>) {
        if self.persistence.quitting() || self.themes.grid.running {
            return;
        }
        let grid = &mut self.themes.grid;
        let choices: Vec<_> = self
            .themes
            .filtered
            .get(grid.visible.clone())
            .unwrap_or_default()
            .iter()
            .filter(|choice| grid.palette(choice).is_none())
            .take(BATCH)
            .cloned()
            .collect();
        if choices.is_empty() {
            return;
        }
        grid.running = true;
        let epoch = grid.epoch;
        let mut config = self.config.clone();
        let shared = self.shared.clone();
        let light = matches!(
            cx.window_appearance(),
            WindowAppearance::Light | WindowAppearance::VibrantLight
        );
        let work = cx.background_executor().spawn(async move {
            let shared = match shared {
                None if choices.iter().any(|choice| choice.scope == Scope::Herdr) => {
                    herdr_settings::Settings::load().ok()
                }
                shared => shared,
            };
            choices
                .into_iter()
                .map(|choice| {
                    config.theme = choice.name.clone();
                    let result = match choice.scope {
                        Scope::Herdr => shared.as_ref().and_then(|shared| {
                            shared
                                .preview_theme(&choice.name, light)
                                .map(|theme| theme.styled_for(&config))
                                .ok()
                        }),
                        Scope::App => config.theme(light).ok(),
                    };
                    (choice, result)
                })
                .collect()
        });
        cx.spawn(async move |this, cx| {
            let palettes = work.await;
            let _ = this.update(cx, |this, cx| {
                this.themes.grid.finish(epoch, palettes);
                this.load_grid_palettes(cx);
                cx.notify();
            });
        })
        .detach();
    }
}

#[cfg(all(feature = "integration-test", target_os = "macos"))]
#[derive(Default)]
struct NativeBounds(std::collections::HashMap<usize, Bounds<Pixels>>);
#[cfg(all(feature = "integration-test", target_os = "macos"))]
impl Global for NativeBounds {}

#[cfg(all(feature = "integration-test", target_os = "macos"))]
pub(super) fn clear_native_bounds(cx: &mut App) {
    cx.default_global::<NativeBounds>().0.clear();
}

#[cfg(all(feature = "integration-test", target_os = "macos"))]
pub(super) fn native_probe(index: usize) -> impl IntoElement {
    canvas(
        |_, _, _| (),
        move |bounds, _, _, cx| {
            cx.default_global::<NativeBounds>().0.insert(index, bounds);
        },
    )
    .absolute()
    .size_full()
}

#[cfg(all(feature = "integration-test", target_os = "macos"))]
impl SettingsWindow {
    /// Latest painted card bounds; settled includes an unavailable-palette result.
    pub(in crate::settings_window) fn native_theme_cards(
        &self,
        cx: &App,
    ) -> Vec<(usize, Bounds<Pixels>, bool)> {
        cx.try_global::<NativeBounds>()
            .into_iter()
            .flat_map(|bounds| bounds.0.iter())
            .map(|(index, bounds)| {
                let settled = self
                    .themes
                    .filtered
                    .get(*index)
                    .is_some_and(|choice| self.themes.grid.palette(choice).is_some());
                (*index, *bounds, settled)
            })
            .collect()
    }
}

#[cfg(test)]
mod tests;
