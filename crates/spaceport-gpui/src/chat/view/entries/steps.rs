// ABOUTME: Renders nested sub-agent steps inside their parent card.
// ABOUTME: Uses the same tool state as the other chat renderers.
//! A sub-agent's own steps, listed in its call's card when it is open: one
//! quiet line each, its state's mark and its title.

use super::{ToolState, tool_header, tool_state};
use crate::{SpaceportWindow, chat::chats::ChatPane, fonts::StyledFont, i18n::fl};
use gpui::{prelude::*, *};
use spaceport_chat_protocol::{EntryId, EntryKind};

/// `12 steps`, beside a sub-agent's state.
pub(super) fn steps_label(count: usize) -> String {
    fl!("chat-tool-steps", count = count)
}

impl SpaceportWindow {
    pub(super) fn render_chat_steps(
        &self,
        chat: &ChatPane,
        parent: EntryId,
        steps: &[EntryId],
    ) -> Div {
        let theme = &self.theme;
        let transcript = chat.session.transcript();
        let text = self.chat_tool_text();
        div()
            .debug_selector(move || format!("chat-steps-{}", parent.0))
            .flex()
            .flex_col()
            .ml(px(6.))
            .pl(px(10.))
            .border_l_1()
            .border_color(rgb(theme.active))
            .text_size(text)
            .children(steps.iter().filter_map(|id| {
                let EntryKind::Tool(tool) = &transcript.entry(*id)?.kind else {
                    return None;
                };
                let state = tool_state(transcript, tool);
                let mark = match state {
                    ToolState::Pending => "\u{25cb}",
                    ToolState::Running => "\u{25cf}",
                    ToolState::Done => "\u{2713}",
                    ToolState::Failed => "\u{2717}",
                    ToolState::Stopped => "\u{25a0}",
                };
                let color = state.color(theme);
                let id = *id;
                Some(
                    div()
                        .debug_selector(move || format!("chat-step-{}", id.0))
                        .flex()
                        .items_center()
                        .gap(px(6.))
                        .child(div().flex_none().text_color(rgb(color)).child(mark))
                        .child(
                            div()
                                .min_w_0()
                                .truncate()
                                .text_font(&self.config.terminal)
                                .text_color(rgb(theme.muted))
                                .child(tool_header(&tool.title, &chat.labels)),
                        ),
                )
            }))
    }
}
