// ABOUTME: Exposes chat disclosures through pointer, keyboard, and screen-reader actions.
// ABOUTME: Keeps expansion tied to the same captured conversation and visible text.
//! The header that opens and closes a folded card: a thought, a tool call,
//! a notice's details, or a context summary.

use crate::{SheprdWindow, config::Theme};
use gpui::{prelude::*, *};
use sheprd_chat_protocol::EntryId;

/// What the selectable text inside a disclosure header is.
#[derive(Clone, Copy)]
pub(super) enum HeaderText {
    /// A label that stays in the header, so a plain click on it always
    /// toggles the card.
    Label,
    /// The entry's own text, which opening shows in full. Once open, a
    /// click on it selects rather than folds.
    Body,
}

/// The chevron leading a disclosure header: right while folded, down while
/// open.
pub(super) fn chevron(expanded: bool, theme: &Theme) -> Svg {
    svg()
        .path(if expanded {
            "icons/chevron-down.svg"
        } else {
            "icons/chevron-right.svg"
        })
        .flex_none()
        .size(px(10.))
        .text_color(rgb(theme.muted))
}

impl SheprdWindow {
    /// Makes `header` open and close entry `id` of the chat in `pane_id`.
    /// Its selectable text takes the mouse-down to keep keyboard focus in
    /// the reader, and a plain click on that text still toggles.
    pub(super) fn chat_disclosure(
        &self,
        header: Stateful<Div>,
        pane_id: &str,
        id: EntryId,
        expanded: bool,
        text: HeaderText,
        cx: &mut Context<Self>,
    ) -> Stateful<Div> {
        let text_pane = pane_id.to_owned();
        let toggle_pane = pane_id.to_owned();
        let pane = self.chats.pane(pane_id);
        let owner = pane.map(|pane| (pane.target.clone(), pane.session.since()));
        let label = pane
            .and_then(|pane| pane.prepared.text().entry(id))
            .and_then(|projection| {
                projection
                    .texts()
                    .next()
                    .map(|text| crate::notifications::safe_text(text, 400))
            })
            .unwrap_or_else(|| crate::i18n::fl!("chat-details"));
        let toggle = move |this: &mut Self, _: &mut Window, cx: &mut Context<Self>| {
            let Some(pane) = this.chats.pane_mut(&toggle_pane) else {
                return;
            };
            if !owner.as_ref().is_some_and(|(target, cursor)| {
                crate::chat::files::is_current(&pane.target, pane.session.since(), target, *cursor)
            }) {
                return;
            }
            pane.toggle_expanded(id);
            cx.notify();
        };
        let header =
            super::accessibility::button(&self.theme, label, header).key_context("ChatDisclosure");
        let header =
            crate::accessibility::expandable(header, expanded, cx.weak_entity(), toggle.clone());
        let label_toggles = match text {
            HeaderText::Label => true,
            HeaderText::Body => !expanded,
        };
        header
            .cursor_pointer()
            .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
            .when(label_toggles, |header| {
                header.on_mouse_up(
                    MouseButton::Left,
                    cx.listener(move |this, event: &MouseUpEvent, _, cx| {
                        this.click_chat_card_text(&text_pane, id, event.position, cx);
                    }),
                )
            })
            .on_click(cx.listener(move |this, _: &ClickEvent, window, cx| {
                cx.stop_propagation();
                toggle(this, window, cx);
            }))
    }
}
