// ABOUTME: Reads chat clipboard content outside the UI execution context.
// ABOUTME: Applies attachments and text only to their original conversation and field ticket.
use super::{chats::ChatInput, files::is_current};
use crate::{
    SheprdWindow,
    multiline_input::MultilineInput,
    window::clipboard::{self, Mode},
};
use gpui::{ClipboardEntry, ClipboardItem, Context, Entity, EntityInputHandler, Window};

impl SheprdWindow {
    pub(super) fn start_chat_paste(
        &mut self,
        pane_id: &str,
        input: &Entity<MultilineInput>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.start_chat_paste_with(
            pane_id,
            input,
            |cx| clipboard::start(Mode::Normal, cx),
            window,
            cx,
        );
    }

    pub(super) fn start_chat_paste_with(
        &mut self,
        pane_id: &str,
        input: &Entity<MultilineInput>,
        read: impl FnOnce(
            &mut gpui::App,
        ) -> crate::Result<gpui::Task<crate::Result<Option<ClipboardItem>>>>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(pane) = self.chats.pane(pane_id) else {
            return;
        };
        let target = pane.target.clone();
        let cursor = pane.session.since();
        let Some(ticket) = input.update(cx, |input, cx| input.begin_paste(window, cx)) else {
            return;
        };
        let read = match read(cx) {
            Ok(read) => read,
            Err(error) => {
                input.update(cx, |input, cx| input.paste_error(error, cx));
                return;
            }
        };
        let weak = input.downgrade();
        let pane_id = pane_id.to_owned();
        let pending = ticket.clone();
        let task = cx.spawn_in(window, async move |this, cx| {
            let result = read.await;
            let _ = this.update_in(cx, |this, window, cx| {
                let Some(input) = weak.upgrade() else {
                    return;
                };
                let current = this.chats.pane(&pane_id).is_some_and(|pane| {
                    pane.input() == ChatInput::Composer
                        && pane
                            .composer
                            .as_ref()
                            .is_some_and(|current| current.entity_id() == input.entity_id())
                        && is_current(&pane.target, pane.session.since(), &target, cursor)
                });
                if !current {
                    input.update(cx, |input, cx| input.cancel_paste(cx));
                    return;
                }
                if !input.update(cx, |input, cx| input.finish_paste(&pending, window, cx)) {
                    return;
                }
                match result {
                    Ok(Some(item)) => this.apply_chat_clipboard(&pane_id, &input, item, window, cx),
                    Ok(None) => {}
                    Err(error) => input.update(cx, |input, cx| input.paste_error(error, cx)),
                }
            });
        });
        input.update(cx, |input, _| input.hold_paste(&ticket, task));
    }

    fn apply_chat_clipboard(
        &mut self,
        pane_id: &str,
        input: &Entity<MultilineInput>,
        item: ClipboardItem,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if let Some(paths) = item.entries().iter().find_map(|entry| match entry {
            ClipboardEntry::ExternalPaths(paths) if !paths.paths().is_empty() => {
                Some(paths.paths().to_vec())
            }
            _ => None,
        }) {
            self.attach_chat_files(pane_id, paths, window, cx);
        } else if let Some(text) = item.text() {
            input.update(cx, |input, cx| {
                input.replace_text_in_range(None, &text, window, cx)
            });
        } else if let Some(image) = item.into_entries().find_map(|entry| match entry {
            ClipboardEntry::Image(image) => Some(image),
            _ => None,
        }) {
            self.attach_chat_image(pane_id, image, window, cx);
        }
        self.redraw_terminal(cx);
        cx.notify();
    }
}
