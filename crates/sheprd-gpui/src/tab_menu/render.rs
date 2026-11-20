// ABOUTME: Shares tab action, rename, and destination popup templates.
// ABOUTME: Keeps presentation separate from captured tab targets and transport work.
use super::paint::{Draft, Kind};
use super::*;

impl SheprdWindow {
    fn render_tab_move_pending(&self) -> Div {
        div().flex().flex_col().child(
            div()
                .debug_selector(|| "tab-move-pending".into())
                .p(px(8.))
                .text_color(rgb(self.theme.muted))
                .child(fl!("tab-menu-moving")),
        )
    }

    /// Fills the menu's `panel` with the workspace picker. The heading and
    /// rows are the panel's own children, which its scroll handle addresses by
    /// index, so a row the keyboard selects in a long list scrolls into view.
    pub(crate) fn render_tab_move_picker(
        &self,
        panel: Stateful<Div>,
        cx: &mut Context<Self>,
    ) -> Stateful<Div> {
        let Some(paint) = self.tab_menu_paint(cx) else {
            return panel;
        };
        self.render_tab_move_content(panel, &paint, self.menu.tab.as_ref(), cx)
    }

    pub(crate) fn render_tab_move_paint(
        &self,
        panel: Stateful<Div>,
        paint: &Paint,
        cx: &mut Context<Self>,
    ) -> Stateful<Div> {
        self.render_tab_move_content(panel, paint, None, cx)
    }

    fn render_tab_move_content(
        &self,
        panel: Stateful<Div>,
        paint: &Paint,
        live: Option<&TabMenu>,
        cx: &mut Context<Self>,
    ) -> Stateful<Div> {
        if matches!(paint.kind, Kind::Moving) {
            return panel.child(self.render_tab_move_pending());
        }
        let Kind::Destinations(choices) = &paint.kind else {
            return panel;
        };
        let mut children = vec![
            div()
                .flex_none()
                .px(px(8.))
                .py(px(4.))
                .text_color(rgb(self.theme.muted))
                .child(fl!("tab-menu-move-heading"))
                .into_any_element(),
        ];
        for (index, place) in choices.iter().enumerate() {
            let destination = live
                .and_then(|tab| tab.target.choices.get(index))
                .map(|choice| choice.destination.clone());
            children.push(
                div()
                    .id(("tab-move-destination", index))
                    .flex_none()
                    .debug_selector(move || format!("tab-move-{index}"))
                    .min_h(px(self.config.ui.line_height() + 12.))
                    .px(px(8.))
                    .flex()
                    .items_center()
                    .overflow_hidden()
                    .when(live.is_some(), |row| row.cursor_pointer())
                    .when(paint.selected == Some(index), |row| {
                        row.bg(rgb(self.theme.active))
                    })
                    .child(
                        div()
                            .debug_selector(move || format!("tab-move-place-{index}"))
                            .min_w(px(0.))
                            .truncate()
                            .child(place.clone()),
                    )
                    .when_some(destination, |row, destination| {
                        row.hover(|row| row.bg(rgb(self.theme.active)))
                            .on_hover(cx.listener(move |this, hovered, _, cx| {
                                if *hovered && let Some(tab) = &mut this.menu.tab {
                                    tab.selected = Some(index);
                                    cx.notify();
                                }
                            }))
                            .on_click(cx.listener(move |this, event: &ClickEvent, window, cx| {
                                // The picker replaces the menu under the pointer, and
                                // snapping it into the window can shift its rows, so
                                // a double-click's second click would otherwise pick
                                // whatever row it lands on.
                                if event.click_count() > 1 {
                                    return;
                                }
                                this.move_tab(destination.clone(), window, cx)
                            }))
                    })
                    .into_any_element(),
            );
        }
        if let Some(error) = &paint.error {
            children.push(
                div()
                    .flex_none()
                    .p(px(8.))
                    .child(error.clone())
                    .into_any_element(),
            );
        }
        match live {
            Some(tab) => panel
                .track_scroll(&tab.scroll)
                .flex()
                .flex_col()
                .children(children),
            None => panel.overflow_hidden().flex().flex_col().child(
                div()
                    .flex_none()
                    .relative()
                    .top(paint.offset.y)
                    .flex()
                    .flex_col()
                    .children(children),
            ),
        }
    }

    pub(crate) fn render_tab_menu(&self, cx: &mut Context<Self>) -> Div {
        let Some(paint) = self.tab_menu_paint(cx) else {
            return div();
        };
        self.render_tab_content(&paint, self.menu.tab.as_ref(), cx)
    }

    pub(crate) fn render_tab_menu_paint(&self, paint: &Paint, cx: &mut Context<Self>) -> Div {
        self.render_tab_content(paint, None, cx)
            .relative()
            .top(paint.offset.y)
    }

    fn render_tab_content(
        &self,
        paint: &Paint,
        live: Option<&TabMenu>,
        cx: &mut Context<Self>,
    ) -> Div {
        let theme = &self.theme;
        let mut body = div().flex().flex_col();
        if matches!(paint.kind, Kind::Moving) {
            body = self.render_tab_move_pending();
        } else if let Kind::Actions(labels) = &paint.kind {
            let actions = live.map(TabMenu::actions);
            for (index, label) in labels.iter().enumerate() {
                let action = actions
                    .as_ref()
                    .and_then(|actions| actions.get(index))
                    .copied();
                body = body.child(
                    div()
                        .id(("tab-menu-action", index))
                        .debug_selector(move || format!("tab-menu-{index}"))
                        .min_h(px(self.config.ui.line_height() + 12.))
                        .px(px(8.))
                        .flex()
                        .items_center()
                        .when(live.is_some(), |row| row.cursor_pointer())
                        .when(paint.selected == Some(index), |row| {
                            row.bg(rgb(theme.active))
                        })
                        .child(
                            div()
                                .debug_selector(move || format!("tab-menu-label-{index}"))
                                .min_w_0()
                                .truncate()
                                .child(label.clone()),
                        )
                        .when_some(action, |row, action| {
                            row.hover(|row| row.bg(rgb(theme.active)))
                                .on_hover(cx.listener(move |this, hovered, _, cx| {
                                    if *hovered && let Some(tab) = &mut this.menu.tab {
                                        tab.selected = Some(index);
                                        cx.notify();
                                    }
                                }))
                                .on_click(cx.listener(move |this, _, window, cx| {
                                    this.activate_tab_menu(action, window, cx)
                                }))
                        }),
                );
            }
        } else if let Kind::Rename(draft) = &paint.kind {
            let pending = matches!(draft, Draft::Pending(_));
            let input = match draft {
                Draft::Pending(text) => text
                    .as_ref()
                    .map(|text| div().child(text.clone()).into_any_element()),
                Draft::Editing(field) => live
                    .and_then(|tab| tab.input.as_ref())
                    .map(|input| input.clone().into_any_element())
                    .or_else(|| {
                        field
                            .as_deref()
                            .map(|field| field.render().into_any_element())
                    }),
            };
            body = body
                .p(px(8.))
                .gap(px(12.))
                .child(
                    div()
                        .font_weight(FontWeight::SEMIBOLD)
                        .child(fl!("tab-menu-rename-title")),
                )
                .when_some(input, |body, input| body.child(input))
                .child(
                    div()
                        .flex()
                        .justify_end()
                        .gap(px(8.))
                        .child(
                            div()
                                .id("tab-rename-cancel")
                                .debug_selector(|| "tab-rename-cancel".into())
                                .p(px(6.))
                                .child(fl!("tab-menu-cancel"))
                                .when(live.is_some(), |button| {
                                    button.cursor_pointer().on_click(cx.listener(
                                        |this, _, window, cx| this.dismiss_menu(window, cx),
                                    ))
                                }),
                        )
                        .child(
                            div()
                                .id("tab-rename-submit")
                                .debug_selector(|| "tab-rename-submit".into())
                                .p(px(6.))
                                .rounded(px(crate::config::corners::CONTROL))
                                .bg(rgb(theme.active))
                                .child(if pending {
                                    fl!("tab-menu-renaming")
                                } else {
                                    fl!("tab-menu-rename-submit")
                                })
                                .when(live.is_some(), |button| {
                                    button.cursor_pointer().on_click(cx.listener(
                                        |this, _, window, cx| this.submit_tab_rename(window, cx),
                                    ))
                                }),
                        ),
                );
        }
        body.when_some(paint.error.clone(), |body, error| {
            // No width of its own, so the labels set the panel's width and
            // the message wraps inside it.
            body.child(div().w_0().min_w_full().p(px(8.)).child(error))
        })
    }
}
