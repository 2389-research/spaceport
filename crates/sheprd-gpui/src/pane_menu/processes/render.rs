// ABOUTME: Shares active and departing pane display templates.
// ABOUTME: Keeps display code separate from target and worker ownership.
use super::*;

impl SheprdWindow {
    pub(in crate::pane_menu) fn render_pane_processes(
        &self,
        paint: &Paint,
        live: Option<&PaneProcesses>,
        cx: &mut Context<Self>,
    ) -> Div {
        let theme = &self.theme;
        let muted = rgb(theme.muted);
        let mut body = div().p(px(8.)).flex().flex_col().gap(px(8.)).child(
            div()
                .row()
                .items_baseline()
                .gap(px(8.))
                .child(
                    div()
                        .flex_1()
                        .min_w_0()
                        .truncate()
                        .font_weight(FontWeight::SEMIBOLD)
                        .child(paint.title.clone()),
                )
                .when_some(paint.total.clone(), |header, total| {
                    header.child(
                        div()
                            .debug_selector(|| "pane-processes-total".into())
                            .flex_none()
                            .text_color(muted)
                            .child(total),
                    )
                }),
        );
        let mut list = div()
            .id("pane-processes-list")
            .debug_selector(|| "pane-processes-list".into())
            .flex()
            .flex_col()
            .max_h(px(360.))
            .when(live.is_some(), |list| list.overflow_y_scroll())
            .when(live.is_none(), |list| list.overflow_hidden())
            .when_some(live, |list, live| list.track_scroll(&live.scroll));
        let mut children = Vec::new();
        match &paint.rows {
            None if paint.loading => {
                children.push(
                    div()
                        .text_color(muted)
                        .child(fl!("pane-menu-processes-listing"))
                        .into_any_element(),
                );
            }
            None => {}
            Some(rows) => {
                for (index, row) in rows.iter().enumerate() {
                    let identity = live
                        .and_then(|live| live.listing.as_ref())
                        .and_then(|listing| listing.rows.get(index))
                        .map(|row| row.entry.identity);
                    let chosen = row.chosen;
                    let choosable = row.choosable;
                    children.push(
                        div()
                            .id(("pane-process", index))
                            .debug_selector(move || format!("pane-process-{index}"))
                            .min_h(px(self.config.ui.line_height() + 8.))
                            .px(px(6.))
                            .row()
                            .items_center()
                            .gap(px(8.))
                            .rounded(px(crate::config::corners::CONTROL))
                            .when(paint.cursor == Some(index), |row| row.bg(rgb(theme.active)))
                            .when_some(identity.filter(|_| choosable), |row, identity| {
                                row.cursor_pointer()
                                    .hover(|row| row.bg(rgb(theme.active)))
                                    .on_click(cx.listener(move |this, _, _, cx| {
                                        this.toggle_pane_process(identity, cx)
                                    }))
                            })
                            .child(
                                // The pane's own process ends with the pane.
                                div()
                                    .size(px(12.))
                                    .flex_none()
                                    .rounded(px(3.))
                                    .when(choosable, |mark| {
                                        mark.border_1().border_color(if chosen {
                                            crate::menu::danger(theme)
                                        } else {
                                            muted
                                        })
                                    })
                                    .when(chosen, |mark| mark.bg(crate::menu::danger(theme))),
                            )
                            .child(
                                div()
                                    .flex_1()
                                    .min_w_0()
                                    .ps(px(12. * row.depth.min(8) as f32))
                                    .row()
                                    .gap(px(6.))
                                    .overflow_hidden()
                                    .child(div().flex_none().child(row.name.clone()))
                                    .when(row.foreground, |line| {
                                        line.child(
                                            div()
                                                .flex_none()
                                                .text_color(crate::menu::accent(theme))
                                                .child(fl!("pane-menu-processes-foreground")),
                                        )
                                    })
                                    .child(
                                        div()
                                            .min_w_0()
                                            .truncate()
                                            .text_color(muted)
                                            .child(row.command.clone()),
                                    ),
                            )
                            .child(
                                div()
                                    .w(px(44.))
                                    .flex_none()
                                    .text_end()
                                    .text_color(muted)
                                    .child(row.pid.clone()),
                            )
                            .child(
                                div()
                                    .w(px(48.))
                                    .flex_none()
                                    .text_end()
                                    .child(row.cpu.clone()),
                            )
                            .child(
                                div()
                                    .w(px(64.))
                                    .flex_none()
                                    .text_end()
                                    .child(row.memory.clone()),
                            )
                            .into_any_element(),
                    );
                }
                if let Some(hidden) = &paint.hidden {
                    children.push(
                        div()
                            .px(px(6.))
                            .text_color(muted)
                            .child(hidden.clone())
                            .into_any_element(),
                    );
                }
            }
        }
        list = match live {
            Some(_) => list.children(children),
            None => list.child(
                div()
                    .flex_none()
                    .flex()
                    .flex_col()
                    .relative()
                    .top(paint.offset.y)
                    .children(children),
            ),
        };
        body = body.child(list);
        body.when_some(paint.status.clone(), |body, status| {
            body.child(
                div()
                    .debug_selector(|| "pane-processes-status".into())
                    .text_color(muted)
                    .child(status),
            )
        })
        .child(
            div().row().justify_end().child(
                div()
                    .id("pane-processes-end")
                    .debug_selector(|| "pane-processes-end".into())
                    .px(px(12.))
                    .py(px(6.))
                    .rounded(px(crate::config::corners::CONTROL))
                    .when(paint.enabled, |button| button.bg(rgb(theme.active)))
                    .when(paint.enabled && live.is_some(), |button| {
                        button
                            .cursor_pointer()
                            .on_click(cx.listener(|this, _, _, cx| this.confirm_pane_processes(cx)))
                    })
                    .when(!paint.enabled, |button| button.text_color(muted))
                    .child(paint.end.clone()),
            ),
        )
    }

    pub(in crate::pane_menu) fn render_kill_processes(
        &self,
        paint: &Confirmation,
        active: bool,
        cx: &mut Context<Self>,
    ) -> Div {
        let theme = &self.theme;
        div()
            .p(px(12.))
            .flex()
            .flex_col()
            .gap(px(12.))
            .child(
                div()
                    .text_size(px(self.config.ui.size * 1.35))
                    .font_weight(FontWeight::SEMIBOLD)
                    .child(paint.title.clone()),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .children(paint.rows.iter().map(|(name, pid)| {
                        div()
                            .row()
                            .gap(px(8.))
                            .child(div().flex_none().child(name.clone()))
                            .child(
                                div()
                                    .flex_none()
                                    .text_color(rgb(theme.muted))
                                    .child(pid.clone()),
                            )
                    })),
            )
            .child(
                div()
                    .text_color(rgb(theme.subtext()))
                    .child(fl!("pane-menu-processes-confirm-note")),
            )
            .child(
                div()
                    .row()
                    .justify_end()
                    .gap(px(8.))
                    .child(
                        div()
                            .id("pane-processes-cancel")
                            .debug_selector(|| "pane-processes-cancel".into())
                            .px(px(12.))
                            .py(px(6.))
                            .rounded(px(crate::config::corners::CONTROL))
                            .border_1()
                            .border_color(rgb(theme.active))
                            .child(fl!("pane-menu-processes-cancel"))
                            .when(active, |button| {
                                button
                                    .cursor_pointer()
                                    .hover(|s| s.bg(rgb(theme.active)))
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.leave_kill_confirmation();
                                        cx.notify();
                                    }))
                            }),
                    )
                    .child(
                        div()
                            .id("pane-processes-confirm")
                            .debug_selector(|| "pane-processes-confirm".into())
                            .px(px(12.))
                            .py(px(6.))
                            .rounded(px(crate::config::corners::CONTROL))
                            .bg(rgb(theme.active))
                            .text_color(crate::menu::danger(theme))
                            .child(paint.confirm.clone())
                            .when(active, |button| {
                                button.cursor_pointer().on_click(
                                    cx.listener(|this, _, _, cx| this.end_pane_processes(cx)),
                                )
                            }),
                    ),
            )
    }
}
