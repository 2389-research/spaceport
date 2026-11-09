//! Building a window: its effects run in a fixed order before the fields,
//! which stay sorted so parallel additions land on different lines.
// i18n: migrated

use super::{SheprdWindow, SurfaceSignal, function_keys, prefix};
#[cfg(feature = "integration-test")]
use crate::smoke;
use crate::{
    WINDOW_TITLE,
    app::WindowLaunch,
    endpoint, log_window, menu, preferences, sessions, sidebar,
    state::LiveState,
    terminal::{Selection, WheelAccumulator},
};
use gpui::{prelude::*, *};
use sheprd_client::ConnectOptions;
#[cfg(feature = "integration-test")]
use sheprd_client::ConnectTarget;
#[cfg(feature = "integration-test")]
use std::sync::Arc;
use std::time::Duration;

impl SheprdWindow {
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
        let (initial_theme, initial_icons, initial_rail) = cx
            .try_global::<crate::app::InitialAppearance>()
            .map(|appearance| {
                (
                    appearance.theme.clone(),
                    appearance.config.status_icons,
                    appearance.config.edge_rail,
                )
            })
            .unwrap_or_default();
        let indicators = sidebar::Indicators::new(
            None,
            initial_icons,
            initial_rail,
            matches!(
                cx.window_appearance(),
                WindowAppearance::Light | WindowAppearance::VibrantLight
            ),
            &initial_theme,
        );
        let sidebar_view = cx.new(|_| sidebar::SidebarView::new(weak, indicators));
        let hosts = vec![cx.entity_id(), sidebar_view.entity_id()];
        let mark_layer = cx.new(|cx| crate::mark_layer::MarkLayer::new(window, hosts, cx));
        let timer = cx.background_executor().clone();
        let poll = cx.spawn_in(window, async move |this, cx| {
            loop {
                timer.timer(Duration::from_millis(16)).await;
                if this
                    .update_in(cx, |this, window, cx| this.tick(window, cx))
                    .is_err()
                {
                    break;
                }
            }
        });
        let appearance = cx
            .try_global::<crate::app::InitialAppearance>()
            .cloned()
            .unwrap_or_default();
        let crate::app::InitialAppearance {
            config,
            theme,
            error,
        } = appearance;
        // Effects run here in a fixed order, so the fields below can stay sorted.
        let configured_terminal_size = config.terminal.size;
        let status_bar_visible = config.status_bar.show;
        let mut gui_config_diagnostic = crate::config_diagnostic::ConfigDiagnostic::default();
        gui_config_diagnostic.sync_own(config.diagnostic().as_deref());
        let (catalog, endpoints, core_key) = match launch {
            WindowLaunch::Legacy(target) => (
                endpoint::Catalog::new(&target),
                vec![endpoint::Endpoint::new(
                    endpoint::LOCAL.into(),
                    "Local".into(), // i18n: data
                    target,
                    true,
                )],
                None,
            ),
            WindowLaunch::Core(key) => (endpoint::Catalog::inactive(), Vec::new(), Some(key)),
        };
        let menu = menu::MenuState::new(cx);
        let sidebar_search = sidebar::SidebarSearch::new(cx);
        let surface_signal = cx.new(|_| SurfaceSignal);
        let sidebar_invalidation = Self::invalidate_sidebar(cx);
        let host_theme = Self::observe_host_theme(cx);
        let browser = crate::browser::Browser::new(cx);
        // Another window, or an agent, may open or close a tab.
        let browser_tabs = cx.observe_global::<crate::browser::Store>(|_, cx| cx.notify());
        let device_tags = crate::device_tags::DeviceTags::current(cx);
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
        let appearance_change = cx.observe_window_appearance(window, |this, _, cx| {
            this.apply_shared_theme(cx);
            this.apply_system_theme(cx);
            cx.notify();
        });
        let window_closed = Self::give_back_torn_tabs(cx);
        let terminal_focus = Self::leave_region_with_terminal(&focus, window, cx);
        let mut this = Self {
            // keep-sorted start
            _activation: activation,
            _appearance: appearance_change,
            _browser_tabs: browser_tabs,
            _device_tags: device_tags_changed,
            _host_theme: host_theme,
            _notification_mutes: notification_mutes,
            _poll: poll,
            _prefix_interceptor: prefix_interceptor,
            _sidebar_invalidation: sidebar_invalidation,
            _terminal_focus: terminal_focus,
            _window_closed: window_closed,
            _worktree_notes: worktree_notes,
            activation_deadline: None,
            active: window.is_window_active(),
            activity: Default::default(),
            agent_sort: preferences::AgentSort::default(),
            agent_sort_modified: false,
            arrival: None,
            backdrop: Default::default(),
            bell: crate::bell::Bell::default(),
            bounds: Bounds::default(),
            browser,
            catalog,
            cell_width: 9.,
            chats: Default::default(),
            chrome_region: None,
            collapsed_repos: Default::default(),
            config,
            config_load: None,
            config_load_revision: 0,
            config_watch: None,
            configured_terminal_size,
            copy_mode: None,
            core_terminals: Default::default(),
            daemon_text: Default::default(),
            device_drag: None,
            device_filter: Default::default(),
            device_order: Vec::new(),
            device_tag_filter: None,
            device_tag_folds: Default::default(),
            device_tags,
            device_themes: Default::default(),
            devices_overview: Default::default(),
            endpoints,
            file_transfer: None,
            find: None,
            find_memory: Default::default(),
            flash: None,
            flash_motion: Default::default(),
            focus,
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
            last_input: None,
            last_pointer: None,
            last_queued_options: None,
            links: Default::default(),
            listening_ports: Default::default(),
            live: LiveState::default(),
            local_error: error,
            mark_layer,
            marked: String::new(),
            marked_selection: None,
            menu,
            #[cfg(feature = "integration-test")]
            navigations: Default::default(),
            network_quality_seen: 0,
            notice_overflow: Default::default(),
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
            sound: Default::default(),
            split_cursor: None,
            split_drag: None,
            status_bar_visible,
            strip_navigation_fits: true,
            surface_signal,
            tab_drag: None,
            terminal_mouse: None,
            theme,
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
                endpoint.live = this.live.clone();
                if let Ok(mut inbox) = endpoint.connection.inbox.lock() {
                    *inbox = this.live.clone();
                }
            }
            return this;
        }
        Self::poll_on_frame(cx.entity().downgrade(), window);
        this.sidebar_preferences = if let Some(key) = core_key.as_ref() {
            Some(preferences::Preferences::for_core(key.hub_core_id()))
        } else {
            this.endpoints
                .first()
                .and_then(|endpoint| endpoint.connection.target.socket_path().ok())
                .map(|path| preferences::Preferences::new(&path))
        };
        if core_key.is_none() {
            this.reconnect(cx);
        }
        log_window::set_appearance(&this.config, &this.theme, cx);
        this.load_gui_config(cx);
        if core_key.is_none() {
            this.load_shared_settings(cx);
        }
        this.watch_gui_config(cx);
        this
    }
}
