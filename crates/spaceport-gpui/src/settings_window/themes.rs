// ABOUTME: Previews explicit theme choices across Settings and app windows.
// ABOUTME: Persists captured choices without clearing newer theme intents.
//! Search-only samples and explicit app-wide theme intents. Disk work stays off the UI thread.
// i18n: migrated
use super::SettingsWindow;
use crate::{
    config::{FOLLOW_HERDR, Theme, ThemeName},
    contrast::Contrast,
    herdr_settings::{self, Edit},
    i18n::{Localize, fl},
    search_input::{Changed, SearchInput},
};
use gpui::{prelude::*, *};
pub(super) use spaceport_settings::theme::Scope;
use spaceport_settings::theme::filter_names;
mod appearance;
mod grid;
mod system;

const ROW_HEIGHT: f32 = 80.;

#[derive(Clone)]
pub(super) struct ThemeIntent {
    pub(super) revision: u64,
    choice: Choice,
    theme: Option<Theme>,
    /// The shared snapshot a Herdr theme was resolved against. It is the save's
    /// expected state when the window holds none, so an external edit made
    /// since then is a conflict rather than silently replaced.
    shared: Option<herdr_settings::Settings>,
}

#[cfg(test)]
#[derive(Clone)]
pub(super) struct ThemeIo {
    pub write: std::sync::Arc<ThemeWriter>,
    pub load: std::sync::Arc<dyn Fn() -> crate::Result<super::Loaded> + Send + Sync>,
    pub resolve: Option<std::sync::Arc<ThemeResolver>>,
    pub shared: Option<std::sync::Arc<SharedLoader>>,
}

#[cfg(test)]
type SharedLoader = dyn Fn() -> crate::Result<herdr_settings::Settings> + Send + Sync;

#[cfg(test)]
type ThemeResolver = dyn Fn(&str) -> crate::Result<Theme> + Send + Sync;

#[cfg(test)]
type ThemeWriter =
    dyn Fn(String, Option<herdr_settings::Settings>) -> crate::Result<()> + Send + Sync;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct Choice {
    pub scope: Scope,
    pub name: String,
}

pub(super) struct ThemeBrowser {
    pub(super) search: Entity<SearchInput>,
    _subscription: Subscription,
    names: Vec<String>,
    pub(super) filtered: Vec<Choice>,
    pub(super) ghostty_enabled: bool,
    pub(super) herdr_enabled: bool,
    query: String,
    selected: Option<usize>,
    scroll: UniformListScrollHandle,
    initialized: bool,
    discovering: bool,
    catalog_error: Option<String>,
    revision: u64,
    running: Option<u64>,
    attempted: Option<u64>,
    loaded: Option<u64>,
    preview: Option<Theme>,
    preview_name: Option<String>,
    preview_error: Option<String>,
    /// The card under the pointer; main windows show it until it leaves.
    hovered: Option<Choice>,
    contrast: Contrast,
    grid: grid::Grid,
    /// Which side of a theme that follows the system the grid edits.
    pub(super) editing_light: bool,
}

impl ThemeBrowser {
    pub(super) fn new(cx: &mut Context<SettingsWindow>) -> Self {
        let search = cx.new(|cx| {
            let mut input = SearchInput::new(cx);
            input.set_placeholder_with(|| fl!("settings-theme-search-placeholder"), cx);
            input
        });
        let subscription = cx.subscribe(&search, |this, search, _: &Changed, cx| {
            let query = search.read(cx).text().to_owned();
            if query == this.themes.query {
                return;
            }
            this.themes.query = query;
            this.themes.filter(None);
            this.request_theme_preview(cx);
        });
        Self {
            search,
            _subscription: subscription,
            names: Vec::new(),
            filtered: Vec::new(),
            ghostty_enabled: true,
            herdr_enabled: true,
            query: String::new(),
            selected: None,
            scroll: UniformListScrollHandle::new(),
            initialized: false,
            discovering: false,
            catalog_error: None,
            revision: 0,
            running: None,
            attempted: None,
            loaded: None,
            preview: None,
            preview_name: None,
            preview_error: None,
            hovered: None,
            contrast: Contrast::Standard,
            grid: grid::Grid::default(),
            editing_light: false,
        }
    }

    pub(super) fn choice(&self) -> Option<Choice> {
        self.filtered.get(self.selected?).cloned()
    }

    fn source_enabled(&self, scope: Scope) -> bool {
        match scope {
            Scope::App => self.ghostty_enabled,
            Scope::Herdr => self.herdr_enabled,
        }
    }

    fn filter(&mut self, preserve: Option<&Choice>) {
        self.grid.invalidate();
        self.filtered.clear();
        for scope in [Scope::App, Scope::Herdr] {
            if !self.source_enabled(scope) {
                continue;
            }
            let names = match scope {
                Scope::App => filter_names(self.names.iter().map(String::as_str), &self.query),
                Scope::Herdr => {
                    filter_names(herdr_settings::THEME_NAMES.iter().copied(), &self.query)
                }
            };
            self.filtered
                .extend(names.into_iter().map(|name| Choice { scope, name }));
        }
        self.filtered.sort_by_cached_key(|choice| {
            (
                choice.name.to_lowercase(),
                choice.scope == Scope::Herdr,
                choice.name.clone(),
            )
        });
        self.selected = preserve
            .and_then(|choice| self.filtered.iter().position(|item| item == choice))
            .or_else(|| (!self.filtered.is_empty()).then_some(0));
        self.scroll.scroll_to_item(
            self.selected.unwrap_or(0) / self.grid.columns,
            ScrollStrategy::Top,
        );
    }

    fn finish_preview(&mut self, revision: u64, name: String, result: crate::Result<Theme>) {
        if self.running != Some(revision) {
            return;
        }
        self.running = None;
        if revision != self.revision {
            return;
        }
        match result {
            Ok(theme) => {
                self.preview = Some(theme);
                self.preview_name = Some(name);
                self.loaded = Some(revision);
                self.preview_error = None;
            }
            Err(error) => self.preview_error = Some(error.to_string()),
        }
    }
}

impl SettingsWindow {
    fn theme_operation(
        &self,
        intent: &ThemeIntent,
    ) -> impl FnOnce(
        Option<herdr_settings::Settings>,
    ) -> crate::Result<Option<super::persistence::SharedWrite>>
    + Send
    + 'static
    + use<> {
        let choice = intent.choice.clone();
        let config = self.config.clone();
        let shared = self.shared.clone().or_else(|| intent.shared.clone());
        #[cfg(test)]
        let io = self.theme_io.clone();
        move |refreshed| {
            let shared = match choice.scope {
                Scope::App => None,
                Scope::Herdr => refreshed.or(shared),
            };
            #[cfg(test)]
            if let Some(io) = io {
                return (io.write)(choice.name, shared).map(|_| None);
            }
            match choice.scope {
                Scope::App => config
                    .save_theme(&choice.name)
                    .map(|()| None)
                    .map_err(crate::Error::from),
                // Every resolved Herdr intent carries a snapshot; a load here
                // only reports the real failure if one never resolved.
                Scope::Herdr => super::persistence::SharedWrite::save(
                    shared.map_or_else(herdr_settings::Settings::load, Ok)?,
                    Edit::Theme(choice.name),
                )
                .map(Some)
                .map_err(crate::Error::from),
            }
        }
    }

    pub(super) fn take_shutdown_theme(
        &mut self,
    ) -> Option<
        impl FnOnce(Option<herdr_settings::Settings>) -> crate::Result<()> + Send + 'static + use<>,
    > {
        let intent = self.theme_intent.take()?;
        if matches!(self.persistence.kind(), Some(super::Kind::Theme(revision)) if *revision == intent.revision)
        {
            return None;
        }
        let validate = self.theme_loader(intent.choice.clone());
        let write = self.theme_operation(&intent);
        Some(move |shared: Option<herdr_settings::Settings>| {
            let resolved = match intent.theme {
                Some(_) => None,
                None => validate()?.1,
            };
            write(shared.or(resolved)).map(|_| ())
        })
    }

    fn theme_loader(
        &self,
        choice: Choice,
    ) -> impl FnOnce() -> crate::Result<(Theme, Option<herdr_settings::Settings>)> + Send + 'static + use<>
    {
        let mut config = self.config.clone();
        config.theme = choice.name;
        config.contrast = Contrast::Standard;
        let shared = self.shared.clone();
        let light = self.theme_light;
        #[cfg(test)]
        let resolve = self.theme_io.as_ref().and_then(|io| io.resolve.clone());
        #[cfg(test)]
        let recover = self.theme_io.as_ref().and_then(|io| io.shared.clone());
        move || {
            #[cfg(test)]
            if let Some(resolve) = resolve {
                return Ok((resolve(ThemeName::side(&config.theme, light))?, None));
            }
            match (choice.scope, shared) {
                (Scope::Herdr, shared) => {
                    // A shared config that failed to load is read again, so the
                    // error names the real cause and a fixed file is picked up.
                    let shared = match shared {
                        Some(shared) => shared,
                        #[cfg(test)]
                        None if let Some(recover) = recover => recover()?,
                        None => herdr_settings::Settings::load()?,
                    };
                    let theme = shared.preview_theme(&config.theme, light)?;
                    // Raw, as the other branches return it: the draft is
                    // styled once, in `apply_theme_draft`.
                    Ok((theme.with_contrast(config.contrast), Some(shared)))
                }
                (Scope::App, Some(shared)) if ThemeName::follows_herdr(&config.theme, light) => {
                    Ok((shared.theme(light)?, None))
                }
                _ => Ok((config.theme(light)?, None)),
            }
        }
    }

    pub(super) fn accept_theme_choice(&mut self, choice: Choice, cx: &mut Context<Self>) {
        if self.persistence.quitting() || self.persistence.closing() {
            return;
        }
        if !choice.scope.editable() {
            self.status = Some(fl!("settings-theme-read-only"));
            cx.notify();
            return;
        }
        self.theme_load_failed = false;
        self.error = None;
        self.theme_revision = self.theme_revision.wrapping_add(1);
        self.theme_light = matches!(
            cx.window_appearance(),
            WindowAppearance::Light | WindowAppearance::VibrantLight
        );
        let theme = match (choice.scope, self.shared.as_ref()) {
            (Scope::Herdr, Some(shared)) => {
                shared.preview_theme(&choice.name, self.theme_light).ok()
            }
            (Scope::App, _) => self.prepared_app_theme(&choice.name, self.theme_light),
            _ => None,
        };
        let shared = match choice.scope {
            Scope::Herdr => self.shared.clone(),
            Scope::App => None,
        };
        self.theme_intent = Some(ThemeIntent {
            revision: self.theme_revision,
            choice,
            theme,
            shared,
        });
        super::drafts::drafts_mut(cx).begin_theme();
        self.drive_theme_intent(cx);
    }

    #[allow(clippy::too_many_lines, reason = "2026-11-27 baseline: 138 lines")]
    pub(super) fn drive_theme_intent(&mut self, cx: &mut Context<Self>) {
        if self.persistence.quitting() {
            return;
        }
        if self.theme_load_failed {
            self.persistence.stop_flush();
            return;
        }
        let light = crate::app::light_appearance(cx);
        if let Some(intent) = &mut self.theme_intent
            && intent.choice.scope == Scope::App
            && ThemeName::follows_herdr(&intent.choice.name, light)
            && let Some(shared) = &self.shared
            && let Ok(theme) = shared.theme(light)
        {
            intent.theme = Some(theme);
        }
        let Some(intent) = self.theme_intent.clone() else {
            self.finish_close(cx);
            return;
        };
        if intent.theme.is_none() {
            if self.theme_loading {
                return;
            }
            self.theme_loading = true;
            let load = self.theme_loader(intent.choice.clone());
            let light = self.theme_light;
            let retained = cx.entity();
            let work = cx.background_executor().spawn(async move { load() });
            cx.spawn(async move |_, cx| {
                let result = work.await;
                retained.update(cx, |this, cx| {
                    this.theme_loading = false;
                    if this.persistence.quitting() {
                        return;
                    }
                    if let Some(current) = &mut this.theme_intent
                        && current.revision == intent.revision
                    {
                        match result {
                            Ok((theme, shared)) => {
                                if shared.is_some() {
                                    current.shared = shared;
                                }
                                let name = ThemeName::side(&intent.choice.name, light);
                                if intent.choice.scope == Scope::App && name != FOLLOW_HERDR {
                                    if this.theme_cache.len() == 64 {
                                        this.theme_cache.pop_front();
                                    }
                                    this.theme_cache.push_back((name.to_owned(), theme.clone()));
                                }
                                current.theme = Some(theme);
                            }
                            Err(error) => {
                                this.theme_load_failed = true;
                                this.persistence.abort_drain();
                                this.error = Some(fl!(
                                    "settings-theme-error-load",
                                    error = error.localize()
                                ));
                            }
                        }
                    }
                    this.drive_theme_intent(cx);
                    cx.notify();
                });
            })
            .detach();
            return;
        }
        // A main picker at its commit boundary must reconcile before we replace it.
        if cx.windows().into_iter().any(|window| {
            window
                .downcast::<crate::SpaceportWindow>()
                .is_some_and(|window| {
                    window
                        .read(cx)
                        .is_ok_and(|view| view.theme_save_in_flight())
                })
        }) {
            if self.theme_waiting {
                return;
            }
            self.theme_waiting = true;
            let retained = cx.entity();
            cx.spawn(async move |_, cx| {
                cx.background_executor()
                    .timer(std::time::Duration::from_millis(10))
                    .await;
                retained.update(cx, |this, cx| {
                    this.theme_waiting = false;
                    this.drive_theme_intent(cx);
                });
            })
            .detach();
            return;
        }
        if let Some(theme) = intent.theme {
            if intent.choice.scope == Scope::App {
                self.config.theme = intent.choice.name.clone();
                super::drafts::drafts_mut(cx)
                    .preview_theme(self.config.theme.clone(), theme.clone());
                self.theme = theme.styled_for(&self.config);
                self.broadcast_theme(cx);
            } else if ThemeName::follows_herdr(&self.config.theme, light) {
                super::drafts::drafts_mut(cx)
                    .preview_theme(self.config.theme.clone(), theme.clone());
                self.theme = theme.styled_for(&self.config);
                self.broadcast_theme(cx);
            }
        }
        if (!self.persistence.closing() && !self.persistence.flushing()) || self.busy() {
            return;
        }
        let Some(intent) = self.theme_intent.as_ref() else {
            return;
        };
        let operation = self.theme_operation(intent);
        let operation = move || operation(None);
        let shared = intent.choice.scope == Scope::Herdr;
        let revision = intent.revision;
        #[cfg(test)]
        if let Some(io) = self.theme_io.clone() {
            self.save_kind_with_snapshot(
                operation,
                move || (io.load)(),
                shared,
                super::Kind::Theme(revision),
                cx,
            );
            return;
        }
        self.save_kind_with_snapshot(
            operation,
            self.loader(cx),
            shared,
            super::Kind::Theme(revision),
            cx,
        );
    }

    pub(super) fn broadcast_theme(&mut self, cx: &mut Context<Self>) {
        for handle in cx.windows() {
            if let Some(handle) = handle.downcast::<crate::SpaceportWindow>() {
                let _ = handle.update(cx, |view, window, cx| {
                    if view.theme_save_in_flight() {
                        return;
                    }
                    if view.menu.page == Some(crate::menu::Page::Themes) {
                        view.dismiss_menu(window, cx);
                    }
                    view.config.theme = self.config.theme.clone();
                    view.config.contrast = self.config.contrast;
                    view.theme = self.theme.clone();
                    if !super::drafts::theme_pending(cx) {
                        view.settings.shared = self.shared.clone();
                    }
                    cx.notify();
                });
            }
        }
        let mut appearance = cx
            .try_global::<crate::app::InitialAppearance>()
            .cloned()
            .unwrap_or_default();
        appearance.config.theme = self.config.theme.clone();
        appearance.config.contrast = self.config.contrast;
        appearance.theme = self.theme.clone();
        crate::log_window::set_appearance(&appearance.config, &appearance.theme, cx);
        cx.set_global(appearance);
        self.themes.search.update(cx, |input, cx| {
            input.set_appearance(self.config.ui.clone(), self.theme.clone(), cx)
        });
        self.refresh_control_appearance(cx);
        cx.notify();
    }

    #[cfg(all(feature = "integration-test", target_os = "macos"))]
    pub(super) fn native_theme_search(&self, cx: &App) -> (FocusHandle, usize, bool) {
        (
            self.themes.search.read(cx).focus.clone(),
            self.themes.filtered.len(),
            self.themes.discovering
                || self
                    .native_theme_cards(cx)
                    .iter()
                    .any(|(_, _, settled)| !settled)
                || self.themes.running.is_some()
                || self.theme_loading
                || self.theme_waiting
                || self.persistence.saving(),
        )
    }

    pub(super) fn initialize_theme_browser(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.themes.initialized {
            return;
        }
        self.themes.initialized = true;
        self.themes.contrast = self.config.contrast;
        self.themes.preview = Some(self.theme.clone());
        self.themes.editing_light = crate::app::light_appearance(cx);
        self.themes.preview_name = Some(self.edited_theme().to_owned());
        self.themes.names = Theme::BUILTIN_NAMES
            .iter()
            .map(|name| (*name).into())
            .collect();
        self.themes.names.extend(self.selected_theme_names());
        self.themes.names.sort();
        self.themes.names.dedup();
        self.themes.filter(Some(&Choice {
            scope: Scope::App,
            name: self.edited_theme().to_owned(),
        }));
        self.themes.search.update(cx, |search, cx| {
            search.set_appearance(self.config.ui.clone(), self.theme.clone(), cx);
            window.focus(&search.focus, cx);
        });
        self.discover_settings_themes(cx);
        self.request_theme_preview(cx);
    }

    pub(super) fn sync_theme_browser(&mut self, cx: &mut Context<Self>) {
        self.themes.contrast = self.config.contrast;
        let missing: Vec<_> = self
            .selected_theme_names()
            .into_iter()
            .filter(|name| !self.themes.names.contains(name))
            .collect();
        if !missing.is_empty() {
            self.themes.names.extend(missing);
            self.themes
                .names
                .sort_by_cached_key(|name| (name.to_lowercase(), name.clone()));
        }
        let selected = self.themes.choice();
        self.themes.filter(selected.as_ref());
        self.themes.search.update(cx, |search, cx| {
            search.set_appearance(self.config.ui.clone(), self.theme.clone(), cx);
        });
        self.request_theme_preview(cx);
    }

    fn discover_settings_themes(&mut self, cx: &mut Context<Self>) {
        if self.themes.discovering {
            return;
        }
        self.themes.discovering = true;
        self.themes.catalog_error = None;
        let config = self.config.clone();
        let task = cx
            .background_executor()
            .spawn(async move { config.available_themes() });
        cx.spawn(async move |this, cx| {
            let result = task.await;
            let _ = this.update(cx, |this, cx| {
                this.themes.discovering = false;
                match result {
                    Ok(mut names) => {
                        names.extend(this.selected_theme_names());
                        names.retain(|name| name != FOLLOW_HERDR);
                        names.sort_by_cached_key(|name| (name.to_lowercase(), name.clone()));
                        names.dedup();
                        let selected = this.themes.choice();
                        this.themes.names = names;
                        this.themes.filter(selected.as_ref());
                        this.request_theme_preview(cx);
                    }
                    Err(error) => this.themes.catalog_error = Some(error.to_string()),
                }
                cx.notify();
            });
        })
        .detach();
        cx.notify();
    }

    fn request_theme_preview(&mut self, cx: &mut Context<Self>) {
        self.themes.revision += 1;
        self.themes.loaded = None;
        self.themes.preview_error = None;
        self.drive_theme_preview(cx);
        cx.notify();
    }

    /// Chooses which theme libraries the grid lists. Neither is never valid.
    pub(super) fn set_theme_sources(&mut self, ghostty: bool, herdr: bool, cx: &mut Context<Self>) {
        if !ghostty && !herdr {
            return;
        }
        let selected = self.themes.choice();
        self.themes.ghostty_enabled = ghostty;
        self.themes.herdr_enabled = herdr;
        self.themes.filter(selected.as_ref());
        self.request_theme_preview(cx);
    }

    /// Resolves a card's theme on the background executor, with the
    /// browser's contrast applied.
    fn resolve_choice(&self, choice: &Choice, cx: &App) -> Task<crate::Result<Theme>> {
        let contrast = self.themes.contrast;
        let light = crate::app::light_appearance(cx);
        let mut config = self.config.clone();
        config.theme = choice.name.clone();
        config.contrast = contrast;
        let scope = choice.scope;
        let shared = self.shared.clone();
        cx.background_executor().spawn(async move {
            // Config::theme already applies contrast; do not apply it twice.
            match scope {
                Scope::Herdr => shared
                    .map_or_else(herdr_settings::Settings::load, Ok)?
                    .preview_theme(&config.theme, light)
                    .map(|theme| theme.styled_for(&config)),
                Scope::App => config.theme(light),
            }
            .map_err(crate::Error::from)
        })
    }

    pub(super) fn theme_hovered(&self) -> bool {
        self.themes.hovered.is_some()
    }

    /// Shows the card under the pointer in every Herdr window without
    /// drafting it; `None` puts the real appearance back at once.
    pub(super) fn hover_theme(&mut self, index: Option<usize>, cx: &mut Context<Self>) {
        let choice = index
            .filter(|_| !self.persistence.quitting() && !self.persistence.closing())
            .and_then(|index| self.themes.filtered.get(index).cloned());
        if self.themes.hovered == choice {
            return;
        }
        self.themes.hovered = choice.clone();
        let Some(choice) = choice else {
            self.broadcast_preview_theme(None, cx);
            return;
        };
        let task = self.resolve_choice(&choice, cx);
        cx.spawn(async move |this, cx| {
            let Ok(theme) = task.await else {
                return;
            };
            let _ = this.update(cx, |this, cx| {
                if this.themes.hovered.as_ref() == Some(&choice) {
                    this.broadcast_preview_theme(Some(&theme), cx);
                }
            });
        })
        .detach();
    }

    /// Sets the theme every Herdr window draws. Touches neither the draft
    /// nor the appearance new windows start with.
    pub(super) fn broadcast_preview_theme(&self, theme: Option<&Theme>, cx: &mut Context<Self>) {
        let theme = theme.cloned().unwrap_or_else(|| {
            let mut config = self.config.clone();
            let mut theme = self.theme.clone();
            if let Some(drafts) = super::drafts::drafts(cx) {
                drafts.apply_theme_draft(&mut config, &mut theme);
            }
            theme
        });
        for handle in cx.windows() {
            if let Some(handle) = handle.downcast::<crate::SpaceportWindow>() {
                let _ = handle.update(cx, |view, _, cx| {
                    if view.theme_save_in_flight() {
                        return;
                    }
                    view.theme = theme.clone();
                    cx.notify();
                });
            }
        }
    }

    fn drive_theme_preview(&mut self, cx: &mut Context<Self>) {
        let browser = &mut self.themes;
        if browser.running.is_some() || browser.attempted == Some(browser.revision) {
            return;
        }
        let Some(choice) = browser.choice() else {
            return;
        };
        let revision = browser.revision;
        browser.attempted = Some(revision);
        browser.running = Some(revision);
        let name = choice.name.clone();
        let task = self.resolve_choice(&choice, cx);
        cx.spawn(async move |this, cx| {
            let result = task.await;
            let _ = this.update(cx, |this, cx| {
                this.themes.finish_preview(revision, name, result);
                this.drive_theme_preview(cx);
                cx.notify();
            });
        })
        .detach();
    }

    pub(super) fn select_settings_theme(&mut self, index: usize, cx: &mut Context<Self>) {
        if self.persistence.quitting()
            || self.persistence.closing()
            || index >= self.themes.filtered.len()
        {
            return;
        }
        self.themes.selected = Some(index);
        self.themes
            .scroll
            .scroll_to_item(index / self.themes.grid.columns, ScrollStrategy::Center);
        self.request_theme_preview(cx);
        if let Some(mut choice) = self.themes.choice() {
            if choice.scope == Scope::App {
                choice.name = ThemeName::with_side(
                    self.drafted_theme(),
                    self.themes.editing_light,
                    &choice.name,
                );
            }
            self.accept_theme_choice(choice, cx);
        }
    }

    fn theme_browser_key(
        &mut self,
        event: &KeyDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.themes.search.read(cx).is_composing() {
            // Do not let the window's Escape action close during IME composition.
            if event.keystroke.key == "escape" {
                cx.stop_propagation();
            }
            return;
        }
        match event.keystroke.key.as_str() {
            "escape" => {
                cx.stop_propagation();
                window.prevent_default();
                self.themes.search.update(cx, |search, cx| search.clear(cx));
            }
            "up" | "down" => {
                cx.stop_propagation();
                window.prevent_default();
                let count = self.themes.filtered.len();
                if count > 0 {
                    let index = self.themes.selected.unwrap_or(0);
                    let next = grid::navigate(
                        index,
                        count,
                        self.themes.grid.columns,
                        event.keystroke.key == "up",
                    );
                    self.select_settings_theme(next, cx);
                }
            }
            "enter" => {
                cx.stop_propagation();
                window.prevent_default();
                if let Some(index) = self.themes.selected {
                    self.select_settings_theme(index, cx);
                }
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests;
