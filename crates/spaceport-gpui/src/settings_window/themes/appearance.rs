// ABOUTME: Draws the Theme page's browser over the crate's card grid.
// ABOUTME: Answers the grid with this window's intents, previews and palettes.
//! The search box, result count and card grid on the Theme page: the crate
//! draws them, and this window answers them with the state it holds.
// i18n: migrated
use super::{ROW_HEIGHT, Scope, SettingsWindow, grid};
use crate::{config::ThemeName, direction::Logical, herdr_settings};
use gpui::{prelude::*, *};
use spaceport_settings_window::themes::appearance as paint;

impl SettingsWindow {
    pub(in crate::settings_window) fn render_theme_search(&self, cx: &mut Context<Self>) -> Div {
        paint::search(
            cx.listener(Self::theme_browser_key),
            self.themes.search.clone().into_any_element(),
        )
    }

    #[allow(clippy::too_many_lines, reason = "2026-11-27 baseline: 211 lines")]
    pub(in crate::settings_window) fn render_theme_results(
        &mut self,
        window: &Window,
        cx: &mut Context<Self>,
    ) -> Div {
        #[cfg(all(feature = "integration-test", target_os = "macos"))]
        grid::clear_native_bounds(cx);
        let columns = grid::columns(f32::from(window.viewport_size().width) - 240.);
        if self.themes.grid.columns != columns {
            self.themes.grid.columns = columns;
            self.themes.scroll.scroll_to_item(
                self.themes.selected.unwrap_or(0) / columns,
                ScrollStrategy::Center,
            );
        }
        let total = usize::from(self.themes.sources.ghostty) * self.themes.names.len()
            + usize::from(self.themes.sources.herdr) * herdr_settings::THEME_NAMES.len();
        let body = (!self.themes.filtered.is_empty()).then(|| {
            uniform_list(
                "settings-theme-results",
                self.themes.filtered.len().div_ceil(columns),
                cx.processor(|this, range: std::ops::Range<usize>, _, cx| {
                    this.schedule_grid_palettes(range.clone(), cx);
                    range
                        .map(|row| {
                            div().h(px(ROW_HEIGHT)).w_full().row().gap(px(6.)).children(
                                (0..this.themes.grid.columns).map(|column| {
                                    let index = row * this.themes.grid.columns + column;
                                    let Some(choice) = this.themes.filtered.get(index).cloned()
                                    else {
                                        return div().flex_1().min_w_0().into_any_element();
                                    };
                                    let palette = match this.themes.grid.palette(&choice) {
                                        Some(Some(theme)) => paint::Palette::Loaded(theme),
                                        Some(None) => paint::Palette::Unavailable,
                                        None => paint::Palette::Loading,
                                    };
                                    let draft = this.theme_intent.as_ref().is_some_and(|intent| {
                                        intent.choice.scope == choice.scope
                                            && match choice.scope {
                                                Scope::App => {
                                                    ThemeName::side(
                                                        &intent.choice.name,
                                                        this.themes.editing_light,
                                                    ) == choice.name
                                                }
                                                Scope::Herdr => intent.choice.name == choice.name,
                                            }
                                    });
                                    let saved = match choice.scope {
                                        Scope::App => choice.name == this.edited_theme(),
                                        Scope::Herdr => this
                                            .shared
                                            .as_ref()
                                            .is_some_and(|shared| shared.theme_name == choice.name),
                                    };
                                    #[cfg(all(feature = "integration-test", target_os = "macos"))]
                                    let probe = Some(grid::native_probe(index).into_any_element());
                                    #[cfg(not(all(
                                        feature = "integration-test",
                                        target_os = "macos"
                                    )))]
                                    let probe = None;
                                    paint::card(
                                        paint::Card {
                                            theme: &this.theme,
                                            choice: &choice,
                                            index,
                                            selected: this.themes.selected == Some(index),
                                            draft,
                                            saved,
                                            palette,
                                        },
                                        probe,
                                        cx.listener(move |this, hovered: &bool, _, cx| {
                                            // Leaving this row is the grid's; moving
                                            // between cards never flashes the saved theme.
                                            if *hovered {
                                                this.hover_theme(Some(index), cx);
                                            }
                                        }),
                                        cx.listener(move |this, _, window, cx| {
                                            let focus = this.themes.search.read(cx).focus.clone();
                                            window.focus(&focus, cx);
                                            this.select_settings_theme(index, cx);
                                        }),
                                    )
                                }),
                            )
                        })
                        .collect()
                }),
            )
            .track_scroll(&self.themes.scroll)
            .flex_1()
            .min_h_0()
            .w_full()
            .into_any_element()
        });
        paint::results(
            paint::Results {
                theme: &self.theme,
                shown: self.themes.filtered.len(),
                total,
                discovering: self.themes.discovering,
                body,
                catalog_error: self.themes.catalog_error.as_deref(),
                preview_error: self.themes.preview_error.as_deref(),
            },
            cx.listener(|this, _, _, cx| this.discover_settings_themes(cx)),
            cx.listener(|this, _, _, cx| this.request_theme_preview(cx)),
            cx.listener(|this, hovered: &bool, _, cx| {
                if !*hovered {
                    this.hover_theme(None, cx);
                }
            }),
        )
    }
}
