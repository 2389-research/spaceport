// ABOUTME: Owns the independent native preferences window and its draft settings.
// ABOUTME: Keeps persistence and configuration watches separate from window rendering.
//! Independent native preferences window. Disk work never owns a window or a socket.
// i18n: migrated
mod catalog;
mod controls;
mod drafts;
mod inputs;
#[cfg(windows)]
mod launch_at_login;
mod layouts;
mod navigation;
mod page_state;
mod pages;
pub(crate) use drafts::{
    apply_loaded_layout, apply_loaded_sidebar, apply_loaded_theme, layout_load_revision,
    sidebar_load_revision, theme_load_revision, theme_pending,
};
#[cfg(all(feature = "integration-test", target_os = "macos"))]
mod native;
#[cfg(all(feature = "integration-test", target_os = "macos"))]
pub(crate) use native::branding::capture as capture_branding;
#[cfg(all(feature = "integration-test", target_os = "macos"))]
pub(crate) use native::core_opt_in::verify as verify_core_opt_in;
mod persistence;
mod remote_history;
mod scrollbar;
mod search;
mod section;
use section::Section;
mod sidebar;
mod source;
use page_state::PageState;
#[cfg(test)]
use persistence::SizeIo;
use persistence::{Kind, Lifecycle, Loaded, SaveCompletion};
use sidebar::SidebarIntent;
use source::SourceWindow;
mod themes;
mod ui;
#[cfg(all(feature = "integration-test", target_os = "macos"))]
pub(crate) use native::verify_native;

use crate::direction::Logical;
use crate::{
    SpaceportWindow,
    config::{Config, Theme},
    fonts::StyledFont,
    herdr_settings,
    i18n::{Localize, fl},
};
use gpui::{prelude::*, *};

actions!(settings_window, [Close]);

pub(crate) fn key_bindings() -> [KeyBinding; 2] {
    [
        KeyBinding::new(
            if crate::controls::CTRL_SHORTCUTS {
                "ctrl-shift-w"
            } else {
                "cmd-w"
            },
            Close,
            Some("SettingsWindow"),
        ),
        KeyBinding::new("ctrl-w", Close, Some("SettingsWindow")),
    ]
}

#[derive(Default)]
struct SettingsWindowHandle {
    window: Option<WindowHandle<SettingsWindow>>,
    model: Option<WeakEntity<SettingsWindow>>,
}
impl Global for SettingsWindowHandle {}

pub(crate) fn open(source: WeakEntity<SpaceportWindow>, cx: &mut App) {
    cx.defer(move |cx| {
        open_with(source, cx, |view, window, cx| {
            view.initialize_theme_browser(window, cx);
            view.initialize_controls(cx);
            #[cfg(windows)]
            view.refresh_launch_at_login(None, cx);
            view.reload(cx);
            view.watch_config(cx);
        })
    });
}

fn open_with(
    source: WeakEntity<SpaceportWindow>,
    cx: &mut App,
    initialize: impl FnOnce(&mut SettingsWindow, &mut Window, &mut Context<SettingsWindow>) + 'static,
) {
    if let Some(handle) = cx.default_global::<SettingsWindowHandle>().window
        && handle
            .update(cx, |view, window, cx| {
                view.retarget_source(source.clone(), cx);
                window.activate_window();
            })
            .is_ok()
    {
        return;
    }
    // A detached save retains this model after close. Reopening must not create
    // a second editor with a stale baseline while that write is still running.
    let retained = cx
        .default_global::<SettingsWindowHandle>()
        .model
        .as_ref()
        .and_then(WeakEntity::upgrade);
    let bounds = Bounds::centered(None, size(px(960.), px(780.)), cx);
    match cx.open_window(
        WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(bounds)),
            window_min_size: Some(size(px(680.), px(560.))),
            titlebar: Some(crate::titlebar::options(&fl!("settings-title"))),
            app_owns_titlebar_drag: cfg!(target_os = "macos"),
            ..Default::default()
        },
        move |window, cx| {
            crate::taskbar::mark(window, cx);
            let model = retained.unwrap_or_else(|| {
                cx.new(|cx| {
                    let mut view = SettingsWindow::new(source.clone(), cx);
                    initialize(&mut view, window, cx);
                    view
                })
            });
            model.update(cx, |view, cx| {
                view.retarget_source(source, cx);
                #[cfg(windows)]
                view.watch_launch_at_login(window, cx);
                view._appearance = Some(cx.observe_window_appearance(window, |view, _, cx| {
                    view.apply_window_appearance(cx);
                }));
            });
            let weak = model.downgrade();
            window.on_window_should_close(cx, move |window, cx| {
                weak.update(cx, |view, cx| view.should_close(window, cx))
                    .unwrap_or(true)
            });
            let focus = model.read(cx).focus.clone();
            window.focus(&focus, cx);
            cx.default_global::<SettingsWindowHandle>().model = Some(model.downgrade());
            model
        },
    ) {
        Ok(handle) => cx.default_global::<SettingsWindowHandle>().window = Some(handle),
        Err(_) => tracing::error!("Unable to open Settings window"),
    }
}

struct SettingsWindow {
    caption: crate::caption::Caption,
    config: Config,
    /// The keymap `config` resolves into, so a published appearance carries a
    /// settings and keymap pair that always agree.
    keymap: spaceport_keymap::Keymap,
    /// The config as the last successful load read it, which sidebar saves
    /// diff against; previews change only `config`.
    loaded_config: Config,
    theme: Theme,
    shared: Option<herdr_settings::Settings>,
    source: WeakEntity<SpaceportWindow>,
    /// What this window reads of `source`. The app follows the window through
    /// the subscription below, so no page or control upgrades it to read a
    /// field; the calls that change the source still use `source` itself.
    source_window: SourceWindow,
    section: Section,
    /// Catalog indices matching the query in `nav_search`; the body shows
    /// them in place of `section` while it is `Some`.
    search_results: Option<Vec<usize>>,
    nav_search: Entity<crate::search_input::SearchInput>,
    themes: themes::ThemeBrowser,
    controls: controls::Controls,
    chat_env: controls::chat_env::ChatEnvSettings,
    /// The open override dropdown, if any; its list renders inside its button.
    dropdown: Option<ui::Dropdown>,
    /// The number field being typed into, if any.
    number_edit: Option<inputs::NumberEdit>,
    /// A slider being dragged, or released and saving, that writes only on
    /// release.
    number_preview: Option<inputs::NumberPreview>,
    /// The host colour being typed as hex, if any.
    hex_edit: Option<sidebar::HexEdit>,
    #[cfg(windows)]
    launch_at_login: launch_at_login::LaunchAtLogin,
    error: Option<String>,
    status: Option<String>,
    focus: FocusHandle,
    body_scroll: ScrollHandle,
    /// The section list scrolls when the window is too short for it.
    navigation_scroll: ScrollHandle,
    /// Where on a scrollbar's thumb the pointer took hold of it.
    thumb_grab: f32,
    /// The section list's width, dragged by its right edge.
    navigation_width: crate::panel_resize::PanelWidth,
    /// The window's width at its last render, which caps the section list.
    viewport_width: f32,
    persistence: Lifecycle,
    /// The window to remove once the writes a close waits on have drained.
    close_when_idle: Option<AnyWindowHandle>,
    save_completion: Option<std::sync::mpsc::Receiver<SaveCompletion>>,
    _quit: Subscription,
    _search: Subscription,
    #[cfg(test)]
    size_io: Option<SizeIo>,
    _source: Option<Subscription>,
    _source_released: Option<Subscription>,
    _appearance: Option<Subscription>,
    _watch: Option<Task<()>>,
    /// The config files as our last save left them, for the watcher to accept
    /// so the save's own write does not trigger a second reload.
    saved_sample: Option<persistence::Sample>,
    /// Enumerating system fonts takes hundreds of milliseconds, so each
    /// Settings window does it once rather than on every load.
    installed_fonts: std::sync::Arc<std::sync::OnceLock<Vec<String>>>,
    load_revision: u64,
    theme_revision: u64,
    theme_intent: Option<themes::ThemeIntent>,
    layout_intent: Option<crate::config::LayoutMode>,
    sidebar_intent: Option<SidebarIntent>,
    #[cfg(test)]
    layout_io: Option<layouts::LayoutIo>,
    remote_history: remote_history::RemoteHistory,
    theme_loading: bool,
    theme_waiting: bool,
    theme_light: bool,
    theme_load_failed: bool,
    theme_cache: std::collections::VecDeque<(String, Theme)>,
    _theme_guard: Subscription,
    #[cfg(test)]
    theme_io: Option<themes::ThemeIo>,
}

impl SettingsWindow {
    #[allow(clippy::too_many_lines, reason = "2026-11-27 baseline: 121 lines")]
    fn new(source: WeakEntity<SpaceportWindow>, cx: &mut Context<Self>) -> Self {
        let appearance = cx
            .try_global::<crate::app::InitialAppearance>()
            .cloned()
            .unwrap_or_default();
        let upgraded = source.upgrade();
        let source_window = upgraded.as_ref().map_or_else(
            || SourceWindow::from_appearance(&appearance),
            |source| SourceWindow::of(source.read(cx)),
        );
        // The window this one reads: observed so the snapshot follows it, and
        // released so the facts that described a live connection go with it.
        let source_changes = upgraded
            .as_ref()
            .map(|source| cx.observe(source, Self::source_changed));
        let source_released =
            upgraded.map(|source| cx.observe_release(&source, Self::source_released));
        let quit = cx.on_app_quit(|this, cx| {
            let task = this.shutdown(cx);
            async move {
                if task.await.is_err() {
                    tracing::warn!("Unable to finish queued Settings changes during shutdown");
                }
            }
        });
        let nav_search = cx.new(crate::search_input::SearchInput::new);
        nav_search.update(cx, |input, cx| {
            input.set_placeholder_with(|| fl!("settings-search-placeholder"), cx)
        });
        let search_changed = cx.subscribe(
            &nav_search,
            |this, input, _: &crate::search_input::Changed, cx| {
                let query = input.read(cx).text().to_owned();
                this.set_search_query(&query, cx);
            },
        );
        Self {
            caption: Default::default(),
            loaded_config: source_window.config.clone(),
            config: source_window.config.clone(),
            keymap: source_window.keymap.clone(),
            theme: source_window.theme.clone(),
            shared: source_window.shared.clone(),
            source,
            source_window,
            section: Section::Theme,
            search_results: None,
            _search: search_changed,
            nav_search,
            themes: themes::ThemeBrowser::new(cx),
            controls: controls::Controls::new(cx),
            chat_env: Default::default(),
            dropdown: None,
            number_edit: None,
            number_preview: None,
            hex_edit: None,
            #[cfg(windows)]
            launch_at_login: Default::default(),
            error: appearance.error,
            status: None,
            focus: cx.focus_handle(),
            body_scroll: ScrollHandle::new(),
            navigation_scroll: ScrollHandle::new(),
            thumb_grab: 0.,
            navigation_width: crate::panel_resize::SETTINGS_NAVIGATION,
            viewport_width: 0.,
            persistence: Lifecycle::default(),
            close_when_idle: None,
            save_completion: None,
            _quit: quit,
            #[cfg(test)]
            size_io: None,
            _source: source_changes,
            _source_released: source_released,
            _appearance: None,
            _watch: None,
            saved_sample: None,
            installed_fonts: Default::default(),
            load_revision: 0,
            theme_revision: 0,
            theme_intent: None,
            layout_intent: None,
            sidebar_intent: None,
            #[cfg(test)]
            layout_io: None,
            remote_history: Default::default(),
            theme_loading: false,
            theme_waiting: false,
            theme_light: false,
            theme_load_failed: false,
            theme_cache: Default::default(),
            // Shared-theme/OS callbacks in main windows also publish here. Fence
            // their prepared, older palette while an explicit choice is live.
            _theme_guard: cx.observe_global::<crate::app::InitialAppearance>(|this, cx| {
                if this.theme_intent.is_some()
                    && cx
                        .try_global::<crate::app::InitialAppearance>()
                        .is_some_and(|appearance| {
                            appearance.config.theme != this.config.theme
                                || appearance.theme != this.theme
                        })
                {
                    this.drive_theme_intent(cx);
                }
                if this.layout_intent.is_some()
                    && cx
                        .try_global::<crate::app::InitialAppearance>()
                        .is_some_and(|appearance| {
                            appearance.config.layout.mode != this.config.layout.mode
                        })
                {
                    this.broadcast_layout(cx);
                }
                if this.sidebar_intent.is_some()
                    && cx
                        .try_global::<crate::app::InitialAppearance>()
                        .is_some_and(|appearance| {
                            SidebarIntent::of(&appearance.config) != SidebarIntent::of(&this.config)
                        })
                {
                    this.broadcast_sidebar(cx);
                }
            }),
            #[cfg(test)]
            theme_io: None,
        }
    }

    fn source_changed(&mut self, source: Entity<SpaceportWindow>, cx: &mut Context<Self>) {
        self.source_window.follow(source.read(cx));
        if self.section == Section::Integrations {
            cx.notify();
        }
        if matches!(self.section, Section::General | Section::Tabs) {
            self.sync_remote_history(false, cx);
        }
    }

    /// The window Settings read is gone: the facts that described a live
    /// connection go with it, and what it opened with stays.
    fn source_released(&mut self, _: &mut SpaceportWindow, cx: &mut Context<Self>) {
        self.source_window.release();
        cx.notify();
    }

    fn retarget_source(&mut self, source: WeakEntity<SpaceportWindow>, cx: &mut Context<Self>) {
        if source.entity_id() == self.source.entity_id() || self.source_window.integrations_busy {
            return;
        }
        let Some(owner) = source.upgrade() else {
            return;
        };
        self._source = Some(cx.observe(&owner, Self::source_changed));
        self._source_released = Some(cx.observe_release(&owner, Self::source_released));
        self.source = source;
        self.source_window.follow(owner.read(cx));
        if self.section == Section::Integrations {
            owner.update(cx, |source, cx| source.load_integrations(cx));
        }
        if matches!(self.section, Section::General | Section::Tabs) {
            self.sync_remote_history(false, cx);
        }
        cx.notify();
    }

    fn apply_window_appearance(&mut self, cx: &mut Context<Self>) {
        self.follow_system_appearance(cx);
        self.sync_appearance(cx);
        self.drive_theme_intent(cx);
        self.publish_appearance(cx);
    }

    fn sync_appearance(&mut self, cx: &mut Context<Self>) {
        let light = crate::app::light_appearance(cx);
        if crate::config::ThemeName::follows_herdr(&self.config.theme, light)
            && self.theme_intent.is_none()
            && let Some(shared) = &self.shared
        {
            match shared.theme(light) {
                Ok(theme) => self.theme = theme.styled_for(&self.config),
                Err(error) => {
                    self.error = Some(fl!(
                        "settings-error-system-appearance",
                        error = error.localize()
                    ));
                }
            }
        }
        // The theme browser owns its unsaved preview; changing system chrome
        // preserves its selected choice instead of selecting the applied theme.
        self.sync_theme_browser(cx);
        self.sync_controls(cx);
        cx.notify();
    }

    fn publish_appearance(&self, cx: &mut Context<Self>) {
        // An in-flight load/save will publish its validated result. In
        // particular, initial source chrome may still be a picker preview.
        if self.busy() {
            return;
        }
        let previous = cx.try_global::<crate::app::InitialAppearance>();
        let keys_changed = previous.is_none_or(|previous| previous.keymap != self.keymap);
        let menus_changed =
            previous.is_some_and(|previous| crate::menus::changed(&previous.config, &self.config));
        // Future windows inherit an explicit Settings draft, but not the source
        // window's cancellable hover preview. Logs follow live appearance.
        let previewing = self.source_window.theme_page_open;
        let mut appearance = crate::app::InitialAppearance {
            config: self.config.clone(),
            keymap: self.keymap.clone(),
            theme: self.theme.clone(),
            error: None,
        };
        if let Some(drafts) = drafts::drafts(cx) {
            drafts.apply_theme_draft(&mut appearance.config, &mut appearance.theme);
        }
        cx.set_global(appearance);
        if !previewing {
            crate::log_window::set_appearance(&self.config, &self.theme, cx);
        }
        if keys_changed {
            crate::actions::rebind_keys(cx);
        } else if menus_changed {
            crate::menus::install(cx);
        }
    }

    pub(super) fn busy(&self) -> bool {
        self.persistence.busy()
    }

    pub(super) fn theme_dirty(&self) -> bool {
        self.theme_intent.is_some()
    }

    /// Starts the writes the footer's "Save now" stands for; completions chain
    /// theme, then layout, then sidebar, through `finish_flush`.
    fn save_pending_drafts(&mut self, cx: &mut Context<Self>) {
        if self.busy() {
            return;
        }
        self.theme_load_failed = false;
        self.persistence.flush();
        self.finish_flush(cx);
    }

    fn finish_flush(&mut self, cx: &mut Context<Self>) {
        if !self.persistence.flushing() {
            return;
        }
        if self.busy() {
            return;
        }
        if self.theme_dirty() {
            self.drive_theme_intent(cx);
        } else if self.layout_intent.is_some() {
            self.save_layout_draft(cx);
        } else if self.sidebar_dirty() {
            self.save_sidebar_draft(cx);
        } else {
            self.persistence.stop_flush();
            self.status = None;
            cx.notify();
        }
    }

    /// Shared entry point for the native titlebar and keyboard close actions.
    pub(super) fn should_close(&mut self, window: &mut Window, cx: &mut Context<Self>) -> bool {
        self.hover_theme(None, cx);
        if self.persistence.closing() {
            return false;
        }
        if !self.finish_number_edit(true, cx) || !self.finish_hex_edit(true, cx) {
            return false;
        }
        self.dropdown = None;
        if !self.theme_dirty()
            && self.layout_intent.is_none()
            && !self.sidebar_dirty()
            && !self.busy()
        {
            return true;
        }
        self.close_when_idle = Some(window.window_handle());
        self.persistence.close();
        self.status = Some(fl!("settings-finishing"));
        self.theme_load_failed = false;
        self.drive_theme_intent(cx);
        false
    }

    pub(super) fn close(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.should_close(window, cx) {
            window.remove_window();
        }
    }

    fn finish_close(&mut self, cx: &mut Context<Self>) {
        self.finish_flush(cx);
        if self.persistence.sidebar_queued() && !self.persistence.closing() && !self.busy() {
            self.save_queued_sidebar(cx);
        }
        if !self.busy() && !self.theme_dirty() && self.persistence.closing() {
            if self.layout_intent.is_some() {
                self.save_layout_draft(cx);
                return;
            }
            if self.sidebar_dirty() {
                self.save_sidebar_draft(cx);
                return;
            }
        }
        if !self.busy()
            && !self.theme_dirty()
            && self.layout_intent.is_none()
            && !self.sidebar_dirty()
            && let Some(handle) = self.close_when_idle.take()
            && self.persistence.take_close()
        {
            cx.defer(move |cx| {
                let _ = handle.update(cx, |_, window, _| window.remove_window());
            });
        }
    }

    /// Where "New window" connects: the source's first endpoint as it is now,
    /// since a new window belongs where the source is, or the target this
    /// window's source snapshot kept once that window closed.
    fn additional_window_target(&self, cx: &App) -> Option<spaceport_client::ConnectTarget> {
        self.source
            .upgrade()
            .and_then(|source| {
                source
                    .read(cx)
                    .endpoints
                    .first()
                    .map(|endpoint| endpoint.connection.target.clone())
            })
            .or_else(|| self.source_window.target.clone())
    }

    pub(super) fn select_section(
        &mut self,
        section: Section,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.hover_theme(None, cx);
        self.drop_number_preview(cx);
        if !self.finish_number_edit(true, cx) {
            self.finish_number_edit(false, cx);
        }
        if !self.finish_hex_edit(true, cx) {
            self.finish_hex_edit(false, cx);
        }
        self.dropdown = None;
        self.dismiss_control_font_picker(window, cx);
        self.clear_search(cx);
        self.section = section;
        // A page opened from search or elsewhere shows in a short list too.
        self.navigation_scroll.scroll_to_item(section.nav_item());
        if section == Section::General {
            self.open_chat_env_settings(cx);
        }
        self.body_scroll.set_offset(Point::default());
        window.focus(&self.focus, cx);
        if section == Section::Integrations {
            let _ = self
                .source
                .update(cx, |source, cx| source.load_integrations(cx));
        }
        if matches!(section, Section::General | Section::Tabs) {
            self.sync_remote_history(false, cx);
        }
        cx.notify();
    }
}

impl Render for SettingsWindow {
    #[allow(clippy::too_many_lines, reason = "2026-11-27 baseline: 104 lines")]
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.caption.sync(window, &self.theme);
        let searching = self.search_results.clone();
        let heading = if searching.is_some() {
            fl!("settings-results")
        } else {
            crate::i18n::message(self.section.message(), &[])
        };
        let content = match &searching {
            Some(_) => self.render_search_results(cx),
            None => match self.section {
                Section::Theme => self.render_theme(window, cx),
                Section::Integrations => self.render_integration_controls(cx),
                _ => self.render_controls(window, cx),
            },
        };
        self.viewport_width = f32::from(window.viewport_size().width);
        let navigation = self.navigation(cx);
        let font_picker = self.render_control_font_picker(window, cx);
        let this = cx.entity().downgrade();
        let header = crate::titlebar::header(&self.theme, window, move |window, cx| {
            let _ = this.update(cx, |view, cx| view.close(window, cx));
        });
        let theme = &self.theme;
        let root = div()
            .key_context("SettingsWindow")
            .relative()
            .track_focus(&self.focus)
            .on_key_down(cx.listener(Self::settings_key))
            .on_action(cx.listener(|this, action: &crate::RunCommand, window, cx| {
                match action.command {
                    // Its own close path, which saves pending edits first.
                    crate::controls::Command::CloseWindow => this.close(window, cx),
                    command if crate::window::run_window_command(command, window, cx) => {}
                    crate::controls::Command::Settings => window.activate_window(),
                    crate::controls::Command::NewWindow => {
                        if let Some(target) = this.additional_window_target(cx) {
                            crate::app::open_additional_window(target, cx);
                        }
                    }
                    _ => {}
                }
                // Session commands never cross from this independent root to
                // the integration source, even when it is still alive.
                cx.stop_propagation();
            }))
            .on_action(cx.listener(|this, _: &Close, window, cx| {
                cx.stop_propagation();
                this.close(window, cx);
            }))
            .size_full()
            .flex()
            .flex_col()
            .overflow_hidden()
            .text_font(&self.config.ui)
            .text_size(px(12.))
            .line_height(px(18.))
            .bg(rgb(theme.background))
            .text_color(rgb(theme.foreground))
            .text_start()
            .children(header)
            .child(
                div().flex_1().min_h_0().row().child(navigation).child(
                    self.scroll_area(
                        scrollbar::Pane::Body,
                        div()
                            .id("settings-body")
                            .debug_selector(|| "settings-body".into())
                            .role(Role::TabPanel)
                            .aria_label(heading.clone())
                            .flex_1()
                            .min_w_0()
                            .min_h_0()
                            .overflow_y_scroll()
                            .track_scroll(&self.body_scroll)
                            .p(px(28.))
                            .child(
                                div()
                                    .text_size(px(18.))
                                    .line_height(px(26.))
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .child(heading),
                            )
                            .child(
                                div().mb(px(16.)).when_some(
                                    self.section
                                        .lead()
                                        .filter(|_| searching.is_none())
                                        .map(|message| crate::i18n::message(message, &[])),
                                    |header, lead| {
                                        header.mt(px(2.)).child(
                                            div()
                                                .text_size(px(12.))
                                                .text_color(rgb(theme.subtext()))
                                                .child(lead),
                                        )
                                    },
                                ),
                            )
                            .child(content),
                        cx,
                    ),
                ),
            )
            .child(self.footer(cx))
            .children(font_picker);
        let border = self.theme.active;
        crate::titlebar::frame(window, border, root)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests;
