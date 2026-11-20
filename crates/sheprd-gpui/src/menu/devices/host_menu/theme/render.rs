// ABOUTME: Shares device-theme dialog and row templates between active and frozen content.
// ABOUTME: Omits native input and choice actions from departing copies.
use super::{paint::Choice, *};

impl SheprdWindow {
    pub(in crate::menu) fn render_device_theme_paint(
        &self,
        paint: &Paint,
        cx: &mut Context<Self>,
    ) -> Div {
        let rows = div()
            .id("device-theme-results")
            .flex_1()
            .min_h_0()
            .relative()
            .overflow_hidden()
            .children(paint.rows.iter().enumerate().map(|(slot, row)| {
                div()
                    .absolute()
                    .top(paint.offset + paint.row_height * slot as f32)
                    .left_0()
                    .right_0()
                    .child(self.device_theme_choice(row, None, paint.row_height, cx))
            }));
        self.device_theme_content(
            &paint.label,
            paint.input.render().into_any_element(),
            rows.into_any_element(),
            paint.status.clone(),
            false,
            cx,
        )
    }

    pub(super) fn device_theme_choice(
        &self,
        choice: &Choice,
        action: Option<Row>,
        height: Pixels,
        cx: &mut Context<Self>,
    ) -> Stateful<Div> {
        let index = choice.index;
        div()
            .id(index)
            .debug_selector(move || format!("device-theme-row-{index}"))
            .w_full()
            .h(height)
            .px(px(16.))
            .flex()
            .items_center()
            .gap(px(8.))
            .when(choice.selected, |row| row.bg(rgb(self.theme.active)))
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .truncate()
                    .child(choice.label.clone()),
            )
            .when(choice.current, |row| {
                row.child(
                    div()
                        .text_color(rgb(self.theme.muted))
                        .child(fl!("host-menu-theme-current")),
                )
            })
            .when_some(action, |row, action| {
                row.cursor_pointer()
                    .hover(|s| s.bg(rgb(self.theme.active)))
                    .on_hover(cx.listener(move |this, hovered, _, cx| {
                        if *hovered && let Some(picker) = this.device_theme_picker_mut() {
                            picker.selected = index;
                            this.preview_device_theme(cx);
                        }
                    }))
                    .on_click(cx.listener(move |this, _, window, cx| {
                        this.choose_device_theme(action.clone(), window, cx)
                    }))
            })
    }

    pub(super) fn device_theme_content(
        &self,
        label: &str,
        input: AnyElement,
        rows: AnyElement,
        status: String,
        active: bool,
        cx: &mut Context<Self>,
    ) -> Div {
        let theme = &self.theme;
        let font = &self.config.ui;
        let header = div()
            .flex_none()
            .p(px(16.))
            .border_b_1()
            .border_color(rgb(theme.active))
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(12.))
                    .child(
                        div()
                            .flex_1()
                            .min_w_0()
                            .truncate()
                            .text_size(px(font.size * 1.35))
                            .font_weight(FontWeight::SEMIBOLD)
                            .child(fl!("host-menu-theme-title", device = label)),
                    )
                    .child(
                        div()
                            .id("device-theme-close")
                            .debug_selector(|| "device-theme-close".into())
                            .px_2()
                            .py_1()
                            .rounded(px(crate::config::corners::CONTROL))
                            .child(fl!("host-menu-theme-close"))
                            .when(active, |button| {
                                button
                                    .cursor_pointer()
                                    .hover(|s| s.bg(rgb(theme.active)))
                                    .on_click(cx.listener(|this, _, window, cx| {
                                        this.dismiss_menu(window, cx)
                                    }))
                            }),
                    ),
            )
            .child(div().pt(px(12.)).child(input));
        div()
            .debug_selector(|| "device-theme".into())
            .flex()
            .flex_col()
            .size_full()
            .min_h_0()
            .child(header)
            .child(rows)
            .child(
                div()
                    .debug_selector(|| "device-theme-status".into())
                    .flex_none()
                    .px(px(16.))
                    .py(px(10.))
                    .border_t_1()
                    .border_color(rgb(theme.active))
                    .text_color(rgb(theme.muted))
                    .child(status),
            )
    }
}
