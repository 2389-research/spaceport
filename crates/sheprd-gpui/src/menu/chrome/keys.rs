//! Keyboard input while a menu page holds focus: the Edit menu's shortcuts
//! and the overlay's key handler, which routes each key to the page that owns
//! it and leaves text editing and composition to the native input handler.

use crate::{
    SheprdWindow, actions,
    menu::{Page, WorkspaceAction},
};
use gpui::*;

impl SheprdWindow {
    /// An Edit menu item while a menu page holds focus. Only the targets the
    /// overlay's key handler gives these shortcuts to are reached: a dialog's
    /// text draft, and the GitHub page's device code for Copy. Search fields
    /// take the action themselves before it bubbles here. Handlers that act
    /// without stopping propagation are never called, so a shortcut the
    /// overlay leaves unhandled cannot run twice through the menu bar.
    pub(super) fn menu_edit(&mut self, key: &str, window: &mut Window, cx: &mut Context<Self>) {
        let event = actions::edit_key(key);
        if self.menu.page == Some(Page::Dialog(WorkspaceAction::OpenWorktree)) {
            return;
        }
        if let Some(input) = self.menu.input.as_mut() {
            if input.key(&event.keystroke, cx) {
                cx.notify();
            }
            return;
        }
        if self.menu.page == Some(Page::GitHub) {
            self.github_key(&event, window, cx);
        }
    }

    pub(super) fn menu_key_down(
        &mut self,
        event: &KeyDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.shortcut_editor_key(event, window, cx) {
            return;
        }
        if self.menu.page == Some(Page::Dialog(WorkspaceAction::OpenWorktree))
            && (self
                .menu
                .worktree_open
                .as_ref()
                .is_some_and(|picker| picker.search.read(cx).is_composing())
                || !matches!(
                    event.keystroke.key.as_str(),
                    "escape" | "enter" | "up" | "down"
                ))
        {
            // SearchInput and the platform own text editing and composition.
            return;
        }
        if let Some(input) = self.menu.input.as_mut() {
            if input.key(&event.keystroke, cx) {
                cx.stop_propagation();
                window.prevent_default();
                cx.notify();
                return;
            }
            // Let the platform deliver printable text and IME navigation/commit.
            if input.marked.is_some() || !matches!(event.keystroke.key.as_str(), "escape" | "enter")
            {
                return;
            }
        }
        if self.menu.page == Some(Page::DeviceTheme) {
            self.device_theme_key(event, window, cx);
            return;
        }
        if self.menu.page == Some(Page::DeviceTags) {
            self.device_tags_key(event, window, cx);
            return;
        }
        if matches!(
            self.menu.page,
            Some(Page::Host | Page::RenameDevice | Page::ForwardPort | Page::RemoveDevice)
        ) {
            self.host_menu_key(event, window, cx);
            return;
        }
        if matches!(
            self.menu.page,
            Some(Page::Tab | Page::RenameTab | Page::MoveTab)
        ) {
            self.tab_menu_key(event, window, cx);
            return;
        }
        if self.menu.page == Some(Page::Group) {
            self.group_menu_key(event, window, cx);
            return;
        }
        if matches!(
            self.menu.page,
            Some(
                Page::Pane
                    | Page::RenamePane
                    | Page::MovePane
                    | Page::PaneProcesses
                    | Page::KillProcesses
            )
        ) {
            self.pane_menu_key(event, window, cx);
            return;
        }
        if let Some(Page::Usage(provider)) = self.menu.page
            && self.usage_key(provider, event, cx)
        {
            cx.stop_propagation();
            window.prevent_default();
            return;
        }
        if self
            .menu
            .page
            .and_then(crate::menu::devices::rail::Source::of)
            .is_some()
            && self.add_device_rail_key(event, window, cx)
        {
            cx.stop_propagation();
            window.prevent_default();
            return;
        }
        if self.menu.page == Some(Page::AddWsl) {
            self.add_wsl_key(event, window, cx);
            return;
        }
        if self.menu.page == Some(Page::RemoveWsl) {
            self.remove_wsl_key(event, window, cx);
            return;
        }
        if self.menu.page == Some(Page::Devices) && self.menu.session_edit.is_some() {
            self.sessions_key(event, window, cx);
            return;
        }
        if matches!(self.menu.page, Some(Page::Devices | Page::AddDevice)) {
            self.devices_key(event, window, cx);
            return;
        }
        if self.menu.page == Some(Page::Palette) {
            self.palette_key(event, window, cx);
            return;
        }
        if self.menu.page == Some(Page::ConfirmClose) {
            self.close_confirmation_key(event, window, cx);
            return;
        }
        if self.menu.page == Some(Page::GitHub) {
            self.github_key(event, window, cx);
            return;
        }
        if self.menu.page == Some(Page::Themes) {
            self.theme_picker_key(event, window, cx);
            return;
        }
        if self.menu.page == Some(Page::Keybinds)
            && (self
                .menu
                .keybinds_search
                .as_ref()
                .is_some_and(|search| search.read(cx).is_composing())
                || !matches!(
                    event.keystroke.key.as_str(),
                    "escape" | "up" | "down" | "pageup" | "pagedown"
                ))
        {
            // Printable input and IME commands must reach the native text handler.
            return;
        }
        cx.stop_propagation();
        window.prevent_default();
        match event.keystroke.key.as_str() {
            "escape" => self.dismiss_menu(window, cx),
            "up" | "down"
                if self.menu.page == Some(Page::Dialog(WorkspaceAction::OpenWorktree)) =>
            {
                if let Some(picker) = &mut self.menu.worktree_open
                    && !picker.filtered.is_empty()
                    && self.menu.opening.is_none()
                {
                    let count = picker.filtered.len();
                    picker.selected = if event.keystroke.key == "up" {
                        (picker.selected + count - 1) % count
                    } else {
                        (picker.selected + 1) % count
                    };
                    picker
                        .scroll
                        .scroll_to_item(picker.selected, ScrollStrategy::Top);
                    cx.notify();
                }
            }
            // A held key can outlive a dirty-worktree refusal and the switch
            // to Force remove. Destructive consent requires a fresh press.
            "enter"
                if event.is_held
                    && (matches!(
                        self.menu.page,
                        Some(Page::Dialog(
                            WorkspaceAction::Close | WorkspaceAction::DeleteWorktree
                        ))
                    ) || self.menu.page == Some(Page::CoreWorkspaceDelete)
                        || self.core_workspace_delete_selected()) => {}
            "enter" if matches!(self.menu.page, Some(Page::Dialog(_))) => {
                self.submit_workspace_dialog(window, cx)
            }
            "up" | "down" if self.menu.page == Some(Page::CoreWorkspaceActions) => {
                self.cycle_core_workspace_action(event.keystroke.key == "down", cx)
            }
            "enter" if self.menu.page == Some(Page::CoreWorkspaceActions) => {
                self.activate_selected_core_workspace_action(window, cx)
            }
            "enter"
                if matches!(
                    self.menu.page,
                    Some(
                        Page::CoreWorkspaceCreate
                            | Page::CoreWorkspaceDelete
                            | Page::CoreWorkspaceRename
                    )
                ) =>
            {
                self.submit_core_workspace_dialog(cx)
            }
            // The tiles read row by row, so left and right walk the same
            // order as up and down.
            "up" | "down" | "left" | "right" if self.menu.page == Some(Page::Workspace) => {
                let actions = self.workspace_menu_actions();
                let selected = self
                    .menu
                    .workspace_selected
                    .and_then(|selected| actions.iter().position(|action| *action == selected));
                let back = matches!(event.keystroke.key.as_str(), "up" | "left");
                if !actions.is_empty() {
                    let index = match (selected, back) {
                        (None, true) => actions.len() - 1,
                        (None, false) => 0,
                        (Some(index), true) => (index + actions.len() - 1) % actions.len(),
                        (Some(index), false) => (index + 1) % actions.len(),
                    };
                    self.menu.workspace_selected = Some(actions[index]);
                }
                cx.notify();
            }
            "up" | "down" if self.menu.page == Some(Page::Menu) => {
                let count = self.menu_items().len();
                if count > 0 {
                    self.menu.selected =
                        Some(match (self.menu.selected, event.keystroke.key.as_str()) {
                            (None, "up") => count - 1,
                            (None, _) => 0,
                            (Some(index), "up") => (index + count - 1) % count,
                            (Some(index), _) => (index + 1) % count,
                        });
                }
                cx.notify();
            }
            "enter" if self.menu.page == Some(Page::Workspace) => {
                if let Some(action) = self
                    .menu
                    .workspace_selected
                    .filter(|action| self.workspace_menu_actions().contains(action))
                {
                    self.activate_workspace_menu(action, window, cx);
                }
            }
            "up" | "down" | "pageup" | "pagedown" if self.menu.page == Some(Page::Keybinds) => {
                let scroll = &self.menu.keybinds_scroll;
                let key = event.keystroke.key.as_str();
                let distance = if key.starts_with("page") {
                    scroll.bounds().size.height * 0.8
                } else {
                    px(self.config.ui.line_height() * 3.)
                };
                let direction = if key.ends_with("up") { 1. } else { -1. };
                scroll.set_offset(scroll.offset() + point(px(0.), distance * direction));
                cx.notify();
            }
            "enter" if self.menu.page == Some(Page::AgentSkill) => {
                self.install_browser_skill(window, cx);
            }
            "enter" if self.menu.page == Some(Page::OpenLink) => {
                self.confirm_open_link(window, cx);
            }
            "enter" if self.menu.page == Some(Page::OpenCheckout) => {
                self.confirm_open_checkout(window, cx);
            }
            "enter" if self.menu.page == Some(Page::VersionMismatch) => {
                self.act_on_version_mismatch(window, cx);
            }
            "enter" if self.menu.page == Some(Page::About) => {
                self.dismiss_menu(window, cx);
            }
            "enter" if self.menu.page == Some(Page::Menu) => {
                if let Some(item) = self
                    .menu
                    .selected
                    .and_then(|index| self.menu_items().get(index).copied())
                {
                    self.activate_menu(item, window, cx);
                }
            }
            _ => {}
        }
    }
}
