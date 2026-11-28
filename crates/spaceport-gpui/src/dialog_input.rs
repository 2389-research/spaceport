//! A menu dialog's draft, in `spaceport-textfield`, and the frame this window
//! draws around it: the accessible name, the click that puts its caret where
//! the pointer is, and the canvas the field paints its own text into.

use gpui::{prelude::*, *};

use super::{SpaceportWindow, input::ViewInputHandler};
pub(crate) use spaceport_textfield::dialog_input::DialogInput;

impl SpaceportWindow {
    pub(super) fn render_dialog_input(&self, cx: &mut Context<Self>) -> Stateful<Div> {
        let entity = cx.entity();
        let focus = self.menu.focus.clone();
        div()
            .id("dialog-input")
            .debug_selector(|| "dialog-input".into())
            .role(Role::TextInput)
            .aria_value(
                self.menu
                    .input
                    .as_ref()
                    .map(|input| SharedString::from(input.text.clone()))
                    .unwrap_or_default(),
            )
            .when_some(
                self.menu
                    .input
                    .as_ref()
                    .and_then(|input| input.label.clone()),
                |field, label| field.aria_label(label),
            )
            .w_full()
            .h(px(self.config.ui.line_height() + 10.))
            .px(px(6.))
            .py(px(5.))
            .rounded(px(crate::config::corners::CONTROL))
            .bg(rgb(self.theme.active))
            .overflow_hidden()
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(|this, event: &MouseDownEvent, window, cx| {
                    cx.stop_propagation();
                    // Another field of the same dialog may hold focus.
                    window.focus(&this.menu.focus, cx);
                    if let Some(input) = this.menu.input.as_mut()
                        && input.marked.is_none()
                        && let Some(index) = input.index_at(event.position)
                    {
                        input.select_to(index, event.modifiers.shift);
                        cx.notify();
                    }
                }),
            )
            .child(
                canvas(
                    |_, _, _| (),
                    move |bounds, _, window, cx| {
                        let focused = focus.is_focused(window);
                        window.handle_input(
                            &focus,
                            ViewInputHandler::new(bounds, entity.clone()),
                            cx,
                        );
                        entity.update(cx, |this, cx| {
                            let font = this.config.ui.clone();
                            let theme = this.theme.clone();
                            let Some(input) = this.menu.input.as_mut() else {
                                return;
                            };
                            input.set_appearance(font, theme);
                            input.paint(bounds, focused, window, cx);
                        });
                    },
                )
                .size_full(),
            )
    }
}
