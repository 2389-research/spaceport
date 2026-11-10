// i18n: migrated
//! The sidebar host header's context menu. Saved SSH devices
//! have this one; a WSL
//! distribution's only offers its removal (see `wsl`). Local is always
//! present, so it offers only its grouping and mute, and an explicit socket is
//! not in the catalog.
use super::setup;
use crate::i18n::fl;
use crate::{
    SheprdWindow,
    config::{Config, KeybindingSource},
    menu::Page,
    search_input::SearchInput,
};
use actions::{Action, LOCAL_ACTIONS, SSH_ACTIONS};
use gpui::{prelude::*, *};
use sheprd_client::ConnectTarget;

/// The saved entry the menu acts on.
#[derive(Clone, Debug, PartialEq, Eq)]
enum Saved {
    /// This computer's daemon, which is not in any catalog.
    Local,
    /// A catalog profile, which the `herdr machine` commands name.
    Ssh { profile: String },
}

pub(crate) struct HostMenu {
    /// The endpoint the window and device catalog key by.
    id: String,
    saved: Saved,
    label: String,
    /// The SSH target. An empty new name falls back to it.
    target: String,
    session: String,
    selected: Option<usize>,
    /// The new name while renaming, or the port to forward.
    input: Option<Entity<SearchInput>>,
    renaming: bool,
    error: Option<String>,
    /// The terminal theme picker, while it is the page shown.
    theme: Option<theme::DeviceThemePicker>,
    /// The tag picker, while it is the page shown.
    tags: Option<tags::TagPicker>,
}

impl HostMenu {
    fn actions(&self) -> &'static [(Action, &'static str)] {
        match self.saved {
            Saved::Local => LOCAL_ACTIONS,
            Saved::Ssh { .. } => SSH_ACTIONS,
        }
    }

    /// The catalog profile `[devices.<profile>]` settings are keyed by.
    fn profile(&self) -> Option<&str> {
        match &self.saved {
            Saved::Ssh { profile } => Some(profile),
            Saved::Local => None,
        }
    }

    /// The SSH target port forwards are keyed by.
    fn ssh_target(&self) -> Option<&str> {
        matches!(self.saved, Saved::Ssh { .. }).then_some(self.target.as_str())
    }
}

impl SheprdWindow {
    pub(crate) fn open_host_menu(
        &mut self,
        id: &str,
        anchor: Point<Pixels>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        tracing::info!(category = "device_remove", id, "Opening host menu");
        if id.starts_with(crate::endpoint::WSL_PREFIX) {
            if self.device_setup_unavailable().is_none() {
                self.open_remove_wsl(id, anchor, window, cx);
            }
            return;
        }
        // A device already being removed has nothing left to offer.
        if self.menu.removing_devices.contains(id) {
            tracing::info!(
                category = "device_remove",
                id,
                "Host menu refused: the device is still being removed"
            );
            return;
        }
        let Some(endpoint) = self.endpoints.iter().find(|endpoint| endpoint.id == id) else {
            tracing::info!(
                category = "device_remove",
                id,
                "Host menu refused: no such device"
            );
            return;
        };
        // The saved entry, not the live target: removal claims and names the
        // profile, whichever session the list has attached the device to.
        let (saved, target, session) = match endpoint.saved_target() {
            _ if endpoint.id == crate::endpoint::LOCAL => (Saved::Local, "", ""),
            ConnectTarget::Ssh { target, session } => {
                // The CLI edits the default catalog, the one device setup also uses.
                if let Some(reason) = self.device_setup_unavailable() {
                    tracing::info!(
                        category = "device_remove",
                        id,
                        ?reason,
                        "Host menu unavailable"
                    );
                    return;
                }
                let Some(profile) = crate::endpoint::saved_profile_id(&endpoint.id) else {
                    tracing::info!(
                        category = "device_remove",
                        id,
                        "Host menu refused: the device has no saved profile"
                    );
                    return;
                };
                (
                    Saved::Ssh {
                        profile: profile.to_owned(),
                    },
                    target.as_str(),
                    session.as_str(),
                )
            }
            _ => {
                tracing::info!(
                    category = "device_remove",
                    id,
                    "Host menu refused: the device is not Local or a saved device"
                );
                return;
            }
        };
        let menu = HostMenu {
            id: endpoint.id.clone(),
            saved,
            label: endpoint.label.clone(),
            target: target.to_owned(),
            session: session.to_owned(),
            selected: None,
            input: None,
            renaming: false,
            error: None,
            theme: None,
            tags: None,
        };
        if !self.open_menu(window, cx) {
            tracing::info!(
                category = "device_remove",
                id,
                "Host menu refused: menu did not open"
            );
            return;
        }
        self.menu.anchor = anchor;
        self.menu.page = Some(Page::Host);
        self.menu.host = Some(menu);
    }

    fn activate_host_menu(&mut self, action: Action, window: &mut Window, cx: &mut Context<Self>) {
        tracing::info!(
            category = "device_remove",
            ?action,
            id = self.menu.host.as_ref().map(|host| host.id.as_str()),
            "Host menu action"
        );
        match action {
            Action::Rename => {
                let Some(host) = &self.menu.host else {
                    return;
                };
                let (label, target) = (host.label.clone(), host.target.clone());
                let input = cx.new(SearchInput::new);
                input.update(cx, |input, cx| {
                    input.set_appearance(self.config.ui.clone(), self.theme.clone(), cx);
                    // An empty name falls back to the target, as when adding.
                    input.set_placeholder(&target, cx);
                    input.set_text_selected(&label, cx);
                    window.focus(&input.focus, cx);
                });
                if let Some(host) = &mut self.menu.host {
                    host.input = Some(input);
                    host.error = None;
                }
                self.menu.page = Some(Page::RenameDevice);
            }
            Action::Mute => {
                if let Some(id) = self.menu.host.as_ref().map(|host| host.id.clone()) {
                    let muted = crate::notification_mutes::Mutes::of(cx)
                        .is_some_and(|mutes| mutes.host(&id));
                    self.set_host_muted(&id, !muted, cx);
                }
            }
            Action::ServerKeybindings => self.toggle_server_keybindings(cx),
            Action::Theme => self.open_device_theme(window, cx),
            Action::Tags(mode) => self.open_device_tags(mode, window, cx),
            #[cfg(unix)]
            Action::ForwardPort => {
                let input = cx.new(SearchInput::new);
                input.update(cx, |input, cx| {
                    input.set_appearance(self.config.ui.clone(), self.theme.clone(), cx);
                    input.set_placeholder("3000", cx);
                    window.focus(&input.focus, cx);
                });
                if let Some(host) = &mut self.menu.host {
                    host.input = Some(input);
                    host.error = None;
                }
                self.menu.page = Some(Page::ForwardPort);
            }
            Action::Remove => {
                self.menu.page = Some(Page::RemoveDevice);
            }
        }
        cx.notify();
    }

    /// Saves the other choice. The reload that follows the save applies it, in
    /// every window, so a save that could not start changes nothing.
    fn toggle_server_keybindings(&mut self, cx: &mut Context<Self>) {
        let Some(HostMenu {
            id,
            saved: Saved::Ssh { profile },
            ..
        }) = &self.menu.host
        else {
            return;
        };
        let profile = profile.clone();
        let source = match self.config.keybinding_source(id) {
            KeybindingSource::Local => KeybindingSource::Server,
            KeybindingSource::Server => KeybindingSource::Local,
        };
        self.save_preference(
            move || Config::save_device_keybindings(&profile, source),
            cx,
        );
    }

    /// Rename in the background and close once the catalog has the new name;
    /// the sidebar and picker pick it up from there.
    fn submit_rename_device(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(host) = &mut self.menu.host else {
            return;
        };
        let Some(input) = &host.input else {
            return;
        };
        if host.renaming || input.read(cx).is_composing() {
            return;
        }
        let label = match setup::device_label(input.read(cx).text(), &host.target) {
            Ok(label) => label,
            Err(error) => {
                host.error = Some(error.to_string());
                cx.notify();
                return;
            }
        };
        let (id, saved) = (host.id.clone(), host.saved.clone());
        host.renaming = true;
        host.error = None;
        let background = cx
            .background_executor()
            .spawn(async move { rename(&saved, &label) });
        cx.spawn_in(window, async move |this, cx| {
            let result = background.await;
            let _ = this.update_in(cx, |this, window, cx| {
                // The menu may have closed or moved to another device meanwhile.
                let Some(host) = this
                    .menu
                    .host
                    .as_mut()
                    .filter(|host| host.id == id && host.renaming)
                else {
                    return;
                };
                host.renaming = false;
                match result {
                    Ok(()) => this.dismiss_menu(window, cx),
                    Err(error) => {
                        host.error = Some(error.to_string());
                        if let Some(input) = &host.input {
                            window.focus(&input.read(cx).focus.clone(), cx);
                        }
                        cx.notify();
                    }
                }
            });
        })
        .detach();
        cx.notify();
    }

    /// Close the confirmation at once and remove in the background. Until the
    /// catalog drops the device, its sidebar header pulses the way a worktree
    /// row does while it is removed; a failure is reported in the status bar.
    fn confirm_remove_device(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(host) = self.menu.host.take() else {
            tracing::warn!(
                category = "device_remove",
                "Remove confirmed without a host menu"
            );
            return;
        };
        let HostMenu {
            id,
            saved,
            label,
            target,
            session,
            ..
        } = host;
        tracing::info!(category = "device_remove", id, "Removing device");
        let background = cx.background_executor().spawn(async move {
            match saved {
                // Offers no Remove.
                Saved::Local => Ok(()),
                Saved::Ssh { profile } => {
                    let _claim = setup::claim_saved(&target, &session)?;
                    setup::remove(&profile)?;
                    Ok::<_, crate::Error>(())
                } // Forget only the exact saved row; leave the machine running.
            }
        });
        self.menu.removing_devices.insert(id.clone());
        self.dismiss_menu(window, cx);
        Self::follow_removal(cx.entity().downgrade(), background, id, label, cx);
    }

    /// Applies a removal's result. Its mute goes first, from the app: the
    /// window that started the removal may have closed by the time it ends.
    fn follow_removal(
        this: WeakEntity<Self>,
        background: Task<crate::Result<()>>,
        id: String,
        label: String,
        cx: &mut App,
    ) {
        cx.spawn(async move |cx| {
            let result = background.await;
            if result.is_ok() {
                cx.update(|cx| crate::notification_mutes::Mutes::forget_host(cx, &id));
            }
            let _ = this.update(cx, |this, cx| this.device_removed(&id, &label, result, cx));
        })
        .detach();
    }

    fn device_removed(
        &mut self,
        id: &str,
        label: &str,
        result: crate::Result<()>,
        cx: &mut Context<Self>,
    ) {
        match &result {
            Ok(_) => tracing::info!(
                category = "device_remove",
                id,
                "Device removed; waiting for the catalog to drop it"
            ),
            Err(_) => tracing::warn!(category = "device_remove", id, "Device removal failed"),
        }
        // On success the catalog watcher drops the device, and a filter or
        // selection on it falls back to Local, as for any removal. The header
        // keeps pulsing until then; `prune_device_removals` clears the mark.
        if let Err(error) = result {
            self.menu.removing_devices.remove(id);
            self.local_error = Some(fl!(
                "host-menu-remove-failed",
                device = label,
                error = error.to_string(),
            ));
        }
        cx.notify();
    }

    /// Forget removal marks for devices the catalog no longer lists.
    pub(crate) fn prune_device_removals(&mut self) {
        let endpoints = &self.endpoints;
        self.menu.removing_devices.retain(|id| {
            let listed = endpoints.iter().any(|endpoint| &endpoint.id == id);
            if !listed {
                tracing::info!(category = "device_remove", id, "Catalog dropped the device");
            }
            listed
        });
    }

    pub(in crate::menu) fn host_menu_key(
        &mut self,
        event: &KeyDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(host) = &mut self.menu.host else {
            return;
        };
        let key = event.keystroke.key.as_str();
        // The name and port fields own typing and composition; only their
        // submit and cancel keys belong to the menu.
        if matches!(self.menu.page, Some(Page::RenameDevice | Page::ForwardPort))
            && !host.renaming
            && (host
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
            "escape" => {
                tracing::info!(category = "device_remove", page = ?self.menu.page, "Host menu dismissed");
                self.dismiss_menu(window, cx)
            }
            "enter" if self.menu.page == Some(Page::RenameDevice) => {
                self.submit_rename_device(window, cx)
            }
            "enter" if self.menu.page == Some(Page::ForwardPort) => {
                self.submit_forward_port(window, cx)
            }
            "enter" if self.menu.page == Some(Page::RemoveDevice) => {
                self.confirm_remove_device(window, cx)
            }
            key @ ("up" | "down") if self.menu.page == Some(Page::Host) => {
                let count = host.actions().len();
                host.selected = Some(match (host.selected, key) {
                    (None, "up") => count - 1,
                    (None, _) => 0,
                    (Some(i), "up") => (i + count - 1) % count,
                    (Some(i), _) => (i + 1) % count,
                });
                cx.notify();
            }
            "enter" => {
                if let Some(&(action, _)) =
                    host.selected.and_then(|index| host.actions().get(index))
                {
                    self.activate_host_menu(action, window, cx);
                }
            }
            _ => {}
        }
    }

    pub(in crate::menu) fn render_host_menu(&self, cx: &mut Context<Self>) -> Div {
        let Some(host) = &self.menu.host else {
            return div();
        };
        let theme = &self.theme;
        let mut body = div().flex().flex_col();
        if self.menu.page == Some(Page::Host) {
            // Name the device first, so the destructive row below cannot be
            // mistaken for acting on another host.
            body = body.child(
                // No width of its own, so a long name or target ends with
                // "…" instead of widening the menu past its actions.
                div()
                    .debug_selector(|| "host-menu-header".into())
                    .w_0()
                    .min_w_full()
                    .px(px(8.))
                    .pt(px(4.))
                    .pb(px(8.))
                    .mb(px(4.))
                    .border_b_1()
                    .border_color(rgb(theme.active))
                    .child(
                        div()
                            .font_weight(FontWeight::SEMIBOLD)
                            .truncate()
                            .child(host.label.clone()),
                    )
                    .when(!host.target.is_empty(), |header| {
                        header.child(
                            div()
                                .text_size(px(self.config.ui.size * 0.85))
                                .text_color(rgb(theme.muted))
                                .truncate()
                                // The device's target and session.
                                .child(format!("{} · {}", host.target, host.session)), // i18n: data
                        )
                    }),
            );
            let server_keys = self.config.keybinding_source(&host.id) == KeybindingSource::Server;
            let device_theme = self.device_theme_value(&host.id).map(str::to_owned);
            let muted =
                crate::notification_mutes::Mutes::of(cx).is_some_and(|mutes| mutes.host(&host.id));
            for (index, &(action, message)) in host.actions().iter().enumerate() {
                let label = crate::i18n::message(message, &[]);
                let toggle = match action {
                    Action::Mute => Some(muted),
                    Action::ServerKeybindings => Some(server_keys),
                    _ => None,
                };
                body = body.child(
                    div()
                        .id(("host-menu-action", index))
                        .debug_selector(move || format!("host-menu-{index}"))
                        .min_h(px(self.config.ui.line_height() + 12.))
                        .px(px(8.))
                        .flex()
                        .items_center()
                        .gap(px(8.))
                        .cursor_pointer()
                        .when_some(toggle, |row, checked| {
                            row.child(crate::toggles::checkbox(
                                theme,
                                self.config.ui.size + 2.,
                                checked,
                            ))
                        })
                        .when(action == Action::Remove, |row| {
                            row.text_color(crate::menu::danger(theme))
                        })
                        .when(host.selected == Some(index), |row| {
                            row.bg(rgb(theme.active))
                        })
                        .hover(|row| row.bg(rgb(theme.active)))
                        .child(
                            div()
                                .debug_selector(move || format!("host-menu-label-{index}"))
                                .min_w_0()
                                .truncate()
                                .child(label),
                        )
                        .when_some(
                            device_theme.clone().filter(|_| action == Action::Theme),
                            |row, name| {
                                row.child(
                                    div()
                                        .debug_selector(|| "host-menu-theme".into())
                                        .ml_auto()
                                        .min_w_0()
                                        .truncate()
                                        .text_color(rgb(theme.muted))
                                        .child(name),
                                )
                            },
                        )
                        .on_hover(cx.listener(move |this, hovered, _, cx| {
                            if *hovered && let Some(host) = &mut this.menu.host {
                                host.selected = Some(index);
                                cx.notify();
                            }
                        }))
                        .on_click(cx.listener(move |this, _, window, cx| {
                            this.activate_host_menu(action, window, cx)
                        })),
                );
                // Opting in never silently changes nothing: say why the
                // device is still on local keys.
                if let Some(error) = (action == Action::ServerKeybindings)
                    .then(|| self.server_keybindings_error(&host.id))
                    .flatten()
                {
                    body = body.child(
                        // No width of its own, so the labels set the menu's
                        // width and the message wraps inside it.
                        div()
                            .debug_selector(|| "host-menu-keybindings-error".into())
                            .w_0()
                            .min_w_full()
                            .px(px(8.))
                            .pb(px(4.))
                            .text_size(px(self.config.ui.size * 0.85))
                            .text_color(rgb(theme.muted))
                            .child(fl!(
                                "host-menu-local-keybindings",
                                error = error.to_string()
                            )),
                    );
                }
            }
            return body.child(self.render_forwards(host, cx));
        }
        if self.menu.page == Some(Page::RenameDevice) {
            return self.render_rename_device(host, cx);
        }
        if self.menu.page == Some(Page::ForwardPort) {
            return self.render_forward_port(host, cx);
        }
        if self.menu.page == Some(Page::DeviceTheme) {
            return self.render_device_theme(host, cx);
        }
        if self.menu.page == Some(Page::DeviceTags) {
            return self.render_device_tags(host, cx);
        }
        body = body
            .debug_selector(|| "remove-device".into())
            .p(px(8.))
            .gap(px(12.))
            .child(
                div()
                    .font_weight(FontWeight::SEMIBOLD)
                    .child(fl!("host-menu-remove-title")),
            )
            .child(
                div()
                    .text_color(rgb(theme.subtext()))
                    .child(removal_note(host)),
            )
            .child(
                div()
                    .flex()
                    .justify_end()
                    .gap(px(8.))
                    .child(
                        div()
                            .id("remove-device-cancel")
                            .p(px(6.))
                            .cursor_pointer()
                            .child(fl!("host-menu-cancel"))
                            .on_click(cx.listener(|this, _, window, cx| {
                                tracing::info!(category = "device_remove", "Remove cancelled");
                                this.dismiss_menu(window, cx)
                            })),
                    )
                    .child(
                        div()
                            .id("remove-device-submit")
                            .debug_selector(|| "remove-device-submit".into())
                            .p(px(6.))
                            .rounded(px(crate::config::corners::CONTROL))
                            .bg(rgb(theme.active))
                            .text_color(crate::menu::danger(theme))
                            .cursor_pointer()
                            .child(fl!("host-menu-remove-submit"))
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.confirm_remove_device(window, cx)
                            })),
                    ),
            );
        body
    }
}

/// Rename the saved SSH profile through `herdr machine rename` on a worker.
fn rename(saved: &Saved, label: &str) -> crate::Result<()> {
    match saved {
        // Offers no Rename.
        Saved::Local => Ok(()),
        Saved::Ssh { profile } => setup::rename(profile, label),
    }
}

/// What removal does, as the confirmation says it: an SSH device leaves its
/// host's Herdr running.
fn removal_note(host: &HostMenu) -> String {
    match host.saved {
        Saved::Local => String::new(),
        Saved::Ssh { .. } => fl!(
            "host-menu-remove-ssh-note",
            device = host.label.as_str(),
            target = host.target.as_str(),
            session = host.session.as_str(),
        ),
    }
}

mod actions;
mod forwards;
mod rename;
mod tags;
mod theme;

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests;
