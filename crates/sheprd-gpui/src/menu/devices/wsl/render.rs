// ABOUTME: Shares WSL setup and removal templates with passive departure copies.
// ABOUTME: Leaves native input, source switching, and accepted effects in active owners.
use super::{paint::Rows, *};

impl SheprdWindow {
    pub(in crate::menu) fn render_add_wsl(&self, compact: bool, cx: &mut Context<Self>) -> Div {
        self.add_wsl_paint(false, cx).map_or_else(div, |paint| {
            self.render_add_wsl_paint(&paint, compact, true, cx)
        })
    }

    pub(in crate::menu::devices) fn render_add_wsl_paint(
        &self,
        paint: &Paint,
        compact: bool,
        active: bool,
        cx: &mut Context<Self>,
    ) -> Div {
        let theme = &self.theme;
        let mut list = div()
            .debug_selector(|| "wsl-setup-distros".into())
            .flex()
            .flex_col()
            .gap(px(2.));
        match &paint.rows {
            Rows::Message { text, danger } => {
                list = list.child(
                    div()
                        .text_color(if *danger {
                            crate::menu::danger(theme)
                        } else {
                            rgb(theme.muted)
                        })
                        .child(text.clone()),
                );
            }
            Rows::Choices {
                rows,
                before,
                after,
            } => {
                list = list.when(*before > px(0.), |list| {
                    list.child(div().flex_none().h(*before))
                });
                for choice in rows {
                    let index = choice.index;
                    let added = choice.added;
                    let checked = choice.checked;
                    let pick = choice.distro.clone();
                    list = list.child(
                        div()
                            .id(("wsl-distro", index))
                            .debug_selector(move || format!("wsl-distro-{index}"))
                            .px(px(8.))
                            .py(px(6.))
                            .rounded(px(crate::config::corners::CONTROL))
                            .flex()
                            .items_center()
                            .gap(px(8.))
                            .when(checked, |row| row.bg(rgb(theme.active)))
                            .when(added, |row| row.text_color(rgb(theme.muted)))
                            .when(active && !added, |row| {
                                row.cursor_pointer().hover(|s| s.bg(rgb(theme.active)))
                            })
                            .child(crate::toggles::radio(
                                theme,
                                self.config.ui.size + 2.,
                                checked,
                            ))
                            .child(div().flex_1().truncate().child(choice.distro.clone()))
                            .when(added, |row| row.child(fl!("devices-wsl-added")))
                            .when(active, |row| {
                                row.on_click(cx.listener(move |this, _, _, cx| {
                                    if added {
                                        return;
                                    }
                                    if let Some(setup) = &mut this.menu.wsl_setup {
                                        setup.selected = Some(pick.clone());
                                        cx.notify();
                                    }
                                }))
                            }),
                    );
                }
                list = list.when(*after > px(0.), |list| {
                    list.child(div().flex_none().h(*after))
                });
            }
        }
        list = list.when(active, |list| {
            list.when_some(self.menu.wsl_setup.as_ref(), |list, setup| {
                let bounds = setup.list_bounds.clone();
                list.child(
                    canvas(move |layout, _, _| bounds.set(layout), |_, _, _, _| {})
                        .absolute()
                        .inset_0(),
                )
            })
        });
        let session = div()
            .flex_none()
            .flex()
            .flex_col()
            .gap(px(6.))
            .child(fl!("devices-wsl-session"));
        let session = if active {
            session.when_some(self.menu.wsl_setup.as_ref(), |row, setup| {
                row.child(setup.session.clone())
            })
        } else {
            session.child(paint.session.render())
        };
        let mut children = vec![
            div()
                .flex_none()
                .text_color(rgb(theme.subtext()))
                .child(fl!("devices-wsl-intro"))
                .into_any_element(),
            list.into_any_element(),
            session.into_any_element(),
        ];
        if let Some(error) = &paint.error {
            children.push(
                div()
                    .debug_selector(|| "wsl-setup-error".into())
                    .flex_none()
                    .text_color(crate::menu::danger(theme))
                    .child(error.clone())
                    .into_any_element(),
            );
        }
        if let Some(status) = &paint.status {
            children.push(
                div()
                    .debug_selector(|| "wsl-setup-status".into())
                    .flex_none()
                    .text_color(rgb(theme.muted))
                    .child(status.clone())
                    .into_any_element(),
            );
        }
        let body = div()
            .id("wsl-setup-body")
            .debug_selector(|| "wsl-setup-body".into())
            .flex_1()
            .min_h_0()
            .p(px(16.))
            .flex()
            .flex_col()
            .gap(px(12.))
            .when(active, |body| {
                body.overflow_y_scroll()
                    .when_some(self.menu.wsl_setup.as_ref(), |body, setup| {
                        body.track_scroll(&setup.scroll)
                    })
            })
            .when(!active, |body| body.overflow_hidden());
        let body = if active {
            body.children(children)
        } else {
            body.child(
                div()
                    .relative()
                    .flex_none()
                    .flex()
                    .flex_col()
                    .gap(px(12.))
                    .top(paint.offset.y)
                    .children(children),
            )
        };
        let footer = div()
            .flex_none()
            .p(px(16.))
            .border_t_1()
            .border_color(rgb(theme.active))
            .flex()
            .justify_end()
            .child(
                div()
                    .id("wsl-setup-submit")
                    .debug_selector(|| "wsl-setup-submit".into())
                    .p(px(8.))
                    .rounded(px(crate::config::corners::CONTROL))
                    .bg(rgb(theme.active))
                    .when(active && paint.ready, |button| {
                        button.cursor_pointer().hover(|s| {
                            s.bg(rgb(theme.active).blend(rgba((theme.foreground << 8) | 0x20)))
                        })
                    })
                    .when(!paint.ready, |button| button.text_color(rgb(theme.muted)))
                    .child(if paint.checking {
                        fl!("devices-wsl-add-checking")
                    } else {
                        fl!("devices-wsl-add")
                    })
                    .when(active, |button| {
                        button.on_click(
                            cx.listener(|this, _, window, cx| this.submit_add_wsl(window, cx)),
                        )
                    }),
            );
        self.add_device_frame(
            &paint.rail,
            compact,
            div()
                .flex_1()
                .min_h_0()
                .flex()
                .flex_col()
                .child(body)
                .child(footer),
            active,
            cx,
        )
    }

    pub(in crate::menu) fn render_remove_wsl(&self, cx: &mut Context<Self>) -> Div {
        self.remove_wsl_paint()
            .map_or_else(div, |paint| self.render_remove_wsl_paint(&paint, true, cx))
    }

    pub(in crate::menu::devices) fn render_remove_wsl_paint(
        &self,
        paint: &RemovalPaint,
        active: bool,
        cx: &mut Context<Self>,
    ) -> Div {
        let theme = &self.theme;
        let body =
            div()
                .debug_selector(|| "remove-wsl".into())
                .flex()
                .flex_col()
                .p(px(8.))
                .gap(px(12.))
                .child(
                    div()
                        .font_weight(FontWeight::SEMIBOLD)
                        .child(fl!("devices-wsl-remove-title")),
                )
                .child(
                    div()
                        .text_color(rgb(theme.subtext()))
                        .child(paint.note.clone()),
                )
                .child(
                    div()
                        .flex()
                        .justify_end()
                        .gap(px(8.))
                        .child(
                            div()
                                .id("remove-wsl-cancel")
                                .debug_selector(|| "remove-wsl-cancel".into())
                                .p(px(6.))
                                .child(fl!("devices-wsl-cancel"))
                                .when(active, |button| {
                                    button.cursor_pointer().on_click(cx.listener(
                                        |this, _, window, cx| this.dismiss_menu(window, cx),
                                    ))
                                }),
                        )
                        .child(
                            div()
                                .id("remove-wsl-submit")
                                .debug_selector(|| "remove-wsl-submit".into())
                                .p(px(6.))
                                .rounded(px(crate::config::corners::CONTROL))
                                .bg(rgb(theme.active))
                                .text_color(crate::menu::danger(theme))
                                .child(fl!("devices-wsl-remove"))
                                .when(active, |button| {
                                    button.cursor_pointer().on_click(cx.listener(
                                        |this, _, window, cx| this.confirm_remove_wsl(window, cx),
                                    ))
                                }),
                        ),
                );
        if active {
            body
        } else {
            div().relative().top(paint.offset.y).child(body)
        }
    }
}
