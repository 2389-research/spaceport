// i18n: migrated
use crate::i18n::fl;
use crate::menu::labels;
use crate::{
    SpaceportWindow, close_modal::CloseConfirmation, menu::Page, pane_move::Destination,
    search_input::SearchInput,
};
use gpui::{prelude::*, *};
use spaceport_client::Method;
use spaceport_menu::form;
use spaceport_menu::pane::{Action, Offers, Target, render};
use spaceport_menu::rows::Row;

mod moves;
mod processes;

#[cfg(test)]
pub(crate) mod tests;

impl PaneMenu {
    /// The rows this menu offers, which the pane's own state and what this
    /// machine can do for it decide.
    fn actions(&self) -> Vec<Action> {
        self.target.actions(Offers {
            movable: self.movable,
            destinations: !self.choices.is_empty(),
            processes: self.daemon.is_some(),
        })
    }
}

pub(super) struct PaneMenu {
    target: Target,
    /// Existing tabs, and new tabs in other workspaces, the pane can move to.
    choices: Vec<crate::pane_move::Choice>,
    selected: Option<usize>,
    input: Option<Entity<SearchInput>>,
    pending: Option<String>,
    error: Option<String>,
    /// The daemon to ask for the pane's processes, when they run here.
    daemon: Option<crate::processes::Daemon>,
    processes: Option<processes::PaneProcesses>,
    /// Whether this endpoint's host can run `herdr pane move`.
    movable: bool,
    /// The move in flight, which keeps the menu from starting another.
    moving: Option<u64>,
    /// The move picker's panel scroll, so a row the keyboard selects in a
    /// long list of destinations scrolls into view.
    scroll: ScrollHandle,
}

impl SpaceportWindow {
    pub(super) fn open_pane_menu_at(
        &mut self,
        position: Point<Pixels>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.pressed_terminal_link = None;
        if self.menu.page.is_some() || !self.input_ready() {
            return;
        }
        let Some(surface) = &self.live.surface else {
            return;
        };
        let Some(id) = crate::terminal::pane_at(surface, self.bounds, position, self.metrics())
        else {
            return;
        };
        let id = id.to_owned();
        self.open_pane_menu(&id, position, window, cx);
    }

    fn open_pane_menu(
        &mut self,
        id: &str,
        anchor: Point<Pixels>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some((target, choices)) = self.live.snapshot.as_ref().and_then(|snapshot| {
            let target = Target::capture(snapshot, id)?;
            let choices = crate::pane_move::choices(snapshot, &target.tab);
            Some((target, choices))
        }) else {
            return;
        };
        if !self.open_menu(window, cx) {
            return;
        }
        let connect = &self.endpoints[self.selected_endpoint].connection.target;
        let daemon = crate::processes::Daemon::for_target(connect);
        let movable = moves::refusal(connect).is_none();
        self.menu.anchor = anchor;
        self.menu.page = Some(Page::Pane);
        self.menu.pane = Some(PaneMenu {
            target,
            choices,
            selected: None,
            input: None,
            pending: None,
            error: None,
            daemon,
            processes: None,
            movable,
            moving: None,
            scroll: ScrollHandle::new(),
        });
    }

    /// Opens the focused pane's rename dialog, as its menu's "Rename" row
    /// would, at the pane's top-left corner.
    pub(super) fn rename_focused_pane(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.open_focused_pane_menu(window, cx);
        if self.menu.page == Some(Page::Pane) {
            self.activate_pane_menu(Action::Rename, window, cx);
        }
    }

    /// Opens the focused pane's menu at the pane's top-left corner.
    fn open_focused_pane_menu(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(id) = self
            .live
            .snapshot
            .as_ref()
            .and_then(|s| s.focused_pane_id.clone())
        else {
            return;
        };
        let origin = self.bounds.origin;
        let anchor = self
            .live
            .surface
            .as_ref()
            .and_then(|surface| surface.panes.iter().find(|pane| pane.pane_id == id))
            .map_or(origin, |pane| {
                point(
                    origin.x + px(f32::from(pane.rect.x) * self.cell_width),
                    origin.y + px(f32::from(pane.rect.y) * self.config.terminal.line_height()),
                )
            });
        self.open_pane_menu(&id, anchor, window, cx);
    }

    /// Opens the menu of an agent's pane from its sidebar row: shows the
    /// pane, switching to its host first, as a workspace row's menu does.
    pub(crate) fn open_agent_menu(
        &mut self,
        endpoint: &str,
        pane: &str,
        anchor: Point<Pixels>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if !self.navigate_endpoint(endpoint, crate::NavigationTarget::Pane(pane), cx) {
            return;
        }
        self.open_pane_menu(pane, anchor, window, cx);
        if let Some(menu) = &mut self.menu.pane {
            // Showing the pane focuses it, so there is no other pane to swap with.
            menu.target.focused = None;
        }
        self.menu.opening_right_click = self.menu.page == Some(Page::Pane);
    }

    /// The pane whose menu, or one of its pages, is open.
    pub(crate) fn pane_menu_target(&self) -> Option<&str> {
        self.menu.page.filter(|page| page.pane_menu())?;
        self.menu
            .pane
            .as_ref()
            .map(|menu| menu.target.pane.as_str())
    }

    fn validate_pane_target(&self) -> crate::Result<&Target> {
        if !self.menu_target_current() {
            return Err(crate::Error::StaleConnection);
        }
        let target = &self
            .menu
            .pane
            .as_ref()
            .ok_or(crate::Error::StalePane)?
            .target;
        target.validate(
            self.live
                .snapshot
                .as_ref()
                .ok_or(crate::Error::NotConnected)?,
        )?;
        Ok(target)
    }

    /// Drops the pending rename and puts the caret back in its field, so a
    /// refusal reads where the name was typed.
    fn abandon_pane_rename(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(pane) = &mut self.menu.pane {
            pane.pending = None;
            if let Some(input) = &pane.input {
                window.focus(&input.read(cx).focus.clone(), cx);
            }
        }
        cx.notify();
    }

    fn pane_error(&mut self, error: impl std::fmt::Display, cx: &mut Context<Self>) {
        if let Some(pane) = &mut self.menu.pane {
            pane.error = Some(error.to_string());
        }
        cx.notify();
    }

    #[allow(clippy::too_many_lines, reason = "2026-11-27 baseline: 114 lines")]
    fn activate_pane_menu(&mut self, action: Action, window: &mut Window, cx: &mut Context<Self>) {
        if self
            .menu
            .pane
            .as_ref()
            .is_some_and(|pane| pane.moving.is_some())
        {
            return;
        }
        let target = match self.validate_pane_target() {
            Ok(target) => target.clone(),
            Err(error) => {
                self.pane_error(error, cx);
                return;
            }
        };
        match action {
            Action::MoveToNewTab => self.move_pane(
                Destination::NewTab {
                    workspace_id: target.workspace,
                },
                window,
                cx,
            ),
            Action::MoveToNewWorkspace => self.move_pane(Destination::NewWorkspace, window, cx),
            Action::MoveTo => self.open_move_picker(cx),
            Action::Rename => {
                let input = cx.new(SearchInput::new);
                input.update(cx, |input, cx| {
                    input.set_appearance(self.config.ui.clone(), self.theme.clone(), cx);
                    input.set_placeholder_with(|| fl!("pane-menu-rename-placeholder"), cx);
                    input.set_text_selected(&target.label, cx);
                    window.focus(&input.focus, cx);
                });
                if let Some(pane) = &mut self.menu.pane {
                    pane.input = Some(input);
                    pane.error = None;
                }
                self.menu.page = Some(Page::RenamePane);
                cx.notify();
            }
            Action::Close => {
                // Keep the original endpoint fence, rather than reopening the menu.
                if let Some(close) = self
                    .live
                    .snapshot
                    .as_ref()
                    .and_then(|s| CloseConfirmation::capture_pane(s, &target.pane))
                {
                    self.present_close(close, window, cx);
                }
            }
            Action::Processes => self.open_pane_processes(cx),
            Action::EditScrollback => {
                let result = (|| {
                    if !self.live.supports_edit_scrollback {
                        return Err(spaceport_client::Error::UnsupportedMethod.into());
                    }
                    if !self.input_ready() {
                        return Err(crate::Error::ConnectionNotReady);
                    }
                    let handle = self.endpoints[self.selected_endpoint]
                        .connection
                        .handle
                        .as_ref()
                        .ok_or(crate::Error::NotConnected)?;
                    // The daemon opens only its focused pane's history, and
                    // runs requests in order, so focusing first is enough.
                    let focused = self
                        .live
                        .snapshot
                        .as_ref()
                        .and_then(|s| s.focused_pane_id.as_deref());
                    if focused != Some(target.pane.as_str()) {
                        handle.focus_pane(&target.boot, &target.pane)?;
                    }
                    handle.edit_scrollback(&target.boot, &target.pane)?;
                    Ok::<_, crate::Error>(())
                })();
                match result {
                    Ok(()) => {
                        self.fence_focus_change(None);
                        self.dismiss_menu(window, cx);
                    }
                    Err(error) => self.pane_error(error, cx),
                }
            }
            action => {
                let result = (|| {
                    let (method, params) = action
                        .request(&target)
                        .ok_or(crate::Error::UnsupportedCommand)?;
                    if !self.input_ready()
                        || self
                            .live
                            .surface
                            .as_ref()
                            .is_some_and(|s| s.popup.is_some())
                    {
                        return Err(crate::Error::ConnectionNotReady);
                    }
                    let handle = self.endpoints[self.selected_endpoint]
                        .connection
                        .handle
                        .as_ref()
                        .ok_or(crate::Error::NotConnected)?;
                    handle.request(&target.boot, method, params)?;
                    Ok::<_, crate::Error>(())
                })();
                match result {
                    Ok(()) => {
                        self.fence_focus_change(None);
                        self.dismiss_menu(window, cx);
                    }
                    Err(error) => self.pane_error(error, cx),
                }
            }
        }
    }

    fn submit_pane_rename(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(pane) = &self.menu.pane else { return };
        let Some(input) = &pane.input else { return };
        if pane.pending.is_some() || input.read(cx).is_composing() {
            return;
        }
        let result = (|| {
            let target = self.validate_pane_target()?;
            if !self.input_ready()
                || self
                    .live
                    .surface
                    .as_ref()
                    .is_some_and(|s| s.popup.is_some())
            {
                return Err(crate::Error::ConnectionNotReady);
            }
            let endpoint = &self.endpoints[self.selected_endpoint];
            let handle = endpoint
                .connection
                .handle
                .as_ref()
                .ok_or(crate::Error::NotConnected)?;
            let mut inbox = endpoint
                .connection
                .inbox
                .try_lock()
                .map_err(|_| crate::Error::ConnectionBusy)?;
            // Register under the reducer's lock so even an immediate reply is retained.
            let request = handle.request(
                &target.boot,
                Method::PaneRename,
                target.rename_params(input.read(cx).text()),
            )?;
            inbox.pane_rename = Some(crate::state::RenameResult {
                request: request.clone(),
                result: None,
            });
            Ok::<_, crate::Error>(request)
        })();
        match result {
            Ok(request) => {
                if let Some(pane) = &mut self.menu.pane {
                    pane.pending = Some(request);
                    pane.error = None;
                }
                window.focus(&self.menu.focus, cx);
                cx.notify();
            }
            Err(error) => self.pane_error(error, cx),
        }
    }

    pub(super) fn poll_pane_rename(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(request) = self
            .menu
            .pane
            .as_ref()
            .and_then(|pane| pane.pending.as_ref())
        else {
            return;
        };
        // A target this window refuses is not the daemon's answer: report it
        // here, where the slot still holds no result.
        if let Err(error) = self.validate_pane_target() {
            self.abandon_pane_rename(window, cx);
            self.pane_error(error, cx);
            return;
        }
        let result = self
            .live
            .pane_rename
            .as_ref()
            .filter(|rename| &rename.request == request)
            .and_then(|rename| rename.result.clone());
        match result {
            Some(Ok(())) => self.dismiss_menu(window, cx),
            Some(Err(error)) => {
                self.abandon_pane_rename(window, cx);
                self.pane_error(error, cx);
            }
            None => {}
        }
    }

    pub(super) fn pane_menu_key(
        &mut self,
        event: &KeyDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(pane) = &mut self.menu.pane else {
            return;
        };
        let key = event.keystroke.key.as_str();
        if matches!(
            self.menu.page,
            Some(Page::PaneProcesses | Page::KillProcesses)
        ) {
            if key == "escape" && self.menu.page == Some(Page::PaneProcesses) {
                self.dismiss_menu(window, cx);
            } else if !self.pane_processes_key(key, cx) {
                return;
            }
            cx.stop_propagation();
            window.prevent_default();
            return;
        }
        if self.menu.page == Some(Page::RenamePane)
            && pane.pending.is_none()
            && (pane
                .input
                .as_ref()
                .is_some_and(|input| input.read(cx).is_composing())
                || !matches!(key, "escape" | "enter"))
        {
            return;
        }
        cx.stop_propagation();
        window.prevent_default();
        match key {
            "escape" => self.dismiss_menu(window, cx),
            "enter" if self.menu.page == Some(Page::RenamePane) => {
                self.submit_pane_rename(window, cx)
            }
            "up" | "down" if matches!(self.menu.page, Some(Page::Pane | Page::MovePane)) => {
                let count = pane.rows(self.menu.page);
                if count == 0 {
                    return;
                }
                pane.selected = Some(match (pane.selected, key) {
                    (None, "up") => count - 1,
                    (None, _) => 0,
                    (Some(i), "up") => (i + count - 1) % count,
                    (Some(i), _) => (i + 1) % count,
                });
                // The picker's heading is the panel's first child.
                if self.menu.page == Some(Page::MovePane)
                    && let Some(selected) = pane.selected
                {
                    pane.scroll.scroll_to_item(selected + 1);
                }
                cx.notify();
            }
            "enter" if self.menu.page == Some(Page::MovePane) => {
                self.submit_move_picker(window, cx)
            }
            "enter" => {
                if let Some(action) = pane
                    .selected
                    .and_then(|index| pane.actions().get(index).copied())
                {
                    self.activate_pane_menu(action, window, cx);
                }
            }
            _ => {}
        }
    }

    pub(super) fn render_pane_menu(&self, cx: &mut Context<Self>) -> Div {
        let Some(pane) = &self.menu.pane else {
            return div();
        };
        match self.menu.page {
            Some(Page::PaneProcesses) => return self.render_pane_processes(pane, cx),
            Some(Page::KillProcesses) => return self.render_kill_processes(pane, cx),
            _ => {}
        }
        let mut body = div().flex().flex_col();
        if pane.moving.is_some() {
            body = self.render_pane_move_pending();
        } else if self.menu.page == Some(Page::Pane) {
            let actions = pane.actions();
            let rows: Vec<Row> = actions
                .iter()
                .enumerate()
                .map(|(index, action)| Row {
                    label: labels::pane_action(*action, &pane.target),
                    selector: format!("pane-menu-{index}"),
                    label_selector: Some(format!("pane-menu-label-{index}")),
                    ..Row::default()
                })
                .collect();
            body = render::render_actions(
                body,
                &crate::menu::chrome::text(&self.config.ui),
                &self.theme,
                &rows,
                pane.selected,
                |index, row| {
                    let action = actions[index];
                    row.on_hover(cx.listener(move |this, hovered, _, cx| {
                        if *hovered && let Some(pane) = &mut this.menu.pane {
                            pane.selected = Some(index);
                            cx.notify();
                        }
                    }))
                    .on_click(cx.listener(move |this, _, window, cx| {
                        this.activate_pane_menu(action, window, cx)
                    }))
                },
            );
        } else {
            // The form draws the refusal its last submit earned under itself.
            let rename = form::Rename {
                buttons: form::Buttons {
                    cancel: fl!("pane-menu-cancel"),
                    submit: if pane.pending.is_some() {
                        fl!("pane-menu-renaming")
                    } else {
                        fl!("pane-menu-rename-submit")
                    },
                },
                error: pane.error.clone(),
                hint: Some(fl!("pane-menu-rename-hint")),
                selector: "pane",
                title: fl!("pane-menu-rename-title"),
            };
            let draft = pane.input.clone().map(|input| {
                if pane.pending.is_some() {
                    div()
                        .child(input.read(cx).text().to_owned())
                        .into_any_element()
                } else {
                    input.into_any_element()
                }
            });
            return form::rename(
                body,
                &rename,
                &self.theme,
                draft,
                |button, element| match button {
                    form::Button::Cancel => element
                        .on_click(cx.listener(|this, _, window, cx| this.dismiss_menu(window, cx))),
                    form::Button::Submit => element.on_click(
                        cx.listener(|this, _, window, cx| this.submit_pane_rename(window, cx)),
                    ),
                },
            );
        }
        body.when_some(pane.error.clone(), |body, error| {
            // No width of its own, so the labels set the panel's width and
            // the message wraps inside it.
            body.child(div().w_0().min_w_full().p(px(8.)).child(error))
        })
    }
}
