// ABOUTME: Prepares Core clipboard input away from the UI context.
// ABOUTME: Reserves FIFO order and pairs native Ctrl-V fallback releases.
//! Pasting into a herdr-core pane: the clipboard's text goes to the
//! program as a paste, which the terminal brackets when the program asks.
//! The same keys paste as over a Herdr pane, and the Edit menu's Paste
//! lands here rather than in the Herdr pane behind this one.

use super::CoreTerminalView;
use crate::core_terminals::worker::Fallback;
use crate::{
    i18n::Localize,
    window::clipboard::{self, Error, Mode},
};
use gpui::{ClipboardItem, Context, Keystroke, Subscription, Task, Window};
use sheprd_backend::ClientPaneInputEvent;

pub(super) struct Pending {
    _blur: Subscription,
    _task: Task<()>,
    cancellation: crate::core_terminals::worker::Cancellation,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Trigger {
    Paste,
    Probe,
}

/// Whether `keystroke` pastes: Cmd-V, Ctrl-Shift-V, or Shift-Insert, and
/// plain Ctrl-V where it pastes (Windows, and Linux, whose desktops send it
/// for a universal paste) while the clipboard holds text.
pub(crate) fn trigger(keystroke: &Keystroke, ctrl_v_pastes: bool) -> Option<Trigger> {
    let modifiers = keystroke.modifiers;
    let v = keystroke.key.eq_ignore_ascii_case("v");
    let insert = keystroke.key == "insert";
    if modifiers.alt || modifiers.function {
        return None;
    }
    if (modifiers.platform && !modifiers.control && v)
        || (modifiers.control && modifiers.shift && (v || insert))
        || (modifiers.shift && !modifiers.control && !modifiers.platform && insert)
    {
        return Some(Trigger::Paste);
    }
    let plain_ctrl_v = modifiers.control && !modifiers.shift && !modifiers.platform && v;
    (ctrl_v_pastes && plain_ctrl_v).then_some(Trigger::Probe)
}

impl CoreTerminalView {
    /// Reserves clipboard input without blocking the UI.
    /// Returns false when the view cannot start preparing input.
    pub(super) fn paste_clipboard(&mut self, window: &mut Window, cx: &mut Context<Self>) -> bool {
        self.start_paste(None, window, cx)
    }

    pub(super) fn cancel_clipboard(&mut self, cx: &mut Context<Self>) {
        if let Some(pending) = self.pending_paste.take() {
            pending.cancellation.cancel();
            drop(pending);
            if let Some((key, _)) = self.paste_release.take() {
                self.held_keys.forget(&key);
            }
            self.link.failure(Error::Cancelled.localize());
            cx.notify();
        }
    }

    pub(super) fn start_paste(
        &mut self,
        fallback: Option<(String, ClientPaneInputEvent)>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        self.start_paste_with(fallback, |cx| clipboard::start(Mode::Text, cx), window, cx)
    }

    pub(super) fn start_paste_with(
        &mut self,
        fallback: Option<(String, ClientPaneInputEvent)>,
        read: impl FnOnce(&mut gpui::App) -> crate::Result<Task<crate::Result<Option<ClipboardItem>>>>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        if self.observe {
            return false;
        }
        if self.pending_paste.is_some() {
            self.link.failure(Error::Busy.localize());
            cx.notify();
            return true;
        }
        if !self.focus.is_focused(window) {
            return false;
        }
        let Some(prepared) = self.link.prepare() else {
            cx.notify();
            return false;
        };
        let cancellation = prepared.cancellation();
        let read = match read(cx) {
            Ok(read) => read,
            Err(error) => {
                self.link.failure(error.localize());
                cx.notify();
                return false;
            }
        };
        let gate = Fallback::default();
        let original = fallback.map(|(key, input)| {
            self.paste_release = Some((key.clone(), gate.clone()));
            self.held_keys.press(&key, input, self.screen.keys.releases)
        });
        let blur = cx.on_blur(&self.focus.clone(), window, |this, _, cx| {
            this.cancel_clipboard(cx)
        });
        let task = cx.spawn_in(window, async move |this, cx| {
            let result = read.await;
            let _ = this.update_in(cx, |this, window, cx| {
                if !this.focus.is_focused(window) {
                    this.cancel_clipboard(cx);
                    return;
                }
                match resolve(result, original) {
                    Ok((input, used_key)) => {
                        if used_key {
                            gate.select();
                        }
                        prepared.finish(input);
                    }
                    Err(error) => this.link.failure(error.localize()),
                }
                this.pending_paste = None;
                cx.notify();
            });
        });
        self.pending_paste = Some(Pending {
            _blur: blur,
            _task: task,
            cancellation,
        });
        true
    }
}

fn resolve(
    result: crate::Result<Option<ClipboardItem>>,
    original: Option<ClientPaneInputEvent>,
) -> crate::Result<(Option<ClientPaneInputEvent>, bool)> {
    let item = match result {
        Ok(item) => item,
        Err(error) if original.is_some() => {
            tracing::warn!(%error, "Core clipboard probe failed; preserving original key");
            None
        }
        Err(error) => return Err(error),
    };
    if let Some(text) = item
        .and_then(|item| item.text())
        .filter(|text| !text.is_empty())
    {
        return Ok((Some(ClientPaneInputEvent::Paste(text)), false));
    }
    let used_key = original.is_some();
    Ok((original, used_key))
}
