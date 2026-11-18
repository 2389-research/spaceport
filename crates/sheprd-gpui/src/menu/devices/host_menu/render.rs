// ABOUTME: Shares host action and confirmation templates across active and outgoing copies.
// ABOUTME: Omits effect handlers from captured text and field paint.
use super::{
    paint::{Content, Header, Row},
    *,
};

impl SheprdWindow {
    pub(in crate::menu) fn render_host_menu(&self, cx: &mut Context<Self>) -> Div {
        if self.menu.page == Some(Page::DeviceTheme) {
            return self
                .menu
                .host
                .as_ref()
                .map_or_else(div, |host| self.render_device_theme(host, cx));
        }
        self.prepare_host_paint(false, cx)
            .map_or_else(div, |paint| self.render_host_content(&paint, true, cx))
    }

    pub(in crate::menu) fn render_host_paint(&self, paint: &Paint, cx: &mut Context<Self>) -> Div {
        div()
            .relative()
            .top(paint.offset.y)
            .child(self.render_host_content(paint, false, cx))
    }

    fn render_host_content(&self, paint: &Paint, active: bool, cx: &mut Context<Self>) -> Div {
        match &paint.content {
            Content::Host {
                header,
                rows,
                forwards,
            } => self
                .render_host_actions(header, rows, active, cx)
                .child(self.render_forwards(forwards, active, cx)),
            Content::Rename(paint) => self.render_rename_device(paint, active, cx),
            Content::Forward(paint) => self.render_forward_port(paint, active, cx),
            Content::Tags(paint) => self.render_device_tags(paint, active, cx),
            Content::Remove(note) => self.render_remove_device(note, active, cx),
        }
    }

    fn render_host_actions(
        &self,
        header: &Header,
        rows: &[Row],
        active: bool,
        cx: &mut Context<Self>,
    ) -> Div {
        let theme = &self.theme;
        // Name the device first, so the destructive row below cannot be
        // mistaken for acting on another header.
        let mut body = div().flex().flex_col().child(
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
                        .child(header.label.clone()),
                )
                .when(!header.target.is_empty(), |row| {
                    row.child(
                        div()
                            .text_size(px(self.config.ui.size * 0.85))
                            .text_color(rgb(theme.muted))
                            .truncate()
                            // The device's target and session.
                            .child(format!("{} · {}", header.target, header.session)), // i18n: data
                    )
                }),
        );
        for row in rows {
            let (index, action) = (row.index, row.action);
            body = body.child(
                div()
                    .id(("host-menu-action", index))
                    .debug_selector(move || format!("host-menu-{index}"))
                    .min_h(px(self.config.ui.line_height() + 12.))
                    .px(px(8.))
                    .flex()
                    .items_center()
                    .gap(px(8.))
                    .when(active, |row| row.cursor_pointer())
                    .when_some(row.toggle, |row, checked| {
                        row.child(crate::toggles::checkbox(
                            theme,
                            self.config.ui.size + 2.,
                            checked,
                        ))
                    })
                    .when(action == Action::Remove, |row| {
                        row.text_color(crate::menu::danger(theme))
                    })
                    .when(row.selected, |row| row.bg(rgb(theme.active)))
                    .when(active, |row| row.hover(|row| row.bg(rgb(theme.active))))
                    .child(
                        div()
                            .debug_selector(move || format!("host-menu-label-{index}"))
                            .min_w_0()
                            .truncate()
                            .child(row.label.clone()),
                    )
                    .when_some(row.theme.clone(), |row, name| {
                        row.child(
                            div()
                                .debug_selector(|| "host-menu-theme".into())
                                .ml_auto()
                                .min_w_0()
                                .truncate()
                                .text_color(rgb(theme.muted))
                                .child(name),
                        )
                    })
                    .when(active, |row| {
                        row.on_hover(cx.listener(move |this, hovered, _, cx| {
                            if *hovered && let Some(host) = &mut this.menu.host {
                                host.selected = Some(index);
                                cx.notify();
                            }
                        }))
                        .on_click(cx.listener(
                            move |this, _, window, cx| this.activate_host_menu(action, window, cx),
                        ))
                    }),
            );
            // Opting in never silently changes nothing: say why the
            // device is still on local keys.
            if let Some(error) = &row.error {
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
                        .child(error.clone()),
                );
            }
        }
        body
    }

    fn render_remove_device(&self, note: &str, active: bool, cx: &mut Context<Self>) -> Div {
        let theme = &self.theme;
        div()
            .flex()
            .flex_col()
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
                    .child(note.to_owned()),
            )
            .child(
                div()
                    .flex()
                    .justify_end()
                    .gap(px(8.))
                    .child(
                        div()
                            .id("remove-device-cancel")
                            .debug_selector(|| "remove-device-cancel".into())
                            .p(px(6.))
                            .when(active, |button| button.cursor_pointer())
                            .child(fl!("host-menu-cancel"))
                            .when(active, |button| {
                                button.on_click(cx.listener(|this, _, window, cx| {
                                    tracing::info!(category = "device_remove", "Remove cancelled");
                                    this.dismiss_menu(window, cx)
                                }))
                            }),
                    )
                    .child(
                        div()
                            .id("remove-device-submit")
                            .debug_selector(|| "remove-device-submit".into())
                            .p(px(6.))
                            .rounded(px(crate::config::corners::CONTROL))
                            .bg(rgb(theme.active))
                            .text_color(crate::menu::danger(theme))
                            .when(active, |button| button.cursor_pointer())
                            .child(fl!("host-menu-remove-submit"))
                            .when(active, |button| {
                                button.on_click(cx.listener(|this, _, window, cx| {
                                    this.confirm_remove_device(window, cx)
                                }))
                            }),
                    ),
            )
    }
}
