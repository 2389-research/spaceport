// ABOUTME: Shares active and departing pane display templates.
// ABOUTME: Keeps display code separate from target and worker ownership.
use super::*;
use crate::pane_menu::{Paint, paint::Kind};

impl SheprdWindow {
    pub(in crate::pane_menu) fn render_pane_move_pending(&self) -> Div {
        div().flex().flex_col().child(
            div()
                .debug_selector(|| "pane-move-pending".into())
                .p(px(8.))
                .text_color(rgb(self.theme.muted))
                .child(fl!("pane-menu-moving")),
        )
    }

    /// Fills the menu's `panel` with the picker. The heading and rows are the
    /// panel's own children, which its scroll handle addresses by index, so
    /// a row the keyboard selects in a long list scrolls into view.
    pub(crate) fn render_move_picker(
        &self,
        panel: Stateful<Div>,
        cx: &mut Context<Self>,
    ) -> Stateful<Div> {
        let Some(paint) = self.pane_menu_paint(cx) else {
            return panel;
        };
        self.render_pane_move_content(panel, &paint, self.menu.pane.as_ref(), cx)
    }

    pub(crate) fn render_pane_move_paint(
        &self,
        panel: Stateful<Div>,
        paint: &Paint,
        cx: &mut Context<Self>,
    ) -> Stateful<Div> {
        self.render_pane_move_content(panel, paint, None, cx)
    }

    fn render_pane_move_content(
        &self,
        panel: Stateful<Div>,
        paint: &Paint,
        live: Option<&PaneMenu>,
        cx: &mut Context<Self>,
    ) -> Stateful<Div> {
        if matches!(paint.kind, Kind::Moving) {
            return panel.child(self.render_pane_move_pending());
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
                .child(fl!("pane-menu-move-heading"))
                .into_any_element(),
        ];
        let row = |index: usize| {
            div()
                .id(("pane-move-destination", index))
                .flex_none()
                .debug_selector(move || format!("pane-move-{index}"))
                .min_h(px(self.config.ui.line_height() + 12.))
                .px(px(8.))
                .row()
                .items_center()
                .overflow_hidden()
                .when(live.is_some(), |row| row.cursor_pointer())
                .when(paint.selected == Some(index), |row| {
                    row.bg(rgb(self.theme.active))
                })
                .when(live.is_some(), |row| {
                    row.hover(|row| row.bg(rgb(self.theme.active)))
                        .on_hover(cx.listener(move |this, hovered, _, cx| {
                            if *hovered && let Some(pane) = &mut this.menu.pane {
                                pane.selected = Some(index);
                                cx.notify();
                            }
                        }))
                })
        };
        let destinations = live.map(|pane| pane.target.destinations());
        for (index, choice) in choices.iter().enumerate() {
            let destination = destinations
                .as_ref()
                .and_then(|destinations| destinations.get(index))
                .map(|choice| choice.destination.clone());
            children.push(
                row(index)
                    .child(destination_label(
                        index,
                        choice.workspace.clone(),
                        choice.place.clone(),
                    ))
                    .when_some(destination, |row, destination| {
                        row.on_click(cx.listener(move |this, event: &ClickEvent, window, cx| {
                            // The picker replaces the menu under the pointer, and
                            // snapping it into the window can shift its rows, so
                            // a double-click's second click would otherwise pick
                            // whatever row it lands on.
                            if event.click_count() > 1 {
                                return;
                            }
                            this.move_pane(destination.clone(), window, cx)
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
            Some(pane) => panel
                .track_scroll(&pane.scroll)
                .flex()
                .flex_col()
                .children(children),
            None => panel.overflow_hidden().flex().flex_col().child(
                div()
                    .flex()
                    .flex_col()
                    .children(children)
                    .flex_none()
                    .relative()
                    .top(paint.offset.y),
            ),
        }
    }
}
