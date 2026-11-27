// ABOUTME: Follows the system appearance for themes written as light/dark pairs.
// ABOUTME: Keeps the pair's naming in the crate and the previews on the window.
//! Themes that follow the system appearance, written as Ghostty's
//! `light:NAME,dark:NAME`: which side the grid edits, turning following on and
//! off, and showing the side the current appearance calls for.
use super::{Choice, Scope, SettingsWindow};
use crate::settings_window::PageState;
use crate::{
    config::{FOLLOW_HERDR, Theme, ThemeName},
    contrast::Contrast,
    i18n::{Localize, fl},
};
use gpui::{Context, ScrollStrategy};
use spaceport_settings_window::themes::system;

impl SettingsWindow {
    /// The theme value the next choice builds on: an unsaved draft, else the
    /// saved one, so editing both sides quickly keeps the first edit.
    pub(in crate::settings_window) fn drafted_theme(&self) -> &str {
        self.theme_intent
            .as_ref()
            .filter(|intent| intent.choice.scope == Scope::App)
            .map_or(&self.config.theme, |intent| &intent.choice.name)
    }

    /// The app theme the grid marks as chosen: the edited side of a pair.
    pub(in crate::settings_window) fn edited_theme(&self) -> &str {
        ThemeName::side(&self.config.theme, self.themes.editing_light)
    }

    /// Chosen names the catalog lists even when no theme directory has them.
    pub(super) fn selected_theme_names(&self) -> Vec<String> {
        system::selected_names(&self.config.theme)
    }

    /// The unadjusted colors for one side of `value`, when they need no disk
    /// read: built-ins, Herdr's prepared palettes, and loaded themes.
    pub(in crate::settings_window) fn prepared_app_theme(
        &self,
        value: &str,
        light: bool,
    ) -> Option<Theme> {
        if ThemeName::follows_herdr(value, light) {
            return self.shared.as_ref()?.theme(light).ok();
        }
        let name = ThemeName::side(value, light);
        Theme::builtin(name).or_else(|| {
            self.theme_cache
                .iter()
                .find(|(cached, _)| cached == name)
                .map(|(_, theme)| theme.clone())
        })
    }

    pub(in crate::settings_window) fn toggle_system_theme(&mut self, cx: &mut Context<Self>) {
        if self.busy() {
            return;
        }
        let light = crate::app::light_appearance(cx);
        let name = system::toggled(self.drafted_theme(), self.theme.background, light);
        self.themes.editing_light = light;
        self.accept_theme_choice(
            Choice {
                scope: Scope::App,
                name,
            },
            cx,
        );
        self.select_edited_theme(cx);
    }

    /// System: starts a light/dark pair around the current theme.
    pub(in crate::settings_window) fn start_system_theme(&mut self, cx: &mut Context<Self>) {
        if !ThemeName::follows_system(self.drafted_theme()) {
            self.toggle_system_theme(cx);
        }
    }

    /// Light or Dark: while following, stops following and keeps that side as
    /// the single theme. A single theme of the other brightness gives way to
    /// the default theme of the chosen one; a matching one is kept.
    pub(in crate::settings_window) fn use_single_theme(
        &mut self,
        light: bool,
        cx: &mut Context<Self>,
    ) {
        if self.busy() {
            return;
        }
        let is_light = self.drafted_is_light(&PageState::of(self, cx), cx);
        let Some(name) = system::single(self.drafted_theme(), light, is_light) else {
            return;
        };
        self.themes.editing_light = light;
        self.accept_theme_choice(
            Choice {
                scope: Scope::App,
                name,
            },
            cx,
        );
        self.select_edited_theme(cx);
    }

    pub(in crate::settings_window) fn edit_theme_side(
        &mut self,
        light: bool,
        cx: &mut Context<Self>,
    ) {
        self.themes.editing_light = light;
        self.select_edited_theme(cx);
    }

    fn select_edited_theme(&mut self, cx: &mut Context<Self>) {
        let edited = Choice {
            scope: Scope::App,
            name: ThemeName::side(self.drafted_theme(), self.themes.editing_light).to_owned(),
        };
        if edited.name != FOLLOW_HERDR {
            super::browser::merged(&mut self.themes.names, [edited.name.clone()]);
        }
        self.themes.filter(Some(&edited));
        if let Some(index) = self.themes.selected {
            self.themes
                .scroll
                .scroll_to_item(index / self.themes.grid.columns, ScrollStrategy::Center);
        }
        self.request_theme_preview(cx);
    }

    /// Re-resolves a following theme, or a draft of one, when the system
    /// changes appearance. Theme files load on the background executor.
    pub(in crate::settings_window) fn follow_system_appearance(&mut self, cx: &mut Context<Self>) {
        let light = crate::app::light_appearance(cx);
        if let Some(intent) = &self.theme_intent {
            if intent.choice.scope != Scope::App
                || !ThemeName::follows_system(&intent.choice.name)
                || self.theme_light == light
            {
                return;
            }
            let theme = self.prepared_app_theme(&intent.choice.name, light);
            // A new revision fences a load already running for the old side.
            self.theme_revision = self.theme_revision.wrapping_add(1);
            self.theme_light = light;
            if let Some(intent) = &mut self.theme_intent {
                intent.revision = self.theme_revision;
                intent.theme = theme;
            }
            return;
        }
        if !ThemeName::follows_system(&self.config.theme) {
            return;
        }
        if let Some(theme) = self.prepared_app_theme(&self.config.theme, light) {
            self.theme = theme.styled_for(&self.config);
            return;
        }
        let mut config = self.config.clone();
        config.contrast = Contrast::Standard;
        let load = cx
            .background_executor()
            .spawn(async move { config.theme(light).map(|theme| (config.theme, theme)) });
        cx.spawn(async move |this, cx| {
            let result = load.await;
            let _ = this.update(cx, |this, cx| {
                let (value, theme) = match result {
                    Ok(loaded) => loaded,
                    Err(error) => {
                        this.error = Some(fl!(
                            "settings-error-system-appearance",
                            error = error.localize()
                        ));
                        cx.notify();
                        return;
                    }
                };
                let name = ThemeName::side(&value, light);
                if name != FOLLOW_HERDR {
                    if this.theme_cache.len() == 64 {
                        this.theme_cache.pop_front();
                    }
                    this.theme_cache.push_back((name.to_owned(), theme.clone()));
                }
                if value != this.config.theme
                    || light != crate::app::light_appearance(cx)
                    || this.theme_intent.is_some()
                {
                    return;
                }
                this.theme = theme.styled_for(&this.config);
                this.sync_appearance(cx);
                this.publish_appearance(cx);
            });
        })
        .detach();
    }
}
