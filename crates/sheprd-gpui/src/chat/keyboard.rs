// ABOUTME: Traverses native chat controls without stealing composer or IME input.
// ABOUTME: Keeps Tab inside its conversation while slash completion retains plain Tab.
use crate::{SheprdWindow, chat::chats::ChatInput};
use gpui::{Context, KeyDownEvent, Modifiers, Window};

impl SheprdWindow {
    pub(super) fn chat_control_key(
        &mut self,
        pane_id: &str,
        event: &KeyDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        let modifiers = event.keystroke.modifiers;
        if event.keystroke.key == "escape" && !modifiers.modified() {
            let closing = self
                .chats
                .pane(pane_id)
                .filter(|pane| {
                    !pane
                        .composer
                        .as_ref()
                        .is_some_and(|input| input.read(cx).is_composing())
                        && !pane
                            .menu_search()
                            .is_some_and(|input| input.read(cx).is_composing())
                })
                .and_then(|pane| pane.menu().cloned());
            if let Some(menu) = closing {
                self.toggle_chat_menu(pane_id, menu, window, cx);
                cx.stop_propagation();
                window.prevent_default();
                return true;
            }
        }
        if event.keystroke.key != "tab"
            || (modifiers != Modifiers::default() && modifiers != Modifiers::shift())
        {
            return false;
        }
        let Some(pane) = self.chats.pane(pane_id) else {
            return false;
        };
        if pane.widgets.focused(window, cx) || pane.menu_search_focused(window, cx) {
            return false;
        }
        if let Some(input) = &pane.composer
            && input.read(cx).focus.is_focused(window)
        {
            if input.read(cx).is_composing() {
                return true;
            }
            if !modifiers.shift
                && !sheprd_chat_protocol::assist::completions(
                    pane.session.transcript(),
                    input.read(cx).text(),
                )
                .is_empty()
            {
                return false;
            }
        }
        let root = match pane.input() {
            ChatInput::ReadOnly => pane.read_only_focus.clone(),
            _ => pane.text_focus.clone(),
        };
        let Some(root) = root else { return false };
        let mut first = None;
        loop {
            if modifiers.shift {
                window.focus_prev(cx);
            } else {
                window.focus_next(cx);
            }
            let focused = window.focused(cx);
            if focused.is_none() || focused == first || root.contains_focused(window, cx) {
                break;
            }
            if first.is_none() {
                first = focused;
            }
        }
        cx.stop_propagation();
        window.prevent_default();
        cx.notify();
        true
    }
}
