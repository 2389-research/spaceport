// ABOUTME: Presents bounded, memory-only warning history for the current window.
// ABOUTME: Reuses notice IDs to merge cloned endpoint projections without duplicate entries.
use super::SheprdWindow;
use crate::{i18n::fl, menu::Page, notice::Notice};
use gpui::{prelude::*, *};
use std::collections::BTreeMap;

const WINDOW_LIMIT: usize = 64;

impl SheprdWindow {
    pub(crate) fn messages(&self) -> Vec<(Notice, String)> {
        let mut entries = BTreeMap::new();
        for notice in self.notices.history() {
            entries.insert(notice.id, (notice.clone(), fl!("messages-this-window")));
        }
        for notice in self.live.notices.history() {
            let label = self.endpoints.get(self.selected_endpoint).map_or_else(
                || fl!("messages-this-window"),
                |endpoint| endpoint.label.clone(),
            );
            entries.insert(notice.id, (notice.clone(), label));
        }
        for endpoint in &self.endpoints {
            for notice in endpoint.live.notices.history() {
                entries
                    .entry(notice.id)
                    .or_insert_with(|| (notice.clone(), endpoint.label.clone()));
            }
        }
        entries.into_values().rev().take(WINDOW_LIMIT).collect()
    }

    pub(crate) fn open_messages(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.open_menu(window, cx) {
            self.menu.page = Some(Page::Messages);
            cx.notify();
        }
    }

    pub(crate) fn messages_key(
        &self,
        event: &KeyDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        if self.menu.page != Some(Page::Messages) || event.keystroke.modifiers.modified() {
            return false;
        }
        let scroll = &self.menu.messages_scroll;
        let step = match event.keystroke.key.as_str() {
            "up" => self.config.ui.size * 2.,
            "down" => -self.config.ui.size * 2.,
            "pageup" => f32::from(scroll.bounds().size.height),
            "pagedown" => -f32::from(scroll.bounds().size.height),
            "home" => {
                scroll.set_offset(Point::default());
                0.
            }
            "end" => {
                scroll.scroll_to_bottom();
                0.
            }
            _ => return false,
        };
        if step != 0. {
            scroll.set_offset(scroll.offset() + point(px(0.), px(step)));
        }
        window.prevent_default();
        cx.stop_propagation();
        cx.notify();
        true
    }

    pub(crate) fn render_messages(&self, window: &Window) -> Stateful<Div> {
        let entries = self.messages();
        self.render_messages_content(&entries, None, window)
    }

    pub(crate) fn render_messages_paint(
        &self,
        entries: &[(Notice, String)],
        offset: Point<Pixels>,
        window: &Window,
    ) -> Stateful<Div> {
        self.render_messages_content(entries, Some(offset), window)
    }

    fn render_messages_content(
        &self,
        entries: &[(Notice, String)],
        offset: Option<Point<Pixels>>,
        window: &Window,
    ) -> Stateful<Div> {
        let active = offset.is_none();
        div()
            .id("message-history")
            .debug_selector(|| "message-history".into())
            .when(active, |list| {
                list.role(Role::List).aria_label(fl!("commands-messages"))
            })
            .min_h_0()
            .max_h((window.viewport_size().height - px(100.)).max(px(0.)))
            .when(active, |list| {
                list.overflow_y_scroll()
                    .track_scroll(&self.menu.messages_scroll)
            })
            .when(!active, |list| list.overflow_hidden())
            .child(
                div()
                    .relative()
                    .when_some(offset, |list, offset| list.top(offset.y))
                    .flex()
                    .flex_col()
                    .gap(px(12.))
                    .child(
                        div()
                            .id("message-history-title")
                            .when(active, |title| title.role(Role::Heading))
                            .child(fl!("commands-messages")),
                    )
                    .child(
                        div()
                            .text_color(rgb(self.theme.muted))
                            .child(fl!("messages-memory-only")),
                    )
                    .when(entries.is_empty(), |list| list.child(fl!("messages-empty")))
                    .children(entries.iter().map(|(notice, source)| {
                        let text = SharedString::from(notice.text.to_string());
                        div()
                            .id(("message", notice.id))
                            .when(active, |row| {
                                row.role(Role::ListItem).aria_label(text.clone())
                            })
                            .flex_none()
                            .flex()
                            .flex_col()
                            .gap(px(4.))
                            .child(
                                div()
                                    .text_color(rgb(self.theme.muted))
                                    .child(source.clone()),
                            )
                            .child(
                                div()
                                    .debug_selector({
                                        let text = text.clone();
                                        move || format!("message-text: {text}")
                                    })
                                    .child(text),
                            )
                    })),
            )
    }
}
