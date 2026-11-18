// ABOUTME: Shares session row display between live picker controls and passive capture.
// ABOUTME: Keeps exact session targets out of captured row and note values.
use super::*;
use crate::direction::Logical;

pub(in crate::menu) struct SessionPaint {
    row: usize,
    selected: bool,
    spec: Spec,
    removable: bool,
    deletion_color: Rgba,
    remaining: Option<f32>,
}

pub(in crate::menu) struct NotePaint {
    selector: String,
    text: String,
    danger: bool,
}

impl SheprdWindow {
    pub(in crate::menu) fn sessions_row(
        &self,
        row: usize,
        ordinal: usize,
        choice: Row,
        selected: bool,
        spec: &Spec,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let paint = self.session_row_paint(row, &choice, selected, spec);
        self.session_row_content(&paint, Some((ordinal, choice)), cx)
    }

    pub(in crate::menu) fn session_row_paint(
        &self,
        row: usize,
        choice: &Row,
        selected: bool,
        spec: &Spec,
    ) -> SessionPaint {
        let theme = &self.theme;
        let matches_target = |target: &ConnectTarget| match (choice, target) {
            (Row::Local(name), ConnectTarget::Session { name: deleted, .. }) => name == deleted,
            (Row::Device { id, session }, target) if target.is_remote() => {
                target.remote_session() == Some(session.as_str())
                    && self.endpoints.iter().any(|endpoint| {
                        endpoint.id == *id
                            && endpoint.connection.target.remote_host() == target.remote_host()
                    })
            }
            _ => false,
        };
        let deleting = self
            .sessions
            .mutation_target
            .as_ref()
            .is_some_and(matches_target);
        let remaining = self
            .sessions
            .departure
            .as_ref()
            .filter(|departure| matches_target(&departure.target))
            .map(|departure| departure.remaining(crate::motion::now()));
        let removable = !matches!(choice, Row::Add(_));
        let deletion_color = if self.session_delete_reason(choice).is_none() {
            colors::danger(theme)
        } else if selected || spec.checked {
            rgb(theme.foreground)
        } else {
            rgb(theme.muted)
        };
        let mut spec = spec.clone();
        if deleting {
            spec.detail = fl!("sessions-menu-deleting").into();
        }
        SessionPaint {
            row,
            selected,
            spec,
            removable,
            deletion_color,
            remaining,
        }
    }

    pub(in crate::menu) fn render_session_row_paint(
        &self,
        paint: &SessionPaint,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        self.session_row_content(paint, None, cx)
    }

    fn session_row_content(
        &self,
        paint: &SessionPaint,
        action: Option<(usize, Row)>,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let theme = &self.theme;
        let font = &self.config.ui;
        let row = paint.row;
        let selected = paint.selected;
        let spec = &paint.spec;
        let removable = paint.removable;
        let deletion_color = paint.deletion_color;
        let remaining = paint.remaining;
        let active = action.is_some();
        let removal = action
            .as_ref()
            .map(|(_, choice)| (choice.clone(), self.management_target(choice)));
        // Two lines plus padding; a wrapped note makes the row taller.
        let height = font.line_height() * 2. + 16.;
        let view = div()
            .id(("session-entry", row))
            .debug_selector(move || format!("sessions-row-{row}"))
            .p(px(8.))
            .min_h(px(height))
            .line_height(px(font.line_height()))
            .rounded(px(crate::config::corners::CONTROL))
            .flex_none()
            .flex()
            .items_center()
            .gap(px(8.))
            .when(selected, |s| s.bg(rgb(theme.active)))
            .when(spec.checked, |s| s.bg(rgb(theme.primary_wash())))
            .when(active && spec.enabled, |s| s.cursor_pointer())
            .text_color(rgb(if spec.enabled {
                theme.foreground
            } else {
                theme.muted
            }))
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .child(
                        div()
                            .min_w_0()
                            .truncate()
                            .when(spec.checked, |label| {
                                label
                                    .debug_selector(move || format!("sessions-current-{row}"))
                                    .font_weight(FontWeight::SEMIBOLD)
                            })
                            .child(spec.label.clone()),
                    )
                    .child(
                        // Wraps rather than cutting a note short: the popup's
                        // width is the device list's, not the note's.
                        div()
                            .debug_selector(move || format!("sessions-detail-{row}"))
                            .min_w_0()
                            .text_size(px(font.size * 0.85))
                            .text_color(rgb(theme.muted))
                            .child(spec.detail.clone()),
                    ),
            )
            .when(removable, |container| {
                container.child(
                    div()
                        .debug_selector(move || format!("sessions-dot-{row}"))
                        .size(px(7.))
                        .flex_none()
                        .rounded_full()
                        .bg(rgb(if spec.online {
                            colors::online(theme)
                        } else {
                            theme.muted
                        })),
                )
            })
            .when(removable, |container| {
                container.child(
                    action_icon(
                        "icons/trash.svg",
                        format!("sessions-delete-icon-{row}"),
                        deletion_color,
                        rgb(theme.foreground),
                        active,
                    )
                    .id(("session-delete", row))
                    .debug_selector(move || format!("sessions-delete-{row}"))
                    .when_some(removal, |button, (removal, removal_target)| {
                        button
                            .hover(|s| s.bg(colors::action_hover(theme)))
                            .on_click(cx.listener(move |this, _, window, cx| {
                                cx.stop_propagation();
                                this.confirm_session_delete(
                                    removal.clone(),
                                    removal_target.clone(),
                                    window,
                                    cx,
                                );
                            }))
                    }),
                )
            })
            // Sits in the trash column; the whole row is the click target.
            .when(!removable, |container| {
                let color = rgb(if spec.enabled {
                    theme.foreground
                } else {
                    theme.muted
                });
                container.child(action_icon(
                    "icons/plus.svg",
                    format!("sessions-add-icon-{row}"),
                    color,
                    color,
                    active,
                ))
            })
            .when_some(action, |view, (ordinal, choice)| {
                view.on_hover(cx.listener(move |this, hovered, _, cx| {
                    if *hovered {
                        this.menu.selected = Some(ordinal);
                        cx.notify();
                    }
                }))
                .on_click(cx.listener(move |this, _, window, cx| {
                    // Keep the painted identity even if the catalog refreshes.
                    this.choose_session(choice.clone(), window, cx);
                }))
            });
        if let Some(remaining) = remaining {
            div()
                .debug_selector(move || format!("sessions-departing-{row}"))
                // Keep the anchored popup's height and every row position stable
                // until the fade finishes; then remove the row in one update.
                // The row sizes the wrapper, so a wrapped note keeps its height.
                .flex_none()
                .opacity(remaining)
                .child(view)
                .into_any_element()
        } else {
            view.into_any_element()
        }
    }

    /// A note under an unfolded device: why its list is empty or partial, or
    /// what went wrong. `None` for a row, which [`Self::sessions_row`] paints.
    pub(in crate::menu) fn session_note_paint(&self, entry: &Entry) -> Option<NotePaint> {
        match entry {
            Entry::Muted { selector, text } => Some(NotePaint {
                selector: selector.clone(),
                text: text.clone(),
                danger: false,
            }),
            Entry::Error(text) => Some(NotePaint {
                selector: "sessions-error".into(),
                text: text.clone(),
                danger: true,
            }),
            Entry::Row { .. } => None,
        }
    }

    pub(in crate::menu) fn session_note(&self, entry: &Entry) -> Option<AnyElement> {
        self.session_note_paint(entry)
            .map(|paint| self.render_session_note_paint(&paint))
    }

    pub(in crate::menu) fn render_session_note_paint(&self, paint: &NotePaint) -> AnyElement {
        let selector = paint.selector.clone();
        div()
            .px(px(8.))
            .py(px(2.))
            .text_size(px(self.config.ui.size * 0.85))
            .debug_selector(move || selector.clone())
            .text_color(if paint.danger {
                colors::danger(&self.theme)
            } else {
                rgb(self.theme.muted)
            })
            .child(paint.text.clone())
            .into_any_element()
    }
}

/// Shared trailing action geometry for session row controls.
fn action_icon(
    path: &'static str,
    selector: String,
    color: Rgba,
    hover_color: Rgba,
    active: bool,
) -> Div {
    div()
        .group("menu-action-icon")
        .size(px(24.))
        .flex_none()
        .row()
        .items_center()
        .justify_center()
        .rounded(px(crate::config::corners::CONTROL))
        .child(
            svg()
                .path(path)
                .size(px(14.))
                // GPUI skips SVG painting without an explicit SVG color.
                // The parent's text color is not inherited by SVG painting.
                .text_color(color)
                .when(active, |icon| {
                    icon.group_hover("menu-action-icon", move |style| {
                        style.text_color(hover_color)
                    })
                })
                .debug_selector(move || selector.clone()),
        )
}
