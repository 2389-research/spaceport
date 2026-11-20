// ABOUTME: Shares Core setup controls with a passive departure template.
// ABOUTME: Leaves native fields and submission callbacks in the active form.
use super::{paint::Field, *};

impl SheprdWindow {
    pub(in crate::menu) fn render_add_device(&self, compact: bool, cx: &mut Context<Self>) -> Div {
        self.add_device_paint(cx).map_or_else(div, |paint| {
            self.render_add_device_paint(&paint, compact, true, cx)
        })
    }

    pub(in crate::menu::devices) fn render_add_device_paint(
        &self,
        paint: &Paint,
        compact: bool,
        active: bool,
        cx: &mut Context<Self>,
    ) -> Div {
        let theme = &self.theme;
        let mut children = vec![
            div()
                .flex_none()
                .text_color(rgb(theme.subtext()))
                .child(fl!("add-device-core-intro"))
                .into_any_element(),
        ];
        for (index, (label, field)) in [
            fl!("add-device-target"),
            fl!("add-device-label"),
            fl!("add-device-session"),
        ]
        .into_iter()
        .zip(&paint.fields)
        .enumerate()
        {
            let row = div()
                .debug_selector(move || format!("device-setup-field-{index}"))
                .flex_none()
                .flex()
                .flex_col()
                .gap(px(6.))
                .child(label);
            children.push(
                match field {
                    Field::Text(text) => row.child(text.clone()),
                    Field::Input(_) if active => row
                        .when_some(self.menu.device_setup.as_ref(), |row, setup| {
                            row.child(setup.fields[index].clone())
                        }),
                    Field::Input(input) => row.child(input.render()),
                }
                .into_any_element(),
            );
        }
        if let Some(error) = &paint.error {
            children.push(
                div()
                    .flex_none()
                    .text_color(crate::menu::danger(theme))
                    .child(error.clone())
                    .into_any_element(),
            );
        }
        if let Some(status) = &paint.status {
            children.push(
                div()
                    .debug_selector(|| "device-setup-status".into())
                    .flex_none()
                    .text_color(rgb(theme.muted))
                    .child(status.clone())
                    .into_any_element(),
            );
        }
        let body = div()
            .id("device-setup-body")
            .debug_selector(|| "device-setup-body".into())
            .flex_1()
            .min_h_0()
            .p(px(16.))
            .flex()
            .flex_col()
            .gap(px(12.))
            .when(active, |body| {
                body.overflow_y_scroll()
                    .when_some(self.menu.device_setup.as_ref(), |body, setup| {
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
        let ready = paint.ready;
        let footer = div()
            .debug_selector(|| "device-setup-footer".into())
            .flex_none()
            .p(px(16.))
            .border_t_1()
            .border_color(rgb(theme.active))
            .flex()
            .justify_end()
            .gap(px(8.))
            .child(
                div()
                    .id("device-setup-submit")
                    .debug_selector(|| "device-setup-submit".into())
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
                        fl!("add-device-submit")
                    } else {
                        fl!("add-device-submit-checking")
                    })
                    .when(active, |button| {
                        button.on_click(
                            cx.listener(|this, _, window, cx| this.submit_core_device(window, cx)),
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
}
