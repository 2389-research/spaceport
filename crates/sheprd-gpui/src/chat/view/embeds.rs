//! What an agent message shows in place of things the app loads only when
//! asked: HTML opened in the system browser after a click, a
//! picture on the web, and the caption of a picture on the agent's machine
//! that follows the message.

use super::{
    super::{
        page::PageSource,
        visual::{Source, VisualId, Visuals},
    },
    visuals::Frame,
};
use crate::{SheprdWindow, browser::WebUrl, config::corners, i18n::fl, notifications::safe_text};
use gpui::{prelude::*, *};
use sheprd_chat_protocol::EntryId;
use std::{path::PathBuf, rc::Rc};

impl SheprdWindow {
    /// An HTML fence shown as a card with an explicit open action.
    pub(super) fn render_chat_html(
        &self,
        pane_id: &str,
        _entry: EntryId,
        part: usize,
        page: &Rc<str>,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let lines = page.lines().count();
        let source = PageSource::Html(page.clone());
        self.embed_card("icons/globe.svg")
            .debug_selector(move || format!("chat-html-{part}"))
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .child(fl!("chat-page-html-lines", count = lines)),
            )
            .child(self.show_button(pane_id, ("chat-show-page", part), source, cx))
            .into_any_element()
    }

    /// Under a tool call that wrote an HTML file, show an explicit open action.
    pub(super) fn render_chat_file_page(
        &self,
        pane_id: &str,
        entry: EntryId,
        path: PathBuf,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        div()
            .flex()
            .child(self.show_button(
                pane_id,
                ("chat-show-file", entry.0),
                PageSource::File(path),
                cx,
            ))
            .into_any_element()
    }

    fn show_button(
        &self,
        pane_id: &str,
        id: impl Into<ElementId>,
        source: PageSource,
        cx: &mut Context<Self>,
    ) -> Stateful<Div> {
        let theme = &self.theme;
        let pane_id = pane_id.to_owned();
        let fill = theme.primary();
        div()
            .id(id)
            .debug_selector(|| "chat-show-page".into())
            .flex_none()
            .px(px(10.))
            .h(px(24.))
            .flex()
            .items_center()
            .rounded(px(corners::CONTROL))
            .bg(rgb(fill))
            .text_color(rgb(theme.text_on(fill)))
            .text_size(self.chat_small_text())
            .cursor_pointer()
            .child(fl!("chat-page-open-in-browser"))
            .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
            .on_click(cx.listener(move |this, _: &ClickEvent, window, cx| {
                cx.stop_propagation();
                this.show_chat_page(&pane_id, source.clone(), window, cx);
            }))
    }

    /// A picture on the web: drawn once loaded, until then a card that
    /// names its host and loads it on request.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn render_chat_remote_picture(
        &self,
        pane_id: &str,
        entry: EntryId,
        index: usize,
        id: VisualId,
        url: &WebUrl,
        alt: &str,
        visuals: &Visuals,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        if visuals.wants(id) {
            return self
                .render_chat_visual(pane_id, id, visuals, Frame::Image, cx)
                .into_any_element();
        }
        let theme = &self.theme;
        let pane_id = pane_id.to_owned();
        let url = url.clone();
        let host = safe_text(url.host(), 120);
        self.embed_card("icons/zoom.svg")
            .debug_selector(move || format!("chat-remote-{index}"))
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .flex()
                    .flex_col()
                    .child(div().truncate().child(if alt.is_empty() {
                        fl!("chat-picture-web")
                    } else {
                        alt.to_owned()
                    }))
                    .child(
                        div()
                            .text_size(px((self.chat_text_size() - 2.).max(10.)))
                            .text_color(rgb(theme.muted))
                            .truncate()
                            .child(fl!("chat-picture-web-warning", host = host.as_str())),
                    ),
            )
            .child(
                div()
                    .id(("chat-load-picture", index))
                    .debug_selector(|| "chat-load-picture".into())
                    .flex_none()
                    .px(px(10.))
                    .h(px(24.))
                    .flex()
                    .items_center()
                    .rounded(px(corners::CONTROL))
                    .border_1()
                    .border_color(rgb(theme.active))
                    .text_size(self.chat_small_text())
                    .cursor_pointer()
                    .hover(|button| button.bg(rgb(theme.active)))
                    .child(fl!("chat-picture-load", host = host.as_str()))
                    .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
                    .on_click(cx.listener(move |this, _: &ClickEvent, _, cx| {
                        cx.stop_propagation();
                        if let Some(pane) = this.chats.pane_mut(&pane_id) {
                            pane.prepared.visuals.want(id, Source::Remote(url.clone()));
                            pane.remeasure(entry);
                        }
                        cx.notify();
                    })),
            )
            .into_any_element()
    }

    /// The caption of a picture on the agent's machine; herdr-acp shows the
    /// picture itself after the message.
    pub(super) fn render_chat_picture_caption(&self, alt: &str) -> AnyElement {
        let theme = &self.theme;
        div()
            .flex()
            .items_center()
            .gap(px(6.))
            .text_size(self.chat_small_text())
            .text_color(rgb(theme.muted))
            .child(
                svg()
                    .path("icons/zoom.svg")
                    .size(px(11.))
                    .text_color(rgb(theme.muted)),
            )
            .child(if alt.is_empty() {
                fl!("chat-picture")
            } else {
                alt.to_owned()
            })
            .into_any_element()
    }

    fn embed_card(&self, icon: &'static str) -> Div {
        let theme = &self.theme;
        div()
            .max_w(px(520.))
            .flex()
            .items_center()
            .gap(px(10.))
            .px(px(10.))
            .py(px(8.))
            .rounded(px(corners::CONTROL))
            .border_1()
            .border_color(rgb(theme.active))
            .bg(rgb(theme.surface))
            .child(
                svg()
                    .flex_none()
                    .path(icon)
                    .size(px(14.))
                    .text_color(rgb(theme.subtext())),
            )
    }
}
