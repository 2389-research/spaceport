// ABOUTME: Shares worktree picker geometry between active and departing content.
// ABOUTME: Keeps row actions and native search input out of passive paint.
use super::*;

impl SheprdWindow {
    pub(in crate::menu) fn render_existing_worktrees_paint(
        &self,
        paint: &Paint,
        cx: &mut Context<Self>,
    ) -> Div {
        let rows = div()
            .debug_selector(|| "open-worktree-list".into())
            .w_full()
            .flex_1()
            .min_h_0()
            .relative()
            .overflow_hidden()
            .children(paint.rows.iter().enumerate().map(|(slot, row)| {
                div()
                    .absolute()
                    .top(paint.offset + paint.height * slot as f32)
                    .left_0()
                    .right_0()
                    .child(self.worktree_row(row, false, paint.height, cx))
            }));
        self.worktree_content(
            paint.search.render().into_any_element(),
            paint.count.clone(),
            paint.empty.clone(),
            rows.into_any_element(),
        )
    }

    pub(in crate::menu) fn render_existing_worktrees(&self, cx: &mut Context<Self>) -> Div {
        let Some(picker) = &self.menu.worktree_open else {
            return div();
        };
        let rows = uniform_list(
            "open-worktrees",
            picker.filtered.len(),
            cx.processor(|this, range: std::ops::Range<usize>, _, cx| {
                range
                    .map(|row| this.render_existing_worktree_row(row, cx))
                    .collect()
            }),
        )
        .debug_selector(|| "open-worktree-list".into())
        .w_full()
        .track_scroll(&picker.scroll)
        .flex_1()
        .min_h_0();
        self.worktree_content(
            picker.search.clone().into_any_element(),
            paint::count(picker),
            self.worktree_empty(picker),
            rows.into_any_element(),
        )
    }

    fn worktree_content(
        &self,
        search: AnyElement,
        count: String,
        empty: Option<(bool, String)>,
        rows: AnyElement,
    ) -> Div {
        let mut panel = div().w_full().flex().flex_col().flex_1().min_h_0().child(
            div()
                .debug_selector(|| "open-worktree-search".into())
                .flex_none()
                .px(px(16.))
                .py(px(10.))
                .child(search)
                .child(
                    div()
                        .pt(px(6.))
                        .text_color(rgb(self.theme.muted))
                        .child(count),
                ),
        );
        if let Some((empty, message)) = empty {
            panel = panel.child(
                div()
                    .debug_selector({
                        move || {
                            if empty {
                                "open-worktree-empty"
                            } else {
                                "open-worktree-no-matches"
                            }
                            .into()
                        }
                    })
                    .flex_1()
                    .p(px(16.))
                    .text_color(rgb(self.theme.muted))
                    .child(message),
            );
        } else {
            panel = panel.child(rows);
        }
        panel
    }

    fn render_existing_worktree_row(&self, row: usize, cx: &mut Context<Self>) -> AnyElement {
        let Some(picker) = &self.menu.worktree_open else {
            return div().into_any_element();
        };
        let Some(entry) = picker.entry(row) else {
            return div().into_any_element();
        };
        let paint = paint::row(entry, row, picker.selected);
        self.worktree_row(
            &paint,
            true,
            px(self.config.ui.line_height() * 2. + 14.),
            cx,
        )
        .into_any_element()
    }

    fn worktree_row(
        &self,
        paint: &paint::Row,
        active: bool,
        height: Pixels,
        cx: &mut Context<Self>,
    ) -> Stateful<Div> {
        let row = paint.index;
        div()
            .id(row)
            .debug_selector(move || format!("open-worktree-row-{row}"))
            .w_full()
            .h(height)
            .px(px(16.))
            .py(px(5.))
            .flex()
            .flex_col()
            .min_w_0()
            .when(paint.selected, |row| row.bg(rgb(self.theme.active)))
            .child(
                div()
                    .w_full()
                    .flex()
                    .min_w_0()
                    .gap(px(8.))
                    .child(
                        div()
                            .flex_1()
                            .min_w_0()
                            .truncate()
                            .child(paint.label.clone()),
                    )
                    .when(!paint.status.is_empty(), |line| {
                        line.child(
                            div()
                                .debug_selector(move || format!("open-worktree-row-status-{row}"))
                                .flex_none()
                                .text_color(rgb(self.theme.muted))
                                .child(paint.status.clone()),
                        )
                    }),
            )
            .child(
                div()
                    .w_full()
                    .truncate()
                    .text_color(rgb(self.theme.muted))
                    .child(paint.path.clone()),
            )
            .when(active, |element| {
                element
                    .cursor_pointer()
                    .hover(|row| row.bg(rgb(self.theme.active)))
                    .on_click(cx.listener(move |this, _, window, cx| {
                        cx.stop_propagation();
                        if this.menu.page != Some(Page::Dialog(WorkspaceAction::OpenWorktree))
                            || this.menu.opening.is_some()
                        {
                            return;
                        }
                        if let Some(picker) = &mut this.menu.worktree_open {
                            picker.selected = row;
                        }
                        this.submit_workspace_dialog(window, cx);
                    }))
            })
    }
}
