// ABOUTME: Shares theme-choice rows between interactive and departing picker content.
// ABOUTME: Omits pointer and mutation handlers from frozen rows and favourite controls.
use super::{paint::Row, *};

impl SheprdWindow {
    pub(super) fn theme_picker_row(
        &self,
        row: &Row,
        height: Pixels,
        active: bool,
        cx: &mut Context<Self>,
    ) -> Stateful<Div> {
        let index = row.index;
        let name = row.name.clone();
        div()
            .id(index)
            .debug_selector(move || format!("theme-row-{index}"))
            // As in the palette, the fill is the row.
            .w_full()
            .h(height)
            .px(px(16.))
            .flex()
            .items_center()
            .gap(px(8.))
            .when(row.selected, |view| view.bg(rgb(self.theme.active)))
            // The dimmed colour keeps the rule visible on a selected row.
            .when(row.last_favorite, |view| {
                view.border_b_1()
                    .border_color(rgba((self.theme.muted << 8) | 0x80))
            })
            .child(
                div()
                    .debug_selector({
                        let name = name.clone();
                        move || format!("theme-name-{name}")
                    })
                    .flex_1()
                    .min_w_0()
                    .truncate()
                    .child(name.clone()),
            )
            .when(row.current, |view| {
                view.child(
                    div()
                        .text_color(rgb(self.theme.muted))
                        .child(fl!("theme-picker-current")),
                )
            })
            // The outline follows pointer or key selection; a star stays.
            .when(row.favorite || row.selected, |view| {
                view.child(self.favorite_star(index, &name, row.favorite, active, cx))
            })
            .when(active, |view| {
                view.cursor_pointer()
                    .hover(|style| style.bg(rgb(self.theme.active)))
                    .on_hover(cx.listener(move |this, hovered, _, cx| {
                        if *hovered {
                            if let Some(picker) = &mut this.menu.themes {
                                if picker.accepting {
                                    return;
                                }
                                picker.selected = index;
                            }
                            this.preview_picker_selection(cx);
                        }
                    }))
                    .on_click(cx.listener(move |this, _, _, cx| this.apply_picker_theme(&name, cx)))
            })
    }
}
