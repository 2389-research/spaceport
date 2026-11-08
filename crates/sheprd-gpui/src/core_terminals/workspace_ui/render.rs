//! Draws the Core group rows, compact action menu, and edit dialogs.
//! All values come from the shared Core observation; rendering does no I/O.

use super::*;
use crate::core_terminals::{Start, State as HubState};

impl SheprdWindow {
    pub(crate) fn core_workspace_rows(&self, cx: &mut Context<Self>) -> AnyElement {
        let theme = &self.theme;
        let mut section = div()
            .id("core-workspaces")
            .debug_selector(|| "core-workspaces".into())
            .flex()
            .flex_col()
            .mt_3()
            .px_3()
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .child(
                        div()
                            .flex_1()
                            .min_w_0()
                            .truncate()
                            .font_weight(FontWeight::SEMIBOLD)
                            .child(fl!("core-workspace-section")),
                    )
                    .child(
                        div()
                            .id("core-workspace-new")
                            .role(Role::Button)
                            .aria_label(fl!("core-workspace-new"))
                            .cursor_pointer()
                            .text_color(rgb(theme.foreground))
                            .child(fl!("core-workspace-new"))
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.open_core_workspace_dialog(window, cx);
                                this.core_terminals.hub.connect(Start::IfMissing);
                            })),
                    ),
            );
        match self.core_workspace_access(cx) {
            Access::Ready(document) if document.value.workspaces().is_empty() => {
                section = section.child(
                    div()
                        .text_color(rgb(theme.subtext()))
                        .child(fl!("core-workspace-empty")),
                );
            }
            Access::Ready(document) => {
                let remaining = document.value.workspaces().len().saturating_sub(50);
                let workspaces: Vec<_> = document
                    .value
                    .workspaces()
                    .iter()
                    .take(50)
                    .cloned()
                    .collect();
                for workspace in &workspaces {
                    let label = workspace.label.clone();
                    let id = workspace.id;
                    let archived = workspace.archived;
                    let selected = self.core_terminals.workspace_ui.selected_group == Some(id);
                    section = section.child(
                        div()
                            .id(SharedString::from(format!(
                                "core-workspace-{}",
                                workspace.id
                            )))
                            .debug_selector(move || format!("core-workspace-{id}"))
                            .role(Role::ListItem)
                            .flex()
                            .items_center()
                            .justify_between()
                            .py_1()
                            .when(selected, |row| row.bg(rgb(theme.active)))
                            .text_color(rgb(if archived {
                                theme.subtext()
                            } else {
                                theme.foreground
                            }))
                            .child(
                                div()
                                    .id(SharedString::from(format!("core-workspace-select-{id}")))
                                    .flex_1()
                                    .min_w_0()
                                    .truncate()
                                    .role(Role::Button)
                                    .aria_label(label.clone())
                                    .cursor_pointer()
                                    .child(crate::sidebar::label_text(&label))
                                    .on_click(cx.listener(move |this, _, _, cx| {
                                        this.select_core_workspace(id, cx);
                                    })),
                            )
                            .child(
                                div()
                                    .id(SharedString::from(format!("core-workspace-actions-{id}")))
                                    .debug_selector(move || format!("core-workspace-actions-{id}"))
                                    .role(Role::Button)
                                    .aria_label(fl!("core-workspace-actions"))
                                    .cursor_pointer()
                                    .px_2()
                                    .child("…")
                                    .on_click(cx.listener(
                                        move |this, event: &ClickEvent, window, cx| {
                                            this.open_core_workspace_actions(
                                                id,
                                                event.position(),
                                                window,
                                                cx,
                                            )
                                        },
                                    )),
                            ),
                    );
                    if selected {
                        section = section.child(self.render_core_workspace_members(workspace, cx));
                    }
                }
                if remaining > 0 {
                    section = section.child(
                        div()
                            .text_color(rgb(theme.subtext()))
                            .child(fl!("core-workspace-more", count = remaining)),
                    );
                }
            }
            Access::Error(error) => {
                section = section.child(div().text_color(rgb(theme.subtext())).child(fl!(
                    "core-workspace-error",
                    reason = bounded(&error.to_string())
                )))
            }
            Access::Missing => {
                let state = self.core_terminals.hub.state();
                let joinable = matches!(state, HubState::Idle | HubState::Unavailable(_));
                let message = match state {
                    HubState::Idle => fl!("core-workspace-connect"),
                    HubState::Connecting(_) | HubState::Ready(_) => fl!("core-workspace-loading"),
                    HubState::Unavailable(error) => fl!(
                        "core-workspace-join-error",
                        reason = bounded(&error.to_string())
                    ),
                };
                section = section.child(
                    div()
                        .id("core-workspace-status")
                        .text_color(rgb(theme.subtext()))
                        .truncate()
                        .when(joinable, |row| {
                            row.role(Role::Button)
                                .aria_label(message.clone())
                                .cursor_pointer()
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.core_terminals.hub.connect(Start::Join);
                                    cx.notify();
                                }))
                        })
                        .child(message),
                );
            }
        }
        section.into_any_element()
    }

    fn render_core_workspace_members(
        &self,
        workspace: &crate::core_terminals::workspace_document::Workspace,
        cx: &mut Context<Self>,
    ) -> Div {
        let group = workspace.id;
        let theme = &self.theme;
        let observed_core = cx
            .try_global::<observation::Observation>()
            .filter(|observation| observation.projection().fresh())
            .and_then(|observation| observation.projection().core_id());
        let local_core = self
            .core_terminals
            .hub
            .backend()
            .filter(|backend| !backend.core().is_lost())
            .map(|backend| backend.core().hello().core.id.as_str())
            .filter(|id| Some(*id) == observed_core);
        let terminals = cx
            .try_global::<observation::Observation>()
            .filter(|observation| observation.projection().fresh())
            .map(|observation| observation.projection().terminals())
            .unwrap_or_default();
        let mut rows = div().flex().flex_col().pl_3();
        for member in workspace.members.iter().take(50) {
            let core_id = member.core_id.clone();
            let terminal_id = member.terminal_id.clone();
            let live = if local_core == Some(core_id.as_str()) {
                terminals
                    .iter()
                    .find(|info| info.id.as_str() == terminal_id)
            } else {
                None
            };
            let available = live.is_some();
            let label = match live {
                Some(info) => bounded(
                    info.name
                        .as_deref()
                        .or(info.title.as_deref())
                        .unwrap_or(&terminal_id),
                ),
                None => format!(
                    "{core_id}/{terminal_id} ({})",
                    fl!("core-workspace-member-unavailable")
                ),
            };
            let open_core = core_id.clone();
            let open_terminal = terminal_id.clone();
            let remove_core = core_id.clone();
            let remove_terminal = terminal_id.clone();
            let remove_selector = format!("core-member-remove-{group}-{core_id}-{terminal_id}");
            let row_core = core_id.clone();
            let row_terminal = terminal_id.clone();
            rows = rows.child(
                div()
                    .id(SharedString::from(format!(
                        "core-member-{core_id}-{terminal_id}"
                    )))
                    .debug_selector(move || format!("core-member-{row_core}-{row_terminal}"))
                    .flex()
                    .items_center()
                    .justify_between()
                    .min_w_0()
                    .child(
                        div()
                            .id(SharedString::from(format!(
                                "core-member-open-{group}-{open_core}-{open_terminal}"
                            )))
                            .flex_1()
                            .min_w_0()
                            .truncate()
                            .text_color(rgb(if available {
                                theme.foreground
                            } else {
                                theme.muted
                            }))
                            .when(available, |row| {
                                row.role(Role::Button)
                                    .aria_label(format!("{core_id}/{terminal_id}"))
                                    .cursor_pointer()
                                    .on_click(cx.listener(move |this, _, window, cx| {
                                        this.open_core_workspace_member(
                                            group,
                                            &open_core,
                                            &open_terminal,
                                            window,
                                            cx,
                                        );
                                    }))
                            })
                            .child(crate::sidebar::label_text(&label)),
                    )
                    .child(
                        div()
                            .id(SharedString::from(remove_selector.clone()))
                            .debug_selector(move || remove_selector.clone())
                            .role(Role::Button)
                            .aria_label(fl!("core-workspace-member-remove"))
                            .cursor_pointer()
                            .px_2()
                            .child("−")
                            .on_click(cx.listener(move |this, _, _, cx| {
                                this.toggle_core_workspace_member(
                                    group,
                                    remove_core.clone(),
                                    remove_terminal.clone(),
                                    false,
                                    cx,
                                );
                            })),
                    ),
            );
        }
        let remaining = workspace.members.len().saturating_sub(50);
        if remaining > 0 {
            rows = rows.child(fl!("core-workspace-members-more", count = remaining));
        }
        if let Some(local_core) = local_core {
            let available = terminals.iter().filter(|info| {
                !info.id.as_str().contains('/')
                    && !workspace.members.iter().any(|member| {
                        member.core_id == local_core && member.terminal_id == info.id.as_str()
                    })
            });
            for info in available.take(50) {
                let core_id = local_core.to_owned();
                let terminal_id = info.id.to_string();
                let label = bounded(
                    info.name
                        .as_deref()
                        .or(info.title.as_deref())
                        .unwrap_or(info.id.as_str()),
                );
                rows = rows.child(
                    div()
                        .flex()
                        .items_center()
                        .justify_between()
                        .min_w_0()
                        .child(
                            div()
                                .flex_1()
                                .min_w_0()
                                .truncate()
                                .child(crate::sidebar::label_text(&label)),
                        )
                        .child(
                            div()
                                .id(SharedString::from(format!(
                                    "core-member-add-{group}-{core_id}-{terminal_id}"
                                )))
                                .role(Role::Button)
                                .aria_label(fl!("core-workspace-member-add"))
                                .cursor_pointer()
                                .px_2()
                                .child("+")
                                .on_click(cx.listener(move |this, _, _, cx| {
                                    this.toggle_core_workspace_member(
                                        group,
                                        core_id.clone(),
                                        terminal_id.clone(),
                                        true,
                                        cx,
                                    );
                                })),
                        ),
                );
            }
        }
        rows
    }

    pub(crate) fn render_core_workspace_actions(&self, cx: &mut Context<Self>) -> Div {
        let Some(target) = self.core_terminals.workspace_ui.target else {
            return div();
        };
        let (label, archived) = match self.core_workspace_access(cx) {
            Access::Ready(document)
                if document.version == target.version
                    && Self::core_workspace_epoch(cx) == Some(target.epoch) =>
            {
                match document
                    .value
                    .workspaces()
                    .iter()
                    .find(|row| row.id == target.id)
                {
                    Some(row) => (row.label.clone(), row.archived),
                    None => return div().p_2().child(fl!("core-workspace-changed")),
                }
            }
            _ => return div().p_2().child(fl!("core-workspace-changed")),
        };
        let id = target.id;
        let selected = self.core_terminals.workspace_ui.selected;
        let archive_label = if archived {
            fl!("core-workspace-restore")
        } else {
            fl!("core-workspace-archive")
        };
        div()
            .flex()
            .flex_col()
            .p_2()
            .child(
                div()
                    .truncate()
                    .font_weight(FontWeight::SEMIBOLD)
                    .child(label),
            )
            .child(div().truncate().child(id.to_string()))
            .when_some(self.menu.error.as_ref(), |menu, error| {
                menu.child(
                    div()
                        .text_color(crate::menu::danger(&self.theme))
                        .child(error.clone()),
                )
            })
            .child(
                div()
                    .id("core-workspace-action-rename")
                    .role(Role::MenuItem)
                    .aria_label(fl!("core-workspace-rename"))
                    .when(selected == MenuAction::Rename, |row| {
                        row.aria_active_descendant()
                    })
                    .cursor_pointer()
                    .p_2()
                    .when(selected == MenuAction::Rename, |row| {
                        row.bg(rgb(self.theme.active))
                    })
                    .child(fl!("core-workspace-rename"))
                    .on_hover(cx.listener(|this, hovered: &bool, _, cx| {
                        this.hover_core_workspace_action(MenuAction::Rename, *hovered, cx)
                    }))
                    .on_click(cx.listener(move |this, _, window, cx| {
                        cx.stop_propagation();
                        this.activate_core_workspace_action(MenuAction::Rename, window, cx)
                    })),
            )
            .child(
                div()
                    .id("core-workspace-action-archive")
                    .role(Role::MenuItem)
                    .aria_label(archive_label.clone())
                    .when(selected == MenuAction::Archive, |row| {
                        row.aria_active_descendant()
                    })
                    .cursor_pointer()
                    .p_2()
                    .when(selected == MenuAction::Archive, |row| {
                        row.bg(rgb(self.theme.active))
                    })
                    .child(archive_label)
                    .on_hover(cx.listener(|this, hovered: &bool, _, cx| {
                        this.hover_core_workspace_action(MenuAction::Archive, *hovered, cx)
                    }))
                    .on_click(cx.listener(move |this, _, window, cx| {
                        cx.stop_propagation();
                        this.activate_core_workspace_action(MenuAction::Archive, window, cx)
                    })),
            )
            .child(
                div()
                    .id("core-workspace-action-delete")
                    .role(Role::MenuItem)
                    .aria_label(fl!("core-workspace-delete"))
                    .when(selected == MenuAction::Delete, |row| {
                        row.aria_active_descendant()
                    })
                    .cursor_pointer()
                    .p_2()
                    .when(selected == MenuAction::Delete, |row| {
                        row.bg(rgb(self.theme.active))
                    })
                    .text_color(crate::menu::danger(&self.theme))
                    .child(fl!("core-workspace-delete"))
                    .on_hover(cx.listener(|this, hovered: &bool, _, cx| {
                        this.hover_core_workspace_action(MenuAction::Delete, *hovered, cx)
                    }))
                    .on_click(cx.listener(move |this, _, window, cx| {
                        cx.stop_propagation();
                        this.activate_core_workspace_action(MenuAction::Delete, window, cx)
                    })),
            )
    }

    pub(crate) fn render_core_workspace_dialog(&self, cx: &mut Context<Self>) -> Div {
        let theme = &self.theme;
        let status = if self.core_terminals.workspace_ui.pending.is_some() {
            Some(fl!("core-workspace-editing"))
        } else {
            match self.core_workspace_access(cx) {
                Access::Error(error) => Some(fl!(
                    "core-workspace-error",
                    reason = bounded(&error.to_string())
                )),
                Access::Missing => Some(fl!("core-workspace-stale")),
                Access::Ready(document)
                    if self
                        .core_terminals
                        .workspace_ui
                        .target
                        .is_some_and(|target| {
                            target.version != document.version
                                || Self::core_workspace_epoch(cx) != Some(target.epoch)
                                || !document
                                    .value
                                    .workspaces()
                                    .iter()
                                    .any(|row| row.id == target.id)
                        }) =>
                {
                    Some(fl!("core-workspace-changed"))
                }
                Access::Ready(document) if EditGuard::blocks(cx, document.version) => {
                    Some(fl!("core-workspace-uncertain"))
                }
                Access::Ready(_)
                    if !self
                        .core_terminals
                        .hub
                        .backend()
                        .is_some_and(|backend| !backend.core().is_lost()) =>
                {
                    Some(fl!("core-workspace-connect"))
                }
                Access::Ready(_) => None,
            }
        };
        let named = matches!(
            self.menu.page,
            Some(Page::CoreWorkspaceCreate | Page::CoreWorkspaceRename)
        );
        let armed =
            status.is_none()
                && (!named
                    || self.menu.input.as_ref().is_some_and(|input| {
                        !input.text.trim().is_empty() && input.marked.is_none()
                    }));
        let title = match self.menu.page {
            Some(Page::CoreWorkspaceRename) => fl!("core-workspace-rename-title"),
            Some(Page::CoreWorkspaceDelete) => fl!("core-workspace-delete-title"),
            _ => fl!("menu-page-new-workspace"),
        };
        let submit = match self.menu.page {
            Some(Page::CoreWorkspaceRename) => fl!("core-workspace-rename"),
            Some(Page::CoreWorkspaceDelete) => fl!("core-workspace-delete"),
            _ => fl!("workspace-dialog-create"),
        };
        let mut body = div()
            .flex()
            .flex_col()
            .gap_3()
            .p_4()
            .child(div().font_weight(FontWeight::SEMIBOLD).child(title));
        if named {
            body = body.child(self.render_dialog_input(cx));
        } else if let Some(target) = self.core_terminals.workspace_ui.target {
            if let Access::Ready(document) = self.core_workspace_access(cx)
                && let Some(row) = document
                    .value
                    .workspaces()
                    .iter()
                    .find(|row| row.id == target.id)
            {
                body = body.child(fl!(
                    "core-workspace-delete-note",
                    name = row.label.as_str(),
                    id = target.id.to_string()
                ));
            }
        }
        if let Some(error) = &self.menu.error {
            body = body.child(
                div()
                    .text_color(crate::menu::danger(theme))
                    .child(error.clone()),
            );
        } else if let Some(status) = status {
            body = body.child(div().text_color(rgb(theme.subtext())).child(status));
        }
        div().child(body).child(
            div()
                .flex()
                .justify_end()
                .gap_2()
                .p_4()
                .child(
                    div()
                        .id("core-workspace-cancel")
                        .role(Role::Button)
                        .aria_label(fl!("workspace-dialog-cancel"))
                        .cursor_pointer()
                        .on_click(cx.listener(|this, _, window, cx| this.dismiss_menu(window, cx)))
                        .child(fl!("workspace-dialog-cancel")),
                )
                .child(
                    div()
                        .id("core-workspace-create")
                        .role(Role::Button)
                        .aria_label(submit.clone())
                        .when(armed, |button| {
                            button.cursor_pointer().on_click(
                                cx.listener(|this, _, _, cx| this.submit_core_workspace_dialog(cx)),
                            )
                        })
                        .text_color(if self.menu.page == Some(Page::CoreWorkspaceDelete) {
                            crate::menu::danger(theme)
                        } else {
                            rgb(theme.foreground)
                        })
                        .child(submit),
                ),
        )
    }
}
