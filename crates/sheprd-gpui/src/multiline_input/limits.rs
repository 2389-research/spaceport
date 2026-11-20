// ABOUTME: Enforces the shared prompt byte contract before edits allocate replacement text.
// ABOUTME: Rejects oversized changes while retaining the draft, selection, and IME state.
use super::MultilineInput;
use gpui::Context;
use sheprd_chat_protocol::MAX_PROMPT_SIZE;
use std::ops::Range;

impl MultilineInput {
    pub(super) fn permits_replacement(
        &mut self,
        replaced: Range<usize>,
        text: &str,
        cx: &mut Context<Self>,
    ) -> bool {
        // Normalizing CRLF can shrink input by at most half. Refuse larger input before scanning it.
        let fits = text.len() <= MAX_PROMPT_SIZE * 2 && {
            let normalized = text.len()
                - text
                    .as_bytes()
                    .windows(2)
                    .filter(|pair| *pair == b"\r\n")
                    .count();
            (self.edit.text.len() - replaced.len())
                .checked_add(normalized)
                .is_some_and(|size| size <= MAX_PROMPT_SIZE)
        };
        if !fits {
            self.error = Some(crate::i18n::fl!("chat-prompt-too-long").into());
            cx.notify();
        }
        fits
    }
}
