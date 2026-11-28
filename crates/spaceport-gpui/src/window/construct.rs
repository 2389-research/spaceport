// ABOUTME: Constructs native windows and their owned presentation state.
// ABOUTME: Copies the app startup gate before any Core work can begin.
//! Building a window: its effects run in a fixed order before the fields,
//! which stay sorted so parallel additions land on different lines.
// i18n: migrated

use super::{SpaceportWindow, SurfaceSignal, function_keys, prefix};
#[cfg(feature = "integration-test")]
use crate::smoke;
use crate::{
    WINDOW_TITLE,
    app::WindowLaunch,
    config::{Config, Theme},
    endpoint, log_window, menu, preferences, sessions, sidebar,
    state::LiveState,
    terminal::{Selection, WheelAccumulator},
};
use gpui::{prelude::*, *};
use spaceport_client::ConnectOptions;
#[cfg(feature = "integration-test")]
use spaceport_client::ConnectTarget;
#[cfg(feature = "integration-test")]
use std::sync::Arc;
use std::time::Duration;

impl SpaceportWindow {
    #[allow(clippy::too_many_lines, reason = "2026-11-27 baseline: 203 lines")]
    pub(crate) fn new(
        launch: impl Into<WindowLaunch>,
        window: &mut Window,
        cx: &mut Context<Self>,
        #[cfg(feature = "integration-test")] sidebar_test: bool,
    ) -> Self {
        let launch = launch.into();
        #[cfg(feature = "integration-test")]
        let launch = match launch {
            WindowLaunch::Legacy(_) if sidebar_test => {
                WindowLaunch::Legacy(ConnectTarget::Socket("/unused-sidebar-fixture.sock".into()))
            }
            launch => launch,
        };
        let focus = cx.focus_handle();
        window.focus(&focus, cx);
        let weak = cx.weak_entity();
        let startup = Startup::of(cx);
        let sidebar_view = cx.new(|_| sidebar::SidebarView::new(weak, startup.indicators));
        let hosts = vec![cx.entity_id(), sidebar_view.entity_id()];
        let mark_layer = cx.new(|cx| crate::mark_layer::MarkLayer::new(window, hosts, cx));
        let effects = Self::window_effects(window, cx, &focus);
        let launched = Launched::of(launch);
        // Effects run here in a fixed order, so the fields below can stay sorted.
        let configured_terminal_size = startup.config.terminal.size;
        let status_bar_visible = startup.config.status_bar.show;
        let mut gui_config_diagnostic = crate::config_diagnostic::ConfigDiagnostic::default();
        gui_config_diagnostic.sync_own(crate::config::diagnostic(&startup.config).as_deref());
        let mut notices = crate::notice::Notices::default();
        if let Some(error) = startup.error.clone() {
            notices.show(crate::notice::Cause::Configuration, error);
        }
        let menu = menu::MenuState::new(cx);
        let sidebar_search = sidebar::SidebarSearch::new(cx);
        let surface_signal = cx.new(|_| SurfaceSignal);
        let browser = crate::browser::new_browser(cx);
        let device_tags = crate::device_tags::DeviceTags::current(cx);
        // The Core group this window shows, if it was opened for one.
        let core_key = launched.core_key.clone();
        let mut this = Self {
            // keep-sorted start
            _activation: effects.activation,
            _appearance: effects.appearance,
            _browser_tabs: effects.browser_tabs,
            _device_tags: effects.device_tags_changed,
            _host_theme: effects.host_theme,
            _notification_mutes: effects.notification_mutes,
            _poll: effects.poll,
            _prefix_interceptor: effects.prefix_interceptor,
            _sidebar_invalidation: effects.sidebar_invalidation,
            _terminal_focus: effects.terminal_focus,
            _window_closed: effects.window_closed,
            _worktree_notes: effects.worktree_notes,
            about_demo: crate::about::Demo::Starfield,
            active: window.is_window_active(),
            activity: Default::default(),
            agent_section_folds: [sidebar::AgentSection::Quiet].into(),
            agent_sort: preferences::AgentSort::default(),
            agent_sort_modified: false,
            arrival: None,
            backdrop: Default::default(),
            bell: crate::bell::Bell::default(),
            bounds: Bounds::default(),
            browser,
            catalog: launched.catalog,
            cell_width: 9.,
            chats: Default::default(),
            chrome_region: None,
            config: startup.config,
            config_load: None,
            config_load_revision: 0,
            config_watch: None,
            configured_terminal_size,
            copy_mode: None,
            core_terminals: crate::core_terminals::CoreTerminals::new(
                crate::core_feature::enabled(cx),
            ),
            daemon_text: Default::default(),
            device_drag: None,
            device_filter: Default::default(),
            device_order: Vec::new(),
            device_tag_filter: None,
            device_tag_folds: Default::default(),
            device_tags,
            device_themes: Default::default(),
            devices_overview: Default::default(),
            endpoints: launched.endpoints,
            file_transfer: Default::default(),
            find: None,
            find_memory: Default::default(),
            flash: None,
            flash_motion: Default::default(),
            focus,
            group_pictures: Default::default(),
            group_terminals: Default::default(),
            gui_config_diagnostic,
            held_keys: Default::default(),
            hover: None,
            hover_menu: None,
            hovered_terminal_link: false,
            icon_font_notice: Default::default(),
            #[cfg(feature = "integration-test")]
            input_probe: smoke::InputProbe::default(),
            integrations: Default::default(),
            key_layout: prefix::current_layout,
            key_press: function_keys::current_press,
            keymap: startup.keymap,
            last_input: None,
            last_pointer: None,
            last_queued_options: None,
            links: Default::default(),
            listening_ports: Default::default(),
            live: LiveState::default(),
            mark_layer,
            marked: String::new(),
            marked_selection: None,
            menu,
            #[cfg(feature = "integration-test")]
            navigations: Default::default(),
            network_quality_seen: 0,
            notice_overflow: Default::default(),
            notices,
            options: ConnectOptions::default(),
            overview: Default::default(),
            painter: Default::default(),
            pending_images: Vec::new(),
            pending_input: Default::default(),
            pending_navigation: None,
            pending_releases: Vec::new(),
            pending_resize: None,
            pending_toast: None,
            port_forwards: Default::default(),
            prefix_armed: false,
            presentation: Default::default(),
            pressed_terminal_link: None,
            regions: Vec::new(),
            resize_mode: false,
            scrollbar_drag: None,
            selected_endpoint: 0,
            selected_generation: 0,
            selection: None,
            selection_epoch: 0,
            selection_follow: Default::default(),
            sent_focus: None,
            server_keys: None,
            sessions: sessions::Sessions::default(),
            sessions_anchor: Default::default(),
            settings: Default::default(),
            shift_taps: Default::default(),
            sidebar_drag: None,
            sidebar_folded: Default::default(),
            sidebar_modified: false,
            sidebar_nested_reveal: Default::default(),
            sidebar_pin_reveal: Default::default(),
            sidebar_preferences: None,
            sidebar_revealed: Default::default(),
            sidebar_scroll: Default::default(),
            sidebar_search,
            sidebar_slide: Default::default(),
            sidebar_split: None,
            sidebar_split_modified: false,
            sidebar_start_pending: true,
            sidebar_view,
            sidebar_visible: true,
            sidebar_width: None,
            sidebar_workspaces_expanded: true,
            sound: Default::default(),
            split_cursor: None,
            split_drag: None,
            status_bar_visible,
            strip_navigation_fits: true,
            surface_signal,
            tab_drag: None,
            terminal_mouse: None,
            theme: startup.theme,
            theme_light: crate::app::light_appearance(cx),
            title: WINDOW_TITLE.to_owned(),
            toast_motion: Default::default(),
            toasts_hidden: false,
            tunnels: Default::default(),
            usage: Default::default(),
            version_notice_shown: false,
            viewport_width: 0.,
            wake: endpoint::WakeClock::now(),
            wheel: WheelAccumulator::default(),
            workspace_drag: None,
            wsl_paste: None,
            zen: Default::default(),
            // keep-sorted end
        };

        // Quitting need not drop this window, so its SSH children (port
        // forwards and listening-port tunnels) are killed here rather than
        // left forwarding after the app is gone.
        cx.on_app_quit(|this, _| {
            this.port_forwards.stop_all();
            this.tunnels.close_all();
            async {}
        })
        .detach();
        if let Some(key) = core_key.as_ref() {
            this.show_core_group(key, cx);
        }
        #[cfg(feature = "integration-test")]
        if sidebar_test {
            this._poll = Task::ready(());
            if let Some(endpoint) = this.endpoints.first_mut() {
                this.live.snapshot = Some(Arc::new(sidebar::layout_tests::snapshot(40)));
                *endpoint.live_mut() = this.live.clone();
                if let Ok(mut inbox) = endpoint.connection.inbox.lock() {
                    *inbox = this.live.clone();
                }
            }
            return this;
        }
        this.start(core_key, window, cx);
        this
    }

    /// The observers and task a new window holds: what it draws from, what it
    /// gives back when it closes, and the frame tick that keeps it current.
    fn window_effects(window: &mut Window, cx: &mut Context<Self>, focus: &FocusHandle) -> Effects {
        let timer = cx.background_executor().clone();
        let poll = cx.spawn_in(window, async move |this, cx| {
            let mut next_idle_refresh = std::time::Instant::now() + Duration::from_secs(60);
            loop {
                timer.timer(Duration::from_millis(16)).await;
                if this
                    .update_in(cx, |this, window, cx| {
                        this.tick(window, cx);
                        let now = std::time::Instant::now();
                        if now >= next_idle_refresh {
                            next_idle_refresh = now + Duration::from_secs(60);
                            if this.has_live_idle_rows() {
                                this.sidebar_view.update(cx, |_, cx| cx.notify());
                            }
                        }
                    })
                    .is_err()
                {
                    break;
                }
            }
        });
        let sidebar_invalidation = Self::invalidate_sidebar(cx);
        let host_theme = Self::observe_host_theme(cx);
        // Another window, or an agent, may open or close a tab.
        let browser_tabs = cx.observe_global::<crate::browser::Store>(|_, cx| cx.notify());
        let device_tags_changed = Self::observe_device_tags(cx);
        let worktree_notes = Self::observe_worktree_notes(window, cx);
        let notification_mutes = Self::observe_notification_mutes(cx);
        let prefix_interceptor = Self::intercept_prefix(window, cx);
        let activation = cx.observe_window_activation(window, |this, window, cx| {
            this.active = window.is_window_active();
            if this.active {
                this.publish_server_keymap(cx);
            } else {
                this.shift_taps.cancel();
                this.disarm_prefix();
                this.cancel_terminal_mouse(cx);
                // A drag cut short is dropped; a selection kept for an
                // explicit copy outlives a trip to another window.
                if this.selection.as_ref().is_some_and(Selection::dragging) {
                    this.selection = None;
                }
                this.pressed_terminal_link = None;
            }
            this.report_focus();
            cx.notify();
        });
        let appearance = cx.observe_window_appearance(window, |this, _, cx| {
            this.apply_shared_theme(cx);
            this.apply_system_theme(cx);
            cx.notify();
        });
        let window_closed = Self::give_back_torn_tabs(cx);
        let terminal_focus = Self::leave_region_with_terminal(focus, window, cx);
        Effects {
            activation,
            appearance,
            browser_tabs,
            device_tags_changed,
            host_theme,
            notification_mutes,
            poll,
            prefix_interceptor,
            sidebar_invalidation,
            terminal_focus,
            window_closed,
            worktree_notes,
        }
    }

    /// What a window does once its state exists: it shows the Core group it
    /// was opened for, reconnects to its daemon, and reads the settings that
    /// arrive with it.
    fn start(
        &mut self,
        core_key: Option<crate::browser::CoreGroupKey>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        Self::poll_on_frame(cx.entity().downgrade(), window);
        self.sidebar_preferences = if let Some(key) = core_key.as_ref() {
            Some(preferences::Preferences::for_core(key.hub_core_id(), cx))
        } else {
            self.endpoints
                .first()
                .and_then(|endpoint| endpoint.connection.target.socket_path().ok())
                .map(|path| preferences::Preferences::new(&path, cx))
        };
        if core_key.is_none() {
            self.reconnect(cx);
        }
        log_window::set_appearance(&self.config, &self.theme, cx);
        self.load_gui_config(cx);
        if core_key.is_none() {
            self.load_shared_settings(cx);
        }
        self.watch_gui_config(cx);
    }
}

/// What the app hands a new window: its config, keymap, theme, and the
/// startup error to show, with the indicators that config draws.
struct Startup {
    config: Config,
    error: Option<String>,
    indicators: sidebar::Indicators,
    keymap: spaceport_keymap::Keymap,
    theme: Theme,
}

impl Startup {
    fn of(cx: &App) -> Self {
        let appearance = cx
            .try_global::<crate::app::InitialAppearance>()
            .cloned()
            .unwrap_or_default();
        let indicators = sidebar::Indicators::new(
            None,
            appearance.config.status_icons,
            appearance.config.edge_rail,
            matches!(
                cx.window_appearance(),
                WindowAppearance::Light | WindowAppearance::VibrantLight
            ),
            &appearance.theme,
        );
        let crate::app::InitialAppearance {
            config,
            keymap,
            theme,
            error,
        } = appearance;
        Self {
            config,
            error,
            indicators,
            keymap,
            theme,
        }
    }
}

/// What a window was opened against: its endpoint catalog, the Core group it
/// shows, and the error it could not start with.
struct Launched {
    catalog: endpoint::Catalog,
    core_key: Option<crate::browser::CoreGroupKey>,
    endpoints: Vec<endpoint::Endpoint>,
}

impl Launched {
    fn of(launch: WindowLaunch) -> Self {
        match launch {
            WindowLaunch::Legacy(target) => Self {
                catalog: endpoint::Catalog::new(&target),
                core_key: None,
                endpoints: vec![endpoint::Endpoint::new(
                    endpoint::LOCAL.into(),
                    "Local".into(), // i18n: data
                    target,
                    true,
                )],
            },
            WindowLaunch::Core(key) => Self {
                catalog: endpoint::Catalog::inactive(),
                core_key: Some(key),
                endpoints: Vec::new(),
            },
        }
    }
}

/// The subscriptions and task a new window holds, so [`SpaceportWindow::new`]
/// can start them in one block and keep its fields sorted.
struct Effects {
    activation: Subscription,
    appearance: Subscription,
    browser_tabs: Subscription,
    device_tags_changed: Subscription,
    host_theme: Subscription,
    notification_mutes: Subscription,
    poll: Task<()>,
    prefix_interceptor: Subscription,
    sidebar_invalidation: Subscription,
    terminal_focus: Subscription,
    window_closed: Subscription,
    worktree_notes: Subscription,
}
