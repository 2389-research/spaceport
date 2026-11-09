//! Painting a workspace dialog: its header, the body each action asks for,
//! and the footer whose submit button arms only once there is something to send.

use crate::{
    SheprdWindow,
    i18n::fl,
    menu::{WorkspaceAction, WorkspaceMenuAction, danger},
};
use gpui::{prelude::*, *};

impl SheprdWindow {
    pub(in crate::menu) fn render_workspace_dialog(
        &self,
        action: WorkspaceAction,
        cx: &mut Context<Self>,
    ) -> Div {
        let Some(target) = &self.menu.target else {
            return div();
        };
        let theme = &self.theme;
        let font = &self.config.ui;
        let danger = danger(theme);
        let creating = self.menu.opening.is_some();
        let armed = (action != WorkspaceAction::OpenWorktree
            || self.menu.worktree_open.as_ref().is_some_and(|picker| {
                picker.pending.is_none()
                    && !picker.filtered.is_empty()
                    && !picker.search.read(cx).is_composing()
            }))
            && !creating;
        let (title, submit): (String, SharedString) = match action {
            WorkspaceAction::Rename => (
                fl!("workspace-dialog-rename-title"),
                fl!("workspace-dialog-rename").into(),
            ),
            WorkspaceAction::OpenWorktree => (
                fl!("workspace-dialog-open-worktree-title"),
                fl!("workspace-dialog-open").into(),
            ),
            WorkspaceAction::NewTab => (
                fl!("workspace-dialog-new-tab-title"),
                fl!("workspace-dialog-create").into(),
            ),
            WorkspaceAction::NewWorkspace => (
                fl!("workspace-dialog-new-workspace-title"),
                fl!("workspace-dialog-create").into(),
            ),
            WorkspaceAction::Note => (
                fl!("workspace-dialog-note-title"),
                fl!("workspace-dialog-save").into(),
            ),
        };
        let mut body = div().flex().flex_col().gap(px(10.)).px(px(16.)).py(px(12.));
        body = match action {
            WorkspaceAction::OpenWorktree => body,
            WorkspaceAction::Rename => body.child(
                div()
                    .text_color(rgb(theme.subtext()))
                    .child(fl!("workspace-dialog-rename-note")),
            ),
            WorkspaceAction::Note => body.child(
                div()
                    .text_color(rgb(theme.subtext()))
                    .child(fl!("workspace-dialog-note-note")),
            ),
            WorkspaceAction::NewTab => body.child(
                div()
                    .text_color(rgb(theme.subtext()))
                    .child(fl!("workspace-dialog-new-tab-note")),
            ),
            WorkspaceAction::NewWorkspace => body.child(
                div()
                    .text_color(rgb(theme.subtext()))
                    .child(fl!("workspace-dialog-new-workspace-note")),
            ),
        };
        if self.menu.input.is_some() {
            body = body.child(self.render_dialog_input(cx));
        }
        if let Some(error) = &self.menu.error {
            body = body.child(
                div()
                    .debug_selector(|| "dialog-error".into())
                    .rounded(px(crate::config::corners::CONTROL))
                    .bg(rgb(theme.active))
                    .px(px(10.))
                    .py(px(6.))
                    .text_color(danger)
                    .child(error.clone()),
            );
        }
        if creating {
            // Dismissing only closes the panel; the daemon keeps the queued work.
            body = body.child(
                div()
                    .debug_selector(|| "dialog-waiting".into())
                    .text_color(rgb(theme.subtext()))
                    .child(fl!("workspace-dialog-daemon-waiting")),
            );
        }
        let button = |id: &'static str| {
            div()
                .id(id)
                .debug_selector(move || id.into())
                .px(px(12.))
                .py(px(6.))
                .rounded(px(crate::config::corners::CONTROL))
                .border_1()
                .cursor_pointer()
        };
        // A picker owns the panel's height, so its list scrolls inside the
        // dialog instead of growing it past the window.
        let listing = action == WorkspaceAction::OpenWorktree;
        div()
            .flex()
            .flex_col()
            .when(listing, |dialog| dialog.size_full().min_h_0())
            .child(
                div()
                    .debug_selector(|| "dialog-header".into())
                    .flex()
                    .items_center()
                    .gap(px(10.))
                    .px(px(16.))
                    .py(px(12.))
                    .border_b_1()
                    .border_color(rgb(theme.active))
                    .when_some(
                        WorkspaceMenuAction::Dialog(action).icon(),
                        |header, icon| {
                            header.child(
                                svg()
                                    .path(icon)
                                    .size(px(16.))
                                    .flex_none()
                                    .text_color(rgb(theme.muted)),
                            )
                        },
                    )
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .flex_1()
                            .min_w_0()
                            .child(
                                div()
                                    .debug_selector(|| "dialog-title".into())
                                    .when(action == WorkspaceAction::OpenWorktree, |title| {
                                        title.truncate()
                                    })
                                    .text_size(px(font.size * 1.35))
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .child(title),
                            )
                            .child(
                                div()
                                    .truncate()
                                    .text_color(rgb(theme.muted))
                                    .child(target.label.clone()),
                            ),
                    )
                    .when(action == WorkspaceAction::OpenWorktree, |header| {
                        header.child(
                            div()
                                .id("open-worktree-escape")
                                .debug_selector(|| "open-worktree-escape".into())
                                .flex_none()
                                .px_2()
                                .py_1()
                                .rounded(px(crate::config::corners::CONTROL))
                                .cursor_pointer()
                                .text_color(rgb(theme.muted))
                                .hover(|button| {
                                    button
                                        .bg(rgb(theme.active))
                                        .text_color(rgb(theme.foreground))
                                })
                                .child("ESC")
                                .on_click(cx.listener(|this, _, window, cx| {
                                    cx.stop_propagation();
                                    this.dismiss_menu(window, cx);
                                })),
                        )
                    }),
            )
            .child(if action == WorkspaceAction::OpenWorktree {
                self.render_existing_worktrees(cx)
                    .when(creating || self.menu.error.is_some(), |picker| {
                        picker.child(
                            div()
                                .id("open-worktree-status")
                                .debug_selector(|| "open-worktree-status".into())
                                .flex_none()
                                .h(px(font.line_height() * 3. + 20.))
                                .overflow_y_scroll()
                                .child(body),
                        )
                    })
                    .into_any_element()
            } else {
                body.into_any_element()
            })
            .child(
                div()
                    .debug_selector(|| "dialog-footer".into())
                    .flex()
                    .justify_end()
                    .gap(px(8.))
                    .px(px(16.))
                    .py(px(12.))
                    .border_t_1()
                    .border_color(rgb(theme.active))
                    .child(
                        button("dialog-cancel")
                            .role(Role::Button)
                            .aria_label(fl!("workspace-dialog-cancel"))
                            .border_color(rgb(theme.active))
                            .hover(|button| button.bg(rgb(theme.active)))
                            .child(fl!("workspace-dialog-cancel"))
                            .on_click(cx.listener(|this, _, window, cx| {
                                cx.stop_propagation();
                                this.dismiss_menu(window, cx);
                            })),
                    )
                    // GitHub rows create their own checkouts without a submit button.
                    .when(
                        !listing || action == WorkspaceAction::OpenWorktree,
                        |footer| {
                            let spoken = if creating {
                                SharedString::from(fl!("workspace-dialog-waiting-spoken"))
                            } else {
                                submit.clone()
                            };
                            let submit_button = crate::accessibility::disabled(
                                button("dialog-submit")
                                    .role(Role::DefaultButton)
                                    .aria_label(spoken),
                                !armed,
                            );
                            footer.child(
                                submit_button
                                    .border_color(if armed {
                                        rgb(theme.foreground)
                                    } else {
                                        rgb(theme.active)
                                    })
                                    .when(armed, |button| button.bg(rgb(theme.active)))
                                    .text_color(if armed {
                                        rgb(theme.foreground)
                                    } else {
                                        rgb(theme.muted)
                                    })
                                    .child(if creating {
                                        fl!("workspace-dialog-waiting").into()
                                    } else {
                                        submit
                                    })
                                    // Faded and inert until there is something
                                    // to submit, so it never reads as pressable.
                                    .when(!armed, |button| {
                                        button.opacity(0.4).cursor(CursorStyle::OperationNotAllowed)
                                    })
                                    .when(armed, |button| {
                                        button
                                            .hover(|button| button.bg(rgb(theme.active)))
                                            .on_click(cx.listener(|this, _, window, cx| {
                                                cx.stop_propagation();
                                                this.submit_workspace_dialog(window, cx);
                                            }))
                                    }),
                            )
                        },
                    ),
            )
    }
}
