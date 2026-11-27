// ABOUTME: Dispatches window commands and isolates focus changes from stale terminal input.
// ABOUTME: Enforces the running app's Core opt-in before Core-only commands reach panes.
//! Command dispatch and focus changes. A focus change is fenced behind an
//! ordered surface barrier so input cannot reach the previous pane while the
//! navigation and its projection are still in flight.
// i18n: migrated

use super::{Flash, SpaceportWindow, layout_save::LayoutSaves};
use crate::{
    app::InitialAppearance,
    config::{
        ChatLayoutMode, Config, FONT_SIZE_RANGE, LayoutMode, TabLayoutMode, preferences::Preference,
    },
    controls::Command,
    endpoint::Step,
    i18n::{Localize, fl},
    log_window,
    navigation::{NavigationTarget, OwnedNavigationTarget},
    state,
};
use dispatch::Run;
use gpui::{Context, Window};

mod dispatch;
mod handlers;

/// Keeps a chat layout choice, as the chat view switches it.
fn save_chat_layout(mode: ChatLayoutMode) -> crate::Result<()> {
    Config::save_preference(Preference::ChatLayout(mode)).map_err(crate::Error::from)
}

impl SpaceportWindow {
    /// Switches the sidebar layout here at once, then keeps it: the choice is
    /// saved to the local overrides off the UI thread, and the config watcher
    /// brings every other window along.
    pub(crate) fn set_layout(&mut self, mode: LayoutMode, cx: &mut Context<Self>) {
        self.set_layout_with(
            mode,
            |mode| Config::save_layout(mode).map_err(crate::Error::from),
            cx,
        );
    }

    /// `save` persists the choice; tests pass one that leaves the real
    /// config alone.
    pub(crate) fn set_layout_with(
        &mut self,
        mode: LayoutMode,
        save: impl FnOnce(LayoutMode) -> crate::Result<()> + Send + 'static,
        cx: &mut Context<Self>,
    ) {
        if self.config.layout.mode == mode {
            return;
        }
        self.config.layout.mode = mode;
        // The menu reads its checkmark from the latest config.
        if cx.has_global::<InitialAppearance>() {
            cx.global_mut::<InitialAppearance>()
                .shown
                .config
                .layout
                .mode = mode;
        }
        crate::menus::install(cx);
        cx.notify();
        let saves = cx.default_global::<LayoutSaves>().sidebar.clone();
        saves.want(mode);
        let save = cx
            .background_executor()
            .spawn(async move { saves.write_latest(save) });
        cx.spawn(async move |_, _| {
            if let Err(error) = save.await {
                tracing::warn!(%error, "Could not save the sidebar layout");
            }
        })
        .detach();
    }

    /// Switches how tab strips draw their tabs, then keeps it as
    /// [`Self::set_layout`] keeps the sidebar's.
    pub(crate) fn set_tab_layout(&mut self, mode: TabLayoutMode, cx: &mut Context<Self>) {
        self.set_tab_layout_with(
            mode,
            |mode| Config::save_preference(Preference::TabLayout(mode)).map_err(crate::Error::from),
            cx,
        );
    }

    /// `save` persists the choice; tests pass one that leaves the real
    /// config alone.
    pub(crate) fn set_tab_layout_with(
        &mut self,
        mode: TabLayoutMode,
        save: impl FnOnce(TabLayoutMode) -> crate::Result<()> + Send + 'static,
        cx: &mut Context<Self>,
    ) {
        if self.config.tab_layout == mode {
            return;
        }
        self.config.tab_layout = mode;
        if cx.has_global::<InitialAppearance>() {
            cx.global_mut::<InitialAppearance>().shown.config.tab_layout = mode;
        }
        crate::menus::install(cx);
        cx.notify();
        let saves = cx.default_global::<LayoutSaves>().tabs.clone();
        saves.want(mode);
        let save = cx
            .background_executor()
            .spawn(async move { saves.write_latest(save) });
        cx.spawn(async move |_, _| {
            if let Err(error) = save.await {
                tracing::warn!(%error, "Could not save the tab layout");
            }
        })
        .detach();
    }

    /// Switches how chat views draw, then keeps it as [`Self::set_layout`]
    /// keeps the sidebar's.
    pub(crate) fn set_chat_layout(&mut self, mode: ChatLayoutMode, cx: &mut Context<Self>) {
        self.set_chat_layout_with(mode, save_chat_layout, cx);
    }

    /// The layout after the one showing now, not the one the last frame
    /// drew: a second click can land before the next frame.
    pub(crate) fn step_chat_layout(&mut self, cx: &mut Context<Self>) {
        self.step_chat_layout_with(save_chat_layout, cx);
    }

    /// `save` persists the choice, as in [`Self::set_chat_layout_with`].
    pub(crate) fn step_chat_layout_with(
        &mut self,
        save: impl FnOnce(ChatLayoutMode) -> crate::Result<()> + Send + 'static,
        cx: &mut Context<Self>,
    ) {
        self.set_chat_layout_with(self.config.chat.layout.next(), save, cx);
    }

    /// `save` persists the choice; tests pass one that leaves the real
    /// config alone.
    pub(crate) fn set_chat_layout_with(
        &mut self,
        mode: ChatLayoutMode,
        save: impl FnOnce(ChatLayoutMode) -> crate::Result<()> + Send + 'static,
        cx: &mut Context<Self>,
    ) {
        if self.config.chat.layout == mode {
            return;
        }
        self.config.chat.layout = mode;
        if cx.has_global::<InitialAppearance>() {
            cx.global_mut::<InitialAppearance>()
                .shown
                .config
                .chat
                .layout = mode;
        }
        self.chats.relayout(mode);
        crate::menus::install(cx);
        cx.notify();
        let saves = cx.default_global::<LayoutSaves>().chat.clone();
        saves.want(mode);
        let save = cx
            .background_executor()
            .spawn(async move { saves.write_latest(save) });
        cx.spawn(async move |_, _| {
            if let Err(error) = save.await {
                tracing::warn!(%error, "Could not save the chat layout");
            }
        })
        .detach();
    }

    #[track_caller]
    pub(crate) fn navigate(
        &mut self,
        target: NavigationTarget<&str>,
        cx: &mut Context<Self>,
    ) -> bool {
        if self.menu.page.is_some() {
            return false;
        }
        let navigated = self.dispatch_navigation(target, cx);
        if navigated {
            self.leave_core_group(cx);
        }
        navigated
    }

    /// Back or Forward along the selected endpoint's focus trail. The trail
    /// remembers the step only once its request is queued, so a press while
    /// an earlier one is still landing leaves that one in charge.
    pub(crate) fn travel(&mut self, step: Step, window: &mut Window, cx: &mut Context<Self>) {
        // A pane a section opened goes back to that section first.
        if step == Step::Back
            && self.activity_section().is_none()
            && let Some(panel) = self.activity.return_to
        {
            self.show_activity_panel(panel, window, cx);
            return;
        }
        let Some(snapshot) = self.live.snapshot.clone() else {
            return;
        };
        let Some((index, pane)) = self.endpoints[self.selected_endpoint]
            .history
            .peek(step, &snapshot)
            .map(|(index, pane)| (index, pane.to_owned()))
        else {
            return;
        };
        if self.navigate(NavigationTarget::Pane(&pane), cx) {
            self.endpoints[self.selected_endpoint].history.begin(index);
        }
    }

    pub(crate) fn can_travel(&self, step: Step) -> bool {
        if step == Step::Back
            && self.activity_section().is_none()
            && self.activity.return_to.is_some()
        {
            return true;
        }
        self.live.snapshot.as_deref().is_some_and(|snapshot| {
            self.endpoints[self.selected_endpoint]
                .history
                .can(step, snapshot)
        })
    }

    /// Complete an accepted navigation even if its context menu has since opened.
    #[track_caller]
    pub(crate) fn dispatch_navigation(
        &mut self,
        target: NavigationTarget<&str>,
        cx: &mut Context<Self>,
    ) -> bool {
        if !self.input_ready() {
            return false;
        }
        let queued = self.request_focus_change(
            &fl!("window-notices-navigate"),
            Some((&target).into()),
            |handle, boot| match target {
                NavigationTarget::Workspace(id) => handle.focus_workspace(boot, id),
                NavigationTarget::Tab(id) => handle.focus_tab(boot, id),
                NavigationTarget::Pane(id) => handle.focus_pane(boot, id),
            },
        );
        self.discard_composition(cx);
        cx.notify();
        queued
    }

    /// `label` is what a failure is reported as, not a method name: navigation
    /// picks its method from the target, so it reports itself by name.
    #[track_caller]
    pub(crate) fn request_focus_change(
        &mut self,
        label: &str,
        focus: Option<OwnedNavigationTarget>,
        enqueue: impl FnOnce(
            &spaceport_client::ClientHandle,
            &str,
        ) -> Result<String, spaceport_client::SendError>,
    ) -> bool {
        if let (Some(handle), Some(snapshot)) = (
            &self.endpoints[self.selected_endpoint].connection.handle,
            &self.live.snapshot,
        ) {
            if let Err(error) = enqueue(handle, &snapshot.boot_id) {
                self.notices.show(
                    crate::notice::Cause::Command,
                    fl!(
                        "window-notices-request-failed",
                        operation = label,
                        error = error.localize()
                    ),
                );
            } else {
                #[cfg(feature = "integration-test")]
                self.navigations
                    .record(label, focus.as_ref(), std::panic::Location::caller());
                self.fence_focus_change(focus);
                return true;
            }
        }
        false
    }

    pub(crate) fn fence_focus_change(&mut self, focus: Option<OwnedNavigationTarget>) {
        if !self.live.supports_surface {
            return;
        }
        if let (Some(handle), Some(snapshot)) = (
            &self.endpoints[self.selected_endpoint].connection.handle,
            &self.live.snapshot,
        ) && let Ok(mut state) = self.endpoints[self.selected_endpoint]
            .connection
            .inbox
            .lock()
        {
            // An ordered surface barrier prevents input hitting the previous
            // pane while navigation/creation and its projection are in flight.
            // Register the barrier under the inbox lock before input can resume.
            let result = handle.set_surface_active(&snapshot.boot_id, true);
            if let Err(error) = &result
                && !matches!(error, spaceport_client::Error::Full)
            {
                self.notices
                    .show(crate::notice::Cause::Command, error.localize());
            }
            state
                .activation
                .transition(state::activation::Transition::Queued {
                    result,
                    boot: snapshot.boot_id.clone(),
                    focus,
                    active: true,
                    now: std::time::Instant::now(),
                });
            state.invalidate_surface();
            state.dirty = true;
            self.live = state.clone();
        }
    }

    /// Applies a session terminal size. Painting, hit testing, and IME
    /// placement all read `config.terminal.size` and its derived line height,
    /// so writing that one field keeps the three in agreement; render
    /// re-measures the cell and the canvas resends the geometry.
    ///
    /// A pending config load is deliberately left alone. Cancelling it the way
    /// the theme picker does would strand the very first load, which has no
    /// retry, on default fonts; a landing reload merely discards the
    /// adjustment, which is what reloading is for.
    pub(crate) fn set_terminal_font_size(&mut self, size: f32, cx: &mut Context<Self>) {
        let size = size.clamp(*FONT_SIZE_RANGE.start(), *FONT_SIZE_RANGE.end());
        if size == self.config.terminal.size {
            return;
        }
        self.config.terminal.size = size;
        // Font changes can alter wrapped row heights without changing the
        // reading column's width. Keep the reader's scroll position.
        for pane in self.chats.panes() {
            pane.list.remeasure();
        }
        // The console follows the rendered terminal face, as a reload makes it.
        log_window::set_appearance(&self.config, &self.theme, cx);
        cx.notify();
    }

    pub(crate) fn command(
        &mut self,
        command: Command,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.menu.page.is_some() {
            return;
        }
        if !self.core_enabled() && crate::core_feature::command_requires_core(command) {
            self.show_flash(Flash::warning(crate::core_feature::disabled_message()), cx);
            return;
        }
        #[cfg(feature = "integration-test")]
        {
            self.input_probe.actions += 1;
        }
        if self.core_pane_command(command, window, cx) {
            return;
        }
        match self.dispatch(command, window, cx) {
            Run::Done => {}
            Run::FallsThrough => self.request_daemon_command(command, window, cx),
        }
    }

    /// Focuses the workspace or agent a sidebar step lands on, selecting its
    /// host first when that is another one.
    fn step_sidebar(&mut self, command: Command, cx: &mut Context<Self>) {
        let agents = matches!(
            command,
            Command::PreviousAgent | Command::NextAgent | Command::AgentNumber(_)
        );
        let rows = if agents {
            self.sidebar_agents()
        } else {
            self.sidebar_workspaces()
        };
        let selected = self.selected_endpoint;
        let focused = self.live.snapshot.as_ref().and_then(|snapshot| {
            if agents {
                snapshot.focused_pane_id.as_deref()
            } else {
                snapshot.focused_workspace_id.as_deref()
            }
        });
        let current = rows
            .iter()
            .position(|&(host, id)| host == selected && Some(id) == focused);
        let Some(&(host, id)) = crate::sidebar::sidebar_step(&rows, current, selected, command)
            .and_then(|index| rows.get(index))
        else {
            return;
        };
        let id = id.to_owned();
        let target = if agents {
            NavigationTarget::Pane(id.as_str())
        } else {
            NavigationTarget::Workspace(id.as_str())
        };
        if host == selected {
            self.navigate(target, cx);
        } else {
            let endpoint = self.endpoints[host].id.clone();
            self.navigate_endpoint(&endpoint, target, cx);
        }
    }

    /// Installs the agent skill for browser tabs where Claude Code and other
    /// agents look for skills, and keeps it current from now on. Only on
    /// request: the one-time offer, the palette, or Settings.
    pub(crate) fn install_browser_skill(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        use crate::agent_skill::{AgentSkill, Choice};
        let home = match crate::config::home() {
            Ok(home) => home,
            Err(error) => {
                self.show_flash(Flash::warning(error.localize()), cx);
                return;
            }
        };
        AgentSkill::choose(Choice::Installed, cx);
        if self.menu.page == Some(crate::menu::Page::AgentSkill) {
            self.dismiss_menu(window, cx);
        }
        let install = cx.background_executor().spawn(async move {
            let text = crate::agent_skill::text(crate::agent_skill::executable().as_deref());
            crate::agent_skill::install(&home, &text)
        });
        cx.spawn(async move |this, cx| {
            let result = install.await;
            this.update(cx, |this, cx| {
                let flash = match result {
                    Ok(paths) => Flash::success(match paths.as_slice() {
                        [path] => fl!(
                            "window-notices-skill-installed-at",
                            path = path.display().to_string()
                        ),
                        _ => fl!("window-notices-skill-installed-places", count = paths.len()),
                    }),
                    Err(error) => Flash::warning(error.localize()),
                };
                this.show_flash(flash, cx);
            })
            .ok();
        })
        .detach();
    }

    /// Offers the agent skill once, when this is the first window ready to
    /// show a dialog. Runs every tick; asking claims the question app-wide.
    pub(crate) fn offer_browser_skill(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let ready = self.menu.page.is_none() && self.active && self.live.snapshot.is_some();
        if !ready || !crate::agent_skill::AgentSkill::take_ask(cx) {
            return;
        }
        if self.open_menu(window, cx) {
            self.menu.page = Some(crate::menu::Page::AgentSkill);
            cx.notify();
        }
    }

    pub(crate) fn render_agent_skill_offer(&self, cx: &mut Context<Self>) -> gpui::Div {
        use gpui::{prelude::*, *};
        let theme = &self.theme;
        let button = |id: &'static str, label: String, primary: bool| {
            div()
                .id(id)
                .debug_selector(move || id.into())
                .p(px(8.))
                .rounded(px(crate::config::corners::CONTROL))
                .when(primary, |button| button.bg(rgb(theme.active)))
                .when(!primary, |button| button.hover(|s| s.bg(rgb(theme.active))))
                .cursor_pointer()
                .child(label)
        };
        div()
            .debug_selector(|| "agent-skill-offer".into())
            .child(
                div()
                    .p(px(8.))
                    .child(fl!("window-notices-skill-offer-title")),
            )
            .child(
                div()
                    .p(px(8.))
                    .child(fl!("window-notices-skill-offer-body")),
            )
            .child(
                div()
                    .p(px(8.))
                    .text_color(rgb(theme.muted))
                    .child(fl!("window-notices-skill-offer-detail")),
            )
            .child(
                div()
                    .flex()
                    .flex_wrap()
                    .gap(px(8.))
                    .p(px(8.))
                    .child(
                        button(
                            "agent-skill-install",
                            fl!("window-notices-skill-offer-install"),
                            true,
                        )
                        .on_click(cx.listener(|this, _, window, cx| {
                            cx.stop_propagation();
                            this.install_browser_skill(window, cx);
                        })),
                    )
                    .child(
                        button(
                            "agent-skill-decline",
                            fl!("window-notices-skill-offer-decline"),
                            false,
                        )
                        .on_click(cx.listener(|this, _, window, cx| {
                            cx.stop_propagation();
                            this.dismiss_menu(window, cx);
                        })),
                    ),
            )
    }
}

/// Runs a command that acts on the window itself rather than its content,
/// for every kind of window, so the Settings and Log windows do not swallow
/// it; whether `command` was one.
pub(crate) fn run_window_command(
    command: Command,
    window: &mut Window,
    cx: &mut gpui::App,
) -> bool {
    match command {
        Command::ToggleFullScreen => window.toggle_fullscreen(),
        Command::CycleWindows => cycle_windows(window, cx),
        Command::CloseWindow => window.remove_window(),
        _ => return false,
    }
    true
}

/// Brings the app's next window forward, in the stable order the app lists
/// its windows, as macOS's Cmd-` does: a front-to-back order would only ever
/// swap the two most recent. Deferred, since activating a window updates it.
fn cycle_windows(window: &Window, cx: &mut gpui::App) {
    let Some(next) = next_window(&cx.windows(), window.window_handle()) else {
        return;
    };
    cx.defer(move |cx| {
        next.update(cx, |_, window, _| window.activate_window())
            .ok();
    });
}

/// The window after `current` in `windows`, wrapping; none when `current`
/// is alone or unknown.
pub(super) fn next_window<T: Copy + PartialEq>(windows: &[T], current: T) -> Option<T> {
    let at = windows.iter().position(|window| *window == current)?;
    let next = windows[(at + 1) % windows.len()];
    (next != current).then_some(next)
}
