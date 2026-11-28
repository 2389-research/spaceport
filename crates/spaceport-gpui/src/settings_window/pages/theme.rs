// ABOUTME: Builds theme preferences from the shared Settings catalog.
// ABOUTME: Previews choices and routes appearance changes through existing save handlers.
//! Theme: appearance mode, the library to browse, the card grid whose hover
//! previews the theme in the real Herdr windows, then the window backdrop and
//! terminal padding.
// i18n: migrated
use super::{PageState, SettingsWindow, catalog, on_value, ui};
use crate::{
    config::{Config, FOLLOW_HERDR, ThemeName},
    contrast::{Contrast, luminance},
    i18n::fl,
    settings_window::themes::{Choice, Scope},
};
use gpui::{prelude::*, *};

#[derive(Clone, Copy, PartialEq)]
enum Mode {
    Light,
    Dark,
    System,
}

#[derive(Clone, Copy, PartialEq)]
enum Library {
    All,
    Ghostty,
    Herdr,
}

/// How the page shows a theme value: [`FOLLOW_HERDR`] is a config value, shown
/// as a message; every other name is data.
pub(in crate::settings_window) fn display_name(name: &str) -> String {
    if name == FOLLOW_HERDR {
        fl!("settings-theme-follow-herdr")
    } else {
        name.to_owned()
    }
}

/// The saved theme as a file on disk, when it is one.
#[cfg(any(target_os = "macos", target_os = "linux"))]
fn theme_file(name: &str) -> Option<std::path::PathBuf> {
    if let Some(relative) = name.strip_prefix("~/") {
        return Some(std::path::PathBuf::from(std::env::var_os("HOME")?).join(relative));
    }
    std::path::Path::new(name)
        .is_absolute()
        .then(|| std::path::PathBuf::from(name))
}

impl SettingsWindow {
    pub(in crate::settings_window) fn render_theme(
        &mut self,
        window: &Window,
        cx: &mut Context<Self>,
    ) -> Div {
        // The grid records the columns it laid out with, so it draws before
        // the page borrows the values it shows.
        let results = self.render_theme_results(window, cx);
        let page = PageState::of(self, cx);
        let mode_row = self.theme_mode_row(&page, cx);
        let side_row = self.theme_side_row(&page, cx);
        let follow_row = self.theme_follow_row(&page, cx);
        let contrast_row = self.theme_contrast_row(&page, cx);
        let source_row = self.theme_source_row(&page, cx);
        let search = self.render_theme_search(cx);
        let open_file = self.theme_open_file(&page, cx);
        let window_rows = self.render_window_rows(&page, cx);
        let theme = page.theme;
        div()
            .debug_selector(|| "settings-appearance".into())
            .w_full()
            .flex()
            .flex_col()
            .min_w_0()
            .child(ui::section(
                theme,
                &fl!("settings-theme-section-mode"),
                None,
            ))
            .child(mode_row)
            .children(side_row)
            .child(follow_row)
            .child(contrast_row)
            .child(ui::section(
                theme,
                &fl!("settings-theme-section-themes"),
                None,
            ))
            .child(source_row)
            .child(div().mt(px(8.)).child(search))
            .children(open_file)
            .child(
                div()
                    .debug_selector(|| "settings-theme-grid".into())
                    .mt(px(8.))
                    .flex()
                    .flex_col()
                    .flex_1()
                    .child(results),
            )
            .child(window_rows)
    }

    /// Whether the theme in effect is light, read from the drafted choice
    /// when its colors are at hand.
    pub(in crate::settings_window) fn drafted_is_light(
        &self,
        page: &PageState<'_>,
        cx: &App,
    ) -> bool {
        let light = crate::app::light_appearance(cx);
        let background = self
            .prepared_app_theme(page.drafted_theme, light)
            .map_or(page.theme.background, |theme| theme.background);
        luminance(background) > 0.4
    }

    fn theme_mode_row(&self, page: &PageState<'_>, cx: &mut Context<Self>) -> ui::Row {
        let following = ThemeName::follows_system(page.drafted_theme);
        let is_light = self.drafted_is_light(page, cx);
        let selected = if following {
            Mode::System
        } else if is_light {
            Mode::Light
        } else {
            Mode::Dark
        };
        ui::row(page.theme, &catalog::THEME_MODE).control(ui::segmented(
            page.theme,
            "theme-mode-control",
            &[
                (Mode::Light, fl!("settings-theme-light")),
                (Mode::Dark, fl!("settings-theme-dark")),
                (Mode::System, fl!("settings-theme-system")),
            ],
            Some(selected),
            true,
            on_value(cx, move |this, mode, window, cx| {
                let focus = this.themes.search.read(cx).focus.clone();
                window.focus(&focus, cx);
                match mode {
                    Mode::System => this.start_system_theme(cx),
                    Mode::Light => this.use_single_theme(true, cx),
                    Mode::Dark => this.use_single_theme(false, cx),
                }
            }),
        ))
    }

    /// Shown while following: which side of the pair the grid changes.
    fn theme_side_row(&self, page: &PageState<'_>, cx: &mut Context<Self>) -> Option<ui::Row> {
        let value = page.drafted_theme;
        page.available(catalog::THEME_SIDE.availability).then(|| {
            ui::row(page.theme, &catalog::THEME_SIDE)
                // The page also names the pair's two themes.
                .description(fl!(
                    "settings-theme-side-hint",
                    light = display_name(ThemeName::side(value, true)),
                    dark = display_name(ThemeName::side(value, false)),
                ))
                .control(ui::segmented(
                    page.theme,
                    "theme-side-control",
                    &[
                        (true, fl!("settings-theme-light")),
                        (false, fl!("settings-theme-dark")),
                    ],
                    Some(self.themes.editing_light),
                    true,
                    on_value(cx, |this, light, window, cx| {
                        let focus = this.themes.search.read(cx).focus.clone();
                        window.focus(&focus, cx);
                        this.edit_theme_side(light, cx);
                    }),
                ))
        })
    }

    fn theme_follow_row(&self, page: &PageState<'_>, cx: &mut Context<Self>) -> ui::Row {
        // The switch speaks for the whole setting: on only when Herdr names
        // the theme in both appearances, and turning it on replaces a pair.
        let following = [true, false]
            .into_iter()
            .all(|light| ThemeName::follows_herdr(&page.config.theme, light));
        ui::row(page.theme, &catalog::THEME_FOLLOW)
            .tag(ui::Source::Herdr)
            .switch(
                ui::toggle(page.theme, "theme-follow-toggle", following, true).on_click(
                    cx.listener(move |this, _, _, cx| {
                        let name = if following {
                            this.themes
                                .choice()
                                .filter(|choice| choice.scope == Scope::App)
                                .map_or_else(|| "Default".to_owned(), |choice| choice.name) // i18n: data
                        } else {
                            FOLLOW_HERDR.to_owned()
                        };
                        this.accept_theme_choice(
                            Choice {
                                scope: Scope::App,
                                name,
                            },
                            cx,
                        );
                    }),
                ),
            )
    }

    fn theme_contrast_row(&self, page: &PageState<'_>, cx: &mut Context<Self>) -> ui::Row {
        let high = page.config.contrast == Contrast::High;
        ui::row(page.theme, &catalog::THEME_CONTRAST).switch(
            ui::toggle(page.theme, "theme-contrast-toggle", high, true).on_click(cx.listener(
                move |this, _, _, cx| {
                    let contrast = if high {
                        Contrast::Standard
                    } else {
                        Contrast::High
                    };
                    this.save_native(
                        move || Config::save_contrast(contrast).map_err(crate::Error::from),
                        cx,
                    );
                },
            )),
        )
    }

    fn theme_source_row(&self, page: &PageState<'_>, cx: &mut Context<Self>) -> ui::Row {
        let selected = match (self.themes.sources.ghostty, self.themes.sources.herdr) {
            (true, true) => Library::All,
            (true, false) => Library::Ghostty,
            (false, _) => Library::Herdr,
        };
        ui::row(page.theme, &catalog::THEME_SOURCE).control(ui::segmented(
            page.theme,
            "theme-source-control",
            &[
                (Library::All, fl!("settings-theme-library-all")),
                (Library::Ghostty, fl!("settings-theme-library-ghostty")),
                (Library::Herdr, fl!("settings-theme-library-herdr")),
            ],
            Some(selected),
            true,
            on_value(cx, |this, library, window, cx| {
                let focus = this.themes.search.read(cx).focus.clone();
                window.focus(&focus, cx);
                let (ghostty, herdr) = match library {
                    Library::All => (true, true),
                    Library::Ghostty => (true, false),
                    Library::Herdr => (false, true),
                };
                this.set_theme_sources(ghostty, herdr, cx);
            }),
        ))
    }

    /// A button that opens the saved theme when it is a file.
    #[cfg(any(target_os = "macos", target_os = "linux"))]
    fn theme_open_file(&self, page: &PageState<'_>, cx: &mut Context<Self>) -> Option<Div> {
        let path = theme_file(self.edited_theme())?;
        Some(
            div().mt(px(8.)).flex().child(
                ui::button(
                    page.theme,
                    "theme-open-file",
                    fl!("settings-theme-open-file"),
                    true,
                )
                .on_click(
                    cx.listener(move |_, _, _, cx| super::open_path(false, path.clone(), cx)),
                ),
            ),
        )
    }

    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    fn theme_open_file(&self, _: &PageState<'_>, _: &mut Context<Self>) -> Option<Div> {
        None
    }
}
