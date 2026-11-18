// ABOUTME: Shares active and departing pane display templates.
// ABOUTME: Keeps display code separate from target and worker ownership.
use super::paint::{Draft, Kind};
use super::*;

impl SheprdWindow {
    pub(crate) fn render_pane_menu(&self, cx: &mut Context<Self>) -> Div {
        let Some(paint) = self.pane_menu_paint(cx) else {
            return div();
        };
        self.render_pane_content(&paint, self.menu.pane.as_ref(), cx)
    }

    pub(crate) fn render_pane_menu_paint(&self, paint: &Paint, cx: &mut Context<Self>) -> Div {
        self.render_pane_content(paint, None, cx)
            .relative()
            .top(paint.offset.y)
    }

    fn render_pane_content(
        &self,
        paint: &Paint,
        live: Option<&PaneMenu>,
        cx: &mut Context<Self>,
    ) -> Div {
        match &paint.kind {
            Kind::Processes(paint) => {
                return self.render_pane_processes(
                    paint,
                    live.and_then(|pane| pane.processes.as_ref()),
                    cx,
                );
            }
            Kind::Confirmation(paint) => {
                return self.render_kill_processes(paint, live.is_some(), cx);
            }
            _ => {}
        }
        let mut body = div().flex().flex_col();
        if matches!(paint.kind, Kind::Moving) {
            body = self.render_pane_move_pending();
        } else if let Kind::Actions(labels) = &paint.kind {
            let actions = live.map(PaneMenu::actions);
            for (index, label) in labels.iter().enumerate() {
                let action = actions
                    .as_ref()
                    .and_then(|actions| actions.get(index))
                    .copied();
                body = body.child(
                    div()
                        .id(("pane-menu-action", index))
                        .debug_selector(move || format!("pane-menu-{index}"))
                        .min_h(px(self.config.ui.line_height() + 12.))
                        .px(px(8.))
                        .row()
                        .items_center()
                        .when(live.is_some(), |row| row.cursor_pointer())
                        .when(paint.selected == Some(index), |row| {
                            row.bg(rgb(self.theme.active))
                        })
                        .child(
                            // One line: the panel widens to fit it, and a
                            // window too narrow for that ends it with "…".
                            div()
                                .debug_selector(move || format!("pane-menu-label-{index}"))
                                .min_w_0()
                                .truncate()
                                .child(label.clone()),
                        )
                        .when_some(action, |row, action| {
                            row.hover(|row| row.bg(rgb(self.theme.active)))
                                .on_hover(cx.listener(move |this, hovered, _, cx| {
                                    if *hovered && let Some(pane) = &mut this.menu.pane {
                                        pane.selected = Some(index);
                                        cx.notify();
                                    }
                                }))
                                .on_click(cx.listener(move |this, _, window, cx| {
                                    this.activate_pane_menu(action, window, cx)
                                }))
                        }),
                );
            }
        } else if let Kind::Rename(draft) = &paint.kind {
            let pending = matches!(draft, Draft::Pending(_));
            let input = match draft {
                Draft::Pending(text) => text
                    .as_ref()
                    .map(|text| div().child(text.clone()).into_any_element()),
                Draft::Editing(field) => live
                    .and_then(|pane| pane.input.as_ref())
                    .map(|input| input.clone().into_any_element())
                    .or_else(|| {
                        field
                            .as_deref()
                            .map(|field| field.render().into_any_element())
                    }),
            };
            body = body
                .p(px(8.))
                .gap(px(12.))
                .child(
                    div()
                        .font_weight(FontWeight::SEMIBOLD)
                        .child(fl!("pane-menu-rename-title")),
                )
                .child(
                    div()
                        .text_color(rgb(self.theme.subtext()))
                        .child(fl!("pane-menu-rename-hint")),
                )
                .when_some(input, |body, input| body.child(input))
                .child(
                    div()
                        .row()
                        .justify_end()
                        .gap(px(8.))
                        .child(
                            div()
                                .id("pane-rename-cancel")
                                .debug_selector(|| "pane-rename-cancel".into())
                                .p(px(6.))
                                .child(fl!("pane-menu-cancel"))
                                .when(live.is_some(), |button| {
                                    button.cursor_pointer().on_click(cx.listener(
                                        |this, _, window, cx| this.dismiss_menu(window, cx),
                                    ))
                                }),
                        )
                        .child(
                            div()
                                .id("pane-rename-submit")
                                .debug_selector(|| "pane-rename-submit".into())
                                .p(px(6.))
                                .rounded(px(crate::config::corners::CONTROL))
                                .bg(rgb(self.theme.active))
                                .child(if pending {
                                    fl!("pane-menu-renaming")
                                } else {
                                    fl!("pane-menu-rename-submit")
                                })
                                .when(live.is_some(), |button| {
                                    button.cursor_pointer().on_click(cx.listener(
                                        |this, _, window, cx| this.submit_pane_rename(window, cx),
                                    ))
                                }),
                        ),
                );
        }
        body.when_some(paint.error.clone(), |body, error| {
            // No width of its own, so the labels set the panel's width and
            // the message wraps inside it.
            body.child(div().w_0().min_w_full().p(px(8.)).child(error))
        })
    }
}
