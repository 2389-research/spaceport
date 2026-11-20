// ABOUTME: Runs window command handlers with explicit completion and fallback outcomes.
// ABOUTME: Preserves confirmation, endpoint admission, focus, and local feedback ownership.
//! What each row of the command table does: the window's side of a command
//! that needs no daemon request, and the request the rest fall through to.

use super::SheprdWindow;
use super::dispatch::Run;
use crate::{
    controls::{self, Command},
    i18n::fl,
    log_window,
    menu::{WorkspaceAction, WorktreeShortcut},
    navigation::NavigationTarget,
    open_additional_window,
};
use gpui::{Context, Window};

impl SheprdWindow {
    pub(super) fn command_close_workspace(
        &mut self,
        _command: Command,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Run {
        let opened = self
            .live
            .snapshot
            .as_ref()
            .and_then(|snapshot| snapshot.focused_workspace_id.clone())
            .is_some_and(|id| self.open_workspace_close(&id, window, cx));
        if !opened {
            self.show_flash(
                super::Flash::warning(fl!("close-modal-workspace-unavailable")),
                cx,
            );
        }
        Run::Done
    }

    pub(super) fn command_messages(
        &mut self,
        _command: Command,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Run {
        self.open_messages(window, cx);
        Run::Done
    }

    /// The command a notice's toast names.
    pub(super) fn command_notification_target(
        &mut self,
        _command: Command,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Run {
        if let Some((endpoint, id)) = self.endpoints.iter().find_map(|e| {
            e.toasts
                .entries
                .iter()
                .find(|(_, n)| n.is_visible())
                .map(|(id, _)| (e, *id))
        }) {
            let source = endpoint.connection_ref();
            self.click_toast(&source, id, cx);
        }
        Run::Done
    }

    pub(super) fn command_logs(
        &mut self,
        _command: Command,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Run {
        log_window::open(cx);
        Run::Done
    }

    pub(super) fn command_new_window(
        &mut self,
        _command: Command,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Run {
        if let Some(key) = self.selected_core_group() {
            crate::app::open_additional_core_window(key.clone(), cx);
        } else if let Some(endpoint) = self.endpoints.first() {
            open_additional_window(endpoint.connection.target.clone(), cx);
        }
        Run::Done
    }

    /// A page has no panes, so either close closes its tab, as in a web
    /// browser; nothing runs in it that needs confirming. A group standing in
    /// for a tab closes that tab, and an empty group of a split closes, as an
    /// editor group does.
    pub(super) fn command_close(
        &mut self,
        command: Command,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Run {
        let Some(group) = self.active_group() else {
            self.open_close_confirmation(command, window, cx);
            return Run::Done;
        };
        match self.group_shown(group, cx) {
            crate::browser::Shown::Terminal => self.open_close_confirmation(command, window, cx),
            crate::browser::Shown::Page(_) | crate::browser::Shown::Elsewhere(_) => {
                self.close_group_tab(group, window, cx)
            }
            crate::browser::Shown::Empty => self.close_group(group, window, cx),
        }
        Run::Done
    }

    pub(super) fn command_step_tab(
        &mut self,
        command: Command,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Run {
        match self.step_tab(command, window, cx) {
            true => Run::Done,
            false => Run::FallsThrough,
        }
    }

    pub(super) fn command_reopen_closed_tab(
        &mut self,
        _command: Command,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Run {
        self.reopen_closed_tab(window, cx);
        Run::Done
    }

    pub(super) fn command_split_editor(
        &mut self,
        _command: Command,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Run {
        self.split_active_group(window, cx);
        Run::Done
    }

    pub(super) fn command_install_browser_skill(
        &mut self,
        _command: Command,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Run {
        self.install_browser_skill(window, cx);
        Run::Done
    }

    pub(super) fn command_palette(
        &mut self,
        command: Command,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Run {
        self.open_palette(
            if command == Command::WorkspacePicker {
                crate::palette::Filter::Navigation
            } else {
                crate::palette::Filter::All
            },
            window,
            cx,
        );
        Run::Done
    }

    pub(super) fn command_resume_palette(
        &mut self,
        _command: Command,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Run {
        self.resume_palette(window, cx);
        Run::Done
    }

    pub(super) fn command_open_worktree(
        &mut self,
        _command: Command,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Run {
        self.open_worktree_shortcut(WorktreeShortcut::Open, window, cx);
        Run::Done
    }

    pub(super) fn command_worktree_notes(
        &mut self,
        _command: Command,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Run {
        self.open_palette(crate::palette::Filter::Notes, window, cx);
        Run::Done
    }

    pub(super) fn command_edit_worktree_note(
        &mut self,
        _command: Command,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Run {
        self.edit_focused_worktree_note(window, cx);
        Run::Done
    }

    /// Herdr's own name prompt, which every kind of tab or workspace makes
    /// through. With Herdr 2 chosen for new terminals, a new tab on this
    /// computer is herdr-core's.
    pub(super) fn command_name_prompt(
        &mut self,
        command: Command,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Run {
        if command == Command::Tab && self.new_tabs_use_core() {
            self.new_core_terminal(window, cx);
            return Run::Done;
        }
        match self.open_name_prompt(command, window, cx) {
            true => Run::Done,
            false => Run::FallsThrough,
        }
    }

    pub(super) fn command_keybinds(
        &mut self,
        _command: Command,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Run {
        self.open_keybinds(window, cx);
        Run::Done
    }

    pub(super) fn command_sessions(
        &mut self,
        _command: Command,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Run {
        // The button's own position, so the shortcut opens the list where
        // clicking the button does.
        self.open_sessions(self.sessions_anchor.get(), window, cx);
        Run::Done
    }

    pub(super) fn command_themes(
        &mut self,
        _command: Command,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Run {
        self.open_theme_picker(window, cx);
        Run::Done
    }

    pub(super) fn command_settings(
        &mut self,
        _command: Command,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Run {
        self.open_preferences(window, cx);
        Run::Done
    }

    pub(super) fn command_check_for_updates(
        &mut self,
        _command: Command,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Run {
        if let Err(error) = crate::updates::check(cx) {
            self.notices
                .show(crate::notice::Cause::Command, error.localize());
            cx.notify();
        }
        Run::Done
    }

    pub(super) fn command_about(
        &mut self,
        _command: Command,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Run {
        self.open_about(window, cx);
        Run::Done
    }

    pub(super) fn command_find(
        &mut self,
        _command: Command,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Run {
        self.open_find(window, cx);
        Run::Done
    }

    pub(super) fn command_find_step(
        &mut self,
        command: Command,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Run {
        let back = command == Command::FindPrevious;
        if self.find.is_some() {
            // The bar's next match is the next older one, as Enter.
            let step = if back {
                crate::find::Step::Newer
            } else {
                crate::find::Step::Older
            };
            self.find_step(step, cx);
        } else {
            self.open_find(window, cx);
        }
        Run::Done
    }

    pub(super) fn command_window(
        &mut self,
        command: Command,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Run {
        super::run_window_command(command, window, cx);
        Run::Done
    }

    pub(super) fn command_copy_mode(
        &mut self,
        _command: Command,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Run {
        self.enter_copy_mode(window, cx);
        Run::Done
    }

    pub(super) fn command_edit_scrollback(
        &mut self,
        _command: Command,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Run {
        if self.live.supports_edit_scrollback {
            return Run::FallsThrough;
        }
        self.show_flash(
            super::Flash::warning(fl!("window-notices-scrollback-needs-newer")),
            cx,
        );
        Run::Done
    }

    pub(super) fn command_toggle_sidebar(
        &mut self,
        _command: Command,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Run {
        self.toggle_sidebar();
        cx.notify();
        Run::Done
    }

    pub(super) fn command_focus_next_region(
        &mut self,
        _command: Command,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Run {
        self.cycle_region(true, window, cx);
        Run::Done
    }

    pub(super) fn command_focus_previous_region(
        &mut self,
        _command: Command,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Run {
        self.cycle_region(false, window, cx);
        Run::Done
    }

    pub(super) fn command_toggle_status_bar(
        &mut self,
        _command: Command,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Run {
        self.status_bar_visible = !self.status_bar_visible;
        window.focus(&self.focus, cx);
        cx.notify();
        Run::Done
    }

    pub(super) fn command_toggle_zen(
        &mut self,
        _command: Command,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Run {
        self.toggle_zen(window, cx);
        Run::Done
    }

    pub(super) fn command_devices_overview(
        &mut self,
        _command: Command,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Run {
        self.open_devices_overview(window, cx);
        Run::Done
    }

    pub(super) fn command_new_core_terminal(
        &mut self,
        _command: Command,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Run {
        self.new_core_terminal(window, cx);
        Run::Done
    }

    pub(super) fn command_mirror_core_pane(
        &mut self,
        _command: Command,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Run {
        self.show_flash(
            super::Flash::warning(fl!("core-terminals-mirror-needs-pane")),
            cx,
        );
        Run::Done
    }

    pub(super) fn command_step_font_size(
        &mut self,
        command: Command,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Run {
        let step = if command == Command::IncreaseFontSize {
            crate::config::FONT_SIZE_STEP
        } else {
            -crate::config::FONT_SIZE_STEP
        };
        self.set_terminal_font_size(self.config.terminal.size + step, cx);
        Run::Done
    }

    pub(super) fn command_reset_font_size(
        &mut self,
        _command: Command,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Run {
        self.set_terminal_font_size(self.configured_terminal_size, cx);
        Run::Done
    }

    /// Older daemons do not advertise `pane.clear`. Say so rather than typing
    /// `clear` into the pane, which could reach a running program.
    pub(super) fn command_clear_pane(
        &mut self,
        _command: Command,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Run {
        if self.live.supports_pane_clear {
            return Run::FallsThrough;
        }
        self.notices.show(
            crate::notice::Cause::Command,
            fl!("window-notices-clear-needs-newer"),
        );
        cx.notify();
        Run::Done
    }

    pub(super) fn command_detach(
        &mut self,
        _command: Command,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Run {
        self.detach_endpoint(cx);
        cx.notify();
        Run::Done
    }

    pub(super) fn command_reconnect(
        &mut self,
        _command: Command,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Run {
        self.reconnect(cx);
        Run::FallsThrough
    }

    pub(super) fn command_quit(
        &mut self,
        _command: Command,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Run {
        cx.quit();
        Run::Done
    }

    pub(super) fn command_rename_tab(
        &mut self,
        _command: Command,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Run {
        self.rename_focused_tab(window, cx);
        Run::Done
    }

    pub(super) fn command_rename_pane(
        &mut self,
        _command: Command,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Run {
        self.rename_focused_pane(window, cx);
        Run::Done
    }

    pub(super) fn command_move_pane(
        &mut self,
        _command: Command,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Run {
        self.move_focused_pane(window, cx);
        Run::Done
    }

    pub(super) fn command_rename_workspace(
        &mut self,
        _command: Command,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Run {
        self.open_focused_workspace_dialog(WorkspaceAction::Rename, window, cx);
        Run::Done
    }

    pub(super) fn command_reload_config(
        &mut self,
        _command: Command,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Run {
        self.reload_daemon_config();
        self.load_gui_config(cx);
        cx.notify();
        Run::Done
    }

    pub(super) fn command_equalize_splits(
        &mut self,
        _command: Command,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Run {
        self.equalize_splits(cx);
        Run::Done
    }

    pub(super) fn command_resize_mode(
        &mut self,
        _command: Command,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Run {
        self.prefix_armed = false;
        self.resize_mode = true;
        cx.notify();
        Run::Done
    }

    pub(super) fn command_last_pane(
        &mut self,
        _command: Command,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Run {
        // Herdr's check: the pane must still exist and not be focused.
        if let Some(snapshot) = &self.live.snapshot
            && let Some(pane) = self.live.previous_pane.clone().filter(|pane| {
                snapshot.focused_pane_id.as_ref() != Some(pane)
                    && snapshot.panes.iter().any(|p| p.pane_id == *pane)
            })
        {
            self.navigate(NavigationTarget::Pane(&pane), cx);
        }
        window.focus(&self.focus, cx);
        Run::Done
    }

    pub(super) fn command_travel(
        &mut self,
        command: Command,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Run {
        let step = match command {
            Command::Back => super::Step::Back,
            _ => super::Step::Forward,
        };
        self.travel(step, window, cx);
        if self.activity_section().is_none() {
            window.focus(&self.focus, cx);
        }
        Run::Done
    }

    pub(super) fn command_step_sidebar(
        &mut self,
        command: Command,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Run {
        self.step_sidebar(command, cx);
        window.focus(&self.focus, cx);
        Run::Done
    }

    /// A browser tab moves without the daemon, so before its check; a daemon
    /// too old to move tabs says so, and the request is all that is left.
    pub(super) fn command_move_tab(
        &mut self,
        command: Command,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Run {
        if self.move_page_tab(command, cx) {
            return Run::Done;
        }
        if self.live.supports_tab_move {
            return Run::FallsThrough;
        }
        self.notices.show(
            crate::notice::Cause::Command,
            fl!("window-notices-move-tabs-needs-newer"),
        );
        cx.notify();
        Run::Done
    }

    /// The daemon request a command still needs, when the window has a
    /// surface to make it against.
    pub(super) fn request_daemon_command(
        &mut self,
        command: Command,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(endpoint) = self.endpoints.get(self.selected_endpoint) else {
            self.show_flash(
                super::Flash::warning(fl!("window-notices-needs-herdr-tui")),
                cx,
            );
            return;
        };
        if self.live.activation.fenced() || !endpoint.surface_requested() {
            return;
        }
        if let Some(snapshot) = &self.live.snapshot
            && let Some((method, params)) = controls::request(command, snapshot)
        {
            self.request_focus_change(method.as_str(), None, |handle, boot| {
                handle.request(boot, method, params)
            });
            self.discard_composition(cx);
        }
        window.focus(&self.focus, cx);
        cx.notify();
    }
}
