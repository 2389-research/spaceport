//! Selection for prepared fallback lines and the plain text of activity cards.

use super::selection::Scope;
use crate::chat::selection::{Cell, Region};
use crate::{SheprdWindow, release_notes};
use gpui::{prelude::*, *};
use sheprd_chat_protocol::EntryId;

impl SheprdWindow {
    pub(in crate::chat) fn chat_text_scope<'a>(
        &'a self,
        pane_id: &'a str,
        entry: EntryId,
        region: Region,
    ) -> Option<Scope<'a>> {
        let chat = self.chats.pane(pane_id)?;
        Some(Scope::new(
            pane_id,
            entry,
            region,
            chat.prepared.text(),
            &chat.selection,
            &chat.text_geometry,
        ))
    }

    /// The selection document's text at `cell` of `region`, drawn
    /// selectable: what is drawn is what is selected and copied. `None`
    /// where the document holds no text.
    pub(super) fn render_chat_segment(
        &self,
        pane_id: &str,
        entry: EntryId,
        region: Region,
        cell: Cell,
        cx: &mut Context<Self>,
    ) -> Option<Div> {
        let scope = self.chat_text_scope(pane_id, entry, region)?;
        let fragment = scope.fragment(cell)?;
        let text = SharedString::from(scope.segment(cell)?.shared());
        let styled = fragment.styled(text, [], &self.theme);
        let layout = styled.layout().clone();
        Some(fragment.wrap(div().min_w_0().child(styled), layout, cx))
    }

    /// One run of an entry's plain text, at `cell` of `region`: the
    /// document's segment. The document is projected whenever the transcript
    /// or a card's fold changes, before the row is drawn; `fallback` is what
    /// the row shows, plain, should the document hold no text there, so a
    /// row is never blank.
    pub(super) fn render_chat_text(
        &self,
        pane_id: &str,
        entry: EntryId,
        region: Region,
        cell: Cell,
        fallback: impl FnOnce() -> String,
        cx: &mut Context<Self>,
    ) -> Div {
        self.render_chat_segment(pane_id, entry, region, cell, cx)
            .unwrap_or_else(|| div().child(fallback()))
    }

    pub(super) fn render_chat_lines(
        &self,
        id: &'static str,
        scope: Option<Scope<'_>>,
        lines: &[release_notes::Line],
        cx: &mut Context<Self>,
    ) -> Div {
        div()
            .flex()
            .flex_col()
            .gap(px(2.))
            .min_w_0()
            .children(lines.iter().enumerate().map(|(index, line)| {
                let Some(fragment) = scope
                    .as_ref()
                    .and_then(|scope| scope.fragment(Cell::row(index)))
                else {
                    return release_notes::line(
                        id,
                        index,
                        line,
                        &self.theme,
                        &self.config.terminal,
                    );
                };
                let styled = fragment.styled(
                    line.text.clone(),
                    release_notes::line_highlights(
                        line,
                        &self.theme,
                        self.theme.ink(self.theme.palette[4]),
                    ),
                    &self.theme,
                );
                let layout = styled.layout().clone();
                let content = if line.links.is_empty() {
                    styled.into_any_element()
                } else {
                    let links = line.links.clone();
                    let guard = fragment.clone();
                    InteractiveText::new(("chat-line-link", index), styled)
                        .on_click(
                            links.iter().map(|link| link.range.clone()).collect(),
                            move |clicked, _, cx| {
                                if guard.allows_link()
                                    && let Some(link) = links.get(clicked)
                                {
                                    cx.open_url(&link.url);
                                }
                            },
                        )
                        .into_any_element()
                };
                let selected = fragment
                    .wrap(div().min_w_0().child(content), layout, cx)
                    .into_any_element();
                release_notes::line_with(
                    id,
                    index,
                    line,
                    &self.theme,
                    &self.config.terminal,
                    selected,
                )
            }))
    }
}
