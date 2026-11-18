// ABOUTME: Acquires and applies clipboard text for one multiline field.
// ABOUTME: Keeps native acquisition separate from bounded text replacement.
use super::MultilineInput;
use crate::{
    i18n::Localize,
    window::clipboard::{self, Error},
};
use gpui::{ClipboardItem, Context, EntityInputHandler, Subscription, Task, Window};
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

#[derive(Clone)]
pub(crate) struct Ticket(Arc<AtomicBool>);

pub(super) struct Pending {
    _blur: Subscription,
    revision: u64,
    selection: (usize, usize),
    task: Option<Task<()>>,
    ticket: Ticket,
}

impl Drop for Pending {
    fn drop(&mut self) {
        self.ticket.0.store(true, Ordering::Release);
    }
}

impl MultilineInput {
    pub(crate) fn is_pasting(&self) -> bool {
        self.pending_paste.is_some()
    }

    pub(super) fn paste_from(
        &mut self,
        read: impl FnOnce() -> crate::Result<Option<ClipboardItem>> + Send + 'static,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(ticket) = self.begin_paste(window, cx) else {
            return;
        };
        let task = match clipboard::start_with(cx, read) {
            Ok(task) => task,
            Err(error) => {
                self.paste_error(error, cx);
                return;
            }
        };
        self.await_paste(&ticket, task, window, cx);
    }

    pub(super) fn await_paste(
        &mut self,
        ticket: &Ticket,
        read: Task<crate::Result<Option<ClipboardItem>>>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let pending = ticket.clone();
        let task = cx.spawn_in(window, async move |this, cx| {
            let result = read.await;
            let _ = this.update_in(cx, |this, window, cx| {
                if !this.finish_paste(&pending, window, cx) {
                    return;
                }
                match result {
                    Ok(Some(item)) => {
                        if let Some(text) = item.text() {
                            this.replace_text_in_range(None, &text, window, cx);
                        }
                    }
                    Ok(None) => {}
                    Err(error) => this.paste_error(error, cx),
                }
            });
        });
        self.hold_paste(ticket, task);
    }

    pub(crate) fn begin_paste(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Option<Ticket> {
        if self.refuse_while_pasting(cx) || !self.focus.is_focused(window) || self.is_composing() {
            return None;
        }
        let ticket = Ticket(Arc::new(AtomicBool::new(false)));
        let blur = cx.on_blur(&self.focus.clone(), window, |this, _, cx| {
            this.cancel_paste(cx)
        });
        self.pending_paste = Some(Pending {
            _blur: blur,
            revision: self.revision,
            selection: (self.edit.anchor, self.edit.cursor),
            task: None,
            ticket: ticket.clone(),
        });
        cx.notify();
        Some(ticket)
    }

    pub(crate) fn hold_paste(&mut self, ticket: &Ticket, task: Task<()>) {
        if let Some(pending) = &mut self.pending_paste
            && Arc::ptr_eq(&ticket.0, &pending.ticket.0)
        {
            pending.task = Some(task);
        }
    }

    pub(crate) fn finish_paste(
        &mut self,
        ticket: &Ticket,
        window: &Window,
        cx: &mut Context<Self>,
    ) -> bool {
        let Some(pending) = &self.pending_paste else {
            return false;
        };
        if !Arc::ptr_eq(&ticket.0, &pending.ticket.0) {
            return false;
        }
        let valid = !ticket.0.load(Ordering::Acquire)
            && pending.revision == self.revision
            && pending.selection == (self.edit.anchor, self.edit.cursor)
            && !self.is_composing()
            && self.focus.is_focused(window);
        self.pending_paste = None;
        if !valid {
            self.paste_error(Error::Cancelled.into(), cx);
        }
        cx.notify();
        valid
    }

    pub(crate) fn refuse_while_pasting(&mut self, cx: &mut Context<Self>) -> bool {
        if self.pending_paste.is_none() {
            return false;
        }
        self.error = Some(Error::Busy.localize().into());
        cx.notify();
        true
    }

    pub(crate) fn cancel_paste(&mut self, cx: &mut Context<Self>) {
        if self.pending_paste.take().is_some() {
            self.error = Some(Error::Cancelled.localize().into());
            cx.notify();
        }
    }

    pub(crate) fn paste_error(&mut self, error: crate::Error, cx: &mut Context<Self>) {
        self.pending_paste = None;
        self.error = Some(error.localize().into());
        cx.notify();
    }
}
