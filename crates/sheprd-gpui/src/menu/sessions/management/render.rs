// ABOUTME: Shares session creation and deletion form templates with passive departure.
// ABOUTME: Keeps field ownership and mutation callbacks in the active form.
use super::*;

impl SheprdWindow {
    pub(in crate::menu) fn render_session_edit(&self, cx: &mut Context<Self>) -> Div {
        let Some(text) = self.session_edit_text() else {
            return div();
        };
        let input = match &self.menu.session_edit {
            Some(Edit::Create { input, .. }) => Some(input.clone().into_any_element()),
            _ => None,
        };
        self.session_edit_content(&text, input, true, cx)
    }

    pub(in crate::menu) fn render_session_edit_paint(
        &self,
        paint: &EditPaint,
        cx: &mut Context<Self>,
    ) -> Div {
        self.session_edit_content(
            &paint.text,
            paint
                .input
                .as_ref()
                .map(|input| input.render().into_any_element()),
            false,
            cx,
        )
        .relative()
        .top(paint.offset)
    }

    fn session_edit_content(
        &self,
        paint: &paint::EditText,
        input: Option<AnyElement>,
        active: bool,
        cx: &mut Context<Self>,
    ) -> Div {
        let theme = &self.theme;
        let mut view = div()
            .debug_selector(|| "session-modal".into())
            .flex()
            .flex_col()
            .gap(px(12.))
            .p(px(12.));
        match &paint.kind {
            paint::Kind::Create { destination } => {
                view = view
                    .child(
                        div()
                            .text_size(px(self.config.ui.size * 1.35))
                            .font_weight(FontWeight::SEMIBOLD)
                            .child(fl!("sessions-menu-add-title")),
                    )
                    .child(
                        div()
                            .text_color(rgb(theme.muted))
                            .child(fl!("sessions-menu-on", destination = destination.as_str())),
                    )
                    .children(input)
                    .child(
                        div()
                            .text_color(rgb(theme.subtext()))
                            .child(fl!("sessions-menu-add-note")),
                    );
            }
            paint::Kind::Delete { name } => {
                view = view
                    .child(
                        div()
                            .text_size(px(self.config.ui.size * 1.35))
                            .font_weight(FontWeight::SEMIBOLD)
                            .child(fl!("sessions-menu-delete-title")),
                    )
                    .child(div().min_w_0().truncate().child(name.to_owned()))
                    .child(
                        div()
                            .text_color(rgb(theme.subtext()))
                            .child(fl!("sessions-menu-delete-note")),
                    );
            }
        }
        if let Some(error) = &paint.error {
            view = view.child(
                div()
                    .text_color(colors::danger(&self.theme))
                    .child(error.clone()),
            );
        }
        view.child(
            div()
                .flex()
                .justify_end()
                .gap(px(8.))
                .child(
                    div()
                        .id("session-edit-cancel")
                        .debug_selector(|| "session-edit-cancel".into())
                        .px(px(12.))
                        .py(px(6.))
                        .rounded(px(crate::config::corners::CONTROL))
                        .border_1()
                        .border_color(rgb(theme.active))
                        .when(active, |button| {
                            button.hover(|s| s.bg(rgb(theme.active))).cursor_pointer()
                        })
                        .child(fl!("sessions-menu-cancel"))
                        .when(active, |button| {
                            button.on_click(cx.listener(|this, _, window, cx| {
                                this.cancel_session_edit(window, cx)
                            }))
                        }),
                )
                .child(
                    div()
                        .id("session-edit-submit")
                        .debug_selector(|| "session-edit-submit".into())
                        .px(px(12.))
                        .py(px(6.))
                        .rounded(px(crate::config::corners::CONTROL))
                        .border_1()
                        .border_color(rgb(theme.active))
                        .bg(rgb(theme.active))
                        .when(matches!(paint.kind, paint::Kind::Delete { .. }), |s| {
                            s.text_color(colors::danger(theme))
                        })
                        .when(active, |button| button.cursor_pointer())
                        .child(if matches!(paint.kind, paint::Kind::Create { .. }) {
                            fl!("sessions-menu-create-and-connect")
                        } else {
                            fl!("sessions-menu-delete-permanently")
                        })
                        .when(active, |button| {
                            button.on_click(cx.listener(|this, _, window, cx| {
                                this.submit_session_edit(window, cx)
                            }))
                        }),
                ),
        )
    }
}
