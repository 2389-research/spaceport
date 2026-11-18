// ABOUTME: Shares host forward rows and the port dialog with inert departure paint.
// ABOUTME: Keeps exact tunnel ownership in the window's existing forward manager.
//! Forwarding a saved device's ports from its menu: the dialog that names a
//! port, and the host menu's list of forwards with their actions. The forwards
//! themselves live in [`crate::port_forward`].
use super::*;
use crate::i18n::fl;
use crate::{SheprdWindow, menu::Page};
use std::num::NonZeroU16;

pub(super) struct Row {
    remote_port: NonZeroU16,
    status: String,
    ended: Option<String>,
    open: bool,
}

pub(super) struct Paint {
    label: String,
    input: Option<crate::search_input::Paint>,
    error: Option<String>,
}

impl Paint {
    pub(super) fn new(host: &HostMenu, cx: &App) -> Self {
        Self {
            label: host.label.clone(),
            input: host.input.as_ref().map(|input| input.read(cx).paint()),
            error: host.error.clone(),
        }
    }
}

impl SheprdWindow {
    /// Starts the forward and returns to the host's menu, which shows it
    /// connecting; a refused port keeps the dialog open and says why.
    pub(super) fn submit_forward_port(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(host) = &mut self.menu.host else {
            return;
        };
        let Some(input) = &host.input else {
            return;
        };
        if input.read(cx).is_composing() {
            return;
        }
        let Some(target) = host.ssh_target().map(str::to_owned) else {
            return;
        };
        let started = crate::port_forward::parse_port(input.read(cx).text())
            .and_then(|port| self.port_forwards.start(&target, port));
        match started {
            Ok(()) => {
                host.input = None;
                host.error = None;
                host.selected = None;
                self.menu.page = Some(Page::Host);
                window.focus(&self.menu.focus, cx);
            }
            Err(error) => host.error = Some(error.to_string()),
        }
        cx.notify();
    }

    /// Whether the open host menu has forwarded ports to list.
    pub(in crate::menu) fn host_menu_lists_forwards(&self) -> bool {
        self.menu
            .host
            .as_ref()
            .and_then(HostMenu::ssh_target)
            .is_some_and(|target| self.port_forwards.for_host(target).next().is_some())
    }

    /// Opens a listening forward with the system URL handler.
    fn open_forward(
        &mut self,
        remote_port: NonZeroU16,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(target) = self.menu.host.as_ref().and_then(HostMenu::ssh_target) else {
            return;
        };
        let url = self
            .port_forwards
            .for_host(target)
            .find(|forward| forward.remote_port() == remote_port)
            .and_then(crate::port_forward::Forward::url);
        if let Some(url) = url {
            self.dismiss_menu(window, cx);
            cx.open_url(url.as_str());
        }
    }

    /// Stops a forward, or dismisses one that ended.
    fn stop_forward(&mut self, remote_port: NonZeroU16, cx: &mut Context<Self>) {
        if let Some(target) = self.menu.host.as_ref().and_then(HostMenu::ssh_target)
            && self.port_forwards.stop(target, remote_port)
        {
            cx.notify();
        }
    }

    pub(super) fn host_forward_rows(&self, host: &HostMenu) -> Vec<Row> {
        use crate::port_forward::State;
        host.ssh_target()
            .into_iter()
            .flat_map(|target| self.port_forwards.for_host(target))
            .map(|forward| {
                let (status, ended) = match forward.state() {
                    State::Starting => (fl!("host-menu-forward-connecting"), None),
                    State::Listening { local_port } => (format!("→ 127.0.0.1:{local_port}"), None),
                    State::Ended(reason) => (fl!("host-menu-forward-ended"), Some(reason.clone())),
                };
                Row {
                    remote_port: forward.remote_port(),
                    status,
                    ended,
                    open: forward.url().is_some(),
                }
            })
            .collect()
    }

    /// The host's forwarded ports, each with its state and the actions it
    /// allows. Nothing at all when there are none.
    pub(super) fn render_forwards(
        &self,
        rows: &[Row],
        active: bool,
        cx: &mut Context<Self>,
    ) -> Div {
        let theme = &self.theme;
        let small = px(self.config.ui.size * 0.85);
        let mut section = div().debug_selector(|| "host-forwards".into());
        let mut any = false;
        for forward in rows {
            if !any {
                section = section
                    .mt(px(4.))
                    .pt(px(4.))
                    .border_t_1()
                    .border_color(rgb(theme.active))
                    .child(
                        div()
                            .px(px(8.))
                            .py(px(4.))
                            .text_size(small)
                            .text_color(rgb(theme.muted))
                            .child(fl!("host-menu-forwards")),
                    );
                any = true;
            }
            let remote_port = forward.remote_port;
            let port = remote_port.get();
            let ended = forward.ended.clone();
            let status = forward.status.clone();
            let button = |id: &'static str, label: String| {
                div()
                    .id((id, usize::from(port)))
                    .debug_selector(move || format!("{id}-{port}"))
                    .flex_none()
                    .px(px(6.))
                    .py(px(2.))
                    .rounded(px(crate::config::corners::CONTROL))
                    .when(active, |button| {
                        button
                            .cursor_pointer()
                            .hover(|button| button.bg(rgb(theme.active)))
                    })
                    .child(label)
            };
            let row = div()
                .debug_selector(move || format!("host-forward-{port}"))
                .min_h(px(self.config.ui.line_height() + 8.))
                .px(px(8.))
                .flex()
                .items_center()
                .gap(px(4.))
                .child(
                    div()
                        .flex_1()
                        .min_w_0()
                        .truncate()
                        .child(port.to_string())
                        .child(
                            div()
                                .text_size(small)
                                .text_color(if ended.is_some() {
                                    crate::menu::danger(theme)
                                } else {
                                    rgb(theme.muted)
                                })
                                .child(status),
                        ),
                )
                .when(forward.open, |row| {
                    row.child(
                        button("host-forward-open", fl!("host-menu-forward-open")).when(
                            active,
                            |button| {
                                button.on_click(cx.listener(move |this, _, window, cx| {
                                    this.open_forward(remote_port, window, cx)
                                }))
                            },
                        ),
                    )
                })
                .child(
                    button(
                        "host-forward-stop",
                        if ended.is_some() {
                            fl!("host-menu-forward-dismiss")
                        } else {
                            fl!("host-menu-forward-stop")
                        },
                    )
                    .when(active, |button| {
                        button.on_click(
                            cx.listener(move |this, _, _, cx| this.stop_forward(remote_port, cx)),
                        )
                    }),
                );
            section = section.child(row).when_some(ended, |section, reason| {
                section.child(
                    div()
                        .px(px(8.))
                        .pb(px(4.))
                        .text_size(small)
                        .text_color(rgb(theme.muted))
                        .child(reason),
                )
            });
        }
        section
    }

    /// A centered modal like Rename: the remote port in the body, Forward in
    /// the footer.
    pub(super) fn render_forward_port(
        &self,
        paint: &Paint,
        active: bool,
        cx: &mut Context<Self>,
    ) -> Div {
        let theme = &self.theme;
        let header = div()
            .debug_selector(|| "forward-port-header".into())
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
                    .child(fl!("host-menu-forward-title")),
            )
            .child(
                div()
                    .id("forward-port-close")
                    .debug_selector(|| "forward-port-close".into())
                    .px_2()
                    .py_1()
                    .rounded(px(crate::config::corners::CONTROL))
                    .when(active, |button| {
                        button.cursor_pointer().hover(|s| s.bg(rgb(theme.active)))
                    })
                    .child(fl!("host-menu-forward-close"))
                    .when(active, |button| {
                        button.on_click(
                            cx.listener(|this, _, window, cx| this.dismiss_menu(window, cx)),
                        )
                    }),
            );
        let body = div()
            .debug_selector(|| "forward-port".into())
            .min_h_0()
            .p(px(16.))
            .flex()
            .flex_col()
            .gap(px(12.))
            .child(div().text_color(rgb(theme.muted)).child(fl!(
                "host-menu-forward-intro",
                device = paint.label.as_str()
            )))
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap(px(6.))
                    .child(fl!("host-menu-forward-remote-port"))
                    .when_some(paint.input.as_ref(), |field, input| {
                        if active {
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
                body.child(
                    div()
                        .debug_selector(|| "forward-port-error".into())
                        .text_color(crate::menu::danger(theme))
                        .child(error),
                )
            });
        let footer = div()
            .flex_none()
            .p(px(16.))
            .border_t_1()
            .border_color(rgb(theme.active))
            .flex()
            .justify_end()
            .child(
                div()
                    .id("forward-port-submit")
                    .debug_selector(|| "forward-port-submit".into())
                    .p(px(8.))
                    .rounded(px(crate::config::corners::CONTROL))
                    .bg(rgb(theme.active))
                    .when(active, |button| {
                        button.cursor_pointer().hover(|s| {
                            s.bg(rgb(theme.active).blend(rgba((theme.foreground << 8) | 0x20)))
                        })
                    })
                    .child(fl!("host-menu-forward-submit"))
                    .when(active, |button| {
                        button.on_click(
                            cx.listener(|this, _, window, cx| this.submit_forward_port(window, cx)),
                        )
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
