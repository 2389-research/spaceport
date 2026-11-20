// ABOUTME: Shares the device rename dialog between live fields and inert departure paint.
// ABOUTME: Keeps accepted rename work outside disposable popup ownership.
//! The page that names a device.
use super::*;

pub(super) struct Paint {
    target: String,
    input: Option<crate::search_input::Paint>,
    submitted: Option<String>,
    error: Option<String>,
    ready: bool,
}

impl Paint {
    pub(super) fn new(host: &HostMenu, cx: &App) -> Self {
        Self {
            target: format!("{} · {}", host.target, host.session), // i18n: data
            input: host.input.as_ref().map(|input| input.read(cx).paint()),
            submitted: host
                .renaming
                .then(|| {
                    host.input
                        .as_ref()
                        .map(|input| input.read(cx).text().to_owned())
                })
                .flatten(),
            error: host.error.clone(),
            ready: !host.renaming,
        }
    }
}

impl SheprdWindow {
    /// A centered modal like Add Device: header with the title and Close,
    /// the name field in the body, and the action in the footer.
    pub(super) fn render_rename_device(
        &self,
        paint: &Paint,
        active: bool,
        cx: &mut Context<Self>,
    ) -> Div {
        let theme = &self.theme;
        let header = div()
            .debug_selector(|| "rename-device-header".into())
            .flex_none()
            .p(px(16.))
            .border_b_1()
            .border_color(rgb(theme.active))
            .flex()
            .items_center()
            .gap(px(12.))
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .text_size(px(self.config.ui.size * 1.35))
                    .font_weight(FontWeight::SEMIBOLD)
                    .child(fl!("host-menu-rename-title")),
            )
            .child(
                div()
                    .id("rename-device-close")
                    .debug_selector(|| "rename-device-close".into())
                    .px_2()
                    .py_1()
                    .rounded(px(crate::config::corners::CONTROL))
                    .child(fl!("host-menu-close"))
                    .when(active, |button| {
                        button
                            .cursor_pointer()
                            .hover(|s| s.bg(rgb(theme.active)))
                            .on_click(
                                cx.listener(|this, _, window, cx| this.dismiss_menu(window, cx)),
                            )
                    }),
            );
        let body = div()
            .debug_selector(|| "rename-device".into())
            .min_h_0()
            .p(px(16.))
            .flex()
            .flex_col()
            .gap(px(12.))
            .child(
                div()
                    .text_color(rgb(theme.muted))
                    .child(paint.target.clone()),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap(px(6.))
                    .child(fl!("host-menu-rename-name"))
                    .when_some(paint.input.as_ref(), |field, input| {
                        if let Some(text) = &paint.submitted {
                            field.child(div().child(text.clone()))
                        } else if active {
                            field.when_some(
                                self.menu.host.as_ref().and_then(|host| host.input.clone()),
                                |field, input| field.child(input),
                            )
                        } else {
                            field.child(input.render())
                        }
                    }),
            )
            .when_some(paint.error.clone(), |body, error| {
                body.child(div().text_color(crate::menu::danger(theme)).child(error))
            });
        let ready = paint.ready;
        let footer =
            div()
                .debug_selector(|| "rename-device-footer".into())
                .flex_none()
                .p(px(16.))
                .border_t_1()
                .border_color(rgb(theme.active))
                .flex()
                .justify_end()
                .child(
                    div()
                        .id("rename-device-submit")
                        .debug_selector(|| "rename-device-submit".into())
                        .p(px(8.))
                        .rounded(px(crate::config::corners::CONTROL))
                        .bg(rgb(theme.active))
                        .when(active && ready, |button| {
                            button.cursor_pointer().hover(|s| {
                                s.bg(rgb(theme.active).blend(rgba((theme.foreground << 8) | 0x20)))
                            })
                        })
                        .when(!ready, |button| button.text_color(rgb(theme.muted)))
                        .child(if ready {
                            fl!("host-menu-rename-submit")
                        } else {
                            fl!("host-menu-renaming")
                        })
                        .when(active, |button| {
                            button.on_click(cx.listener(|this, _, window, cx| {
                                this.submit_rename_device(window, cx)
                            }))
                        }),
                );
        div()
            .flex()
            .flex_col()
            .min_h_0()
            .child(header)
            .child(body)
            .child(footer)
    }
}
