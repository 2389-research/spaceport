// ABOUTME: Implements Settings navigation interactions.
// ABOUTME: Connects native controls to Settings state and persistence.
//! The section list and the footer: grouped sections under a search box, and
//! the save status line.
// i18n: migrated
use super::*;

const NAVIGATION_PADDING: f32 = 12.;
const NAVIGATION_BORDER: f32 = 1.;
const ITEM_PADDING: f32 = 10.;
const ITEM_GAP: f32 = 10.;
const ICON_SIZE: f32 = 17.;

impl SettingsWindow {
    pub(super) fn navigation(&mut self, window: &Window, cx: &mut Context<Self>) -> Stateful<Div> {
        use crate::panel_resize::{PanelDrag, Side, handle};
        let font = crate::config::font_for(&self.config.ui);
        let widest = Section::ALL
            .iter()
            .map(|section| {
                let label = section.label();
                let mut run = window.text_style().to_run(label.len());
                run.font = font.clone();
                window
                    .text_system()
                    .shape_line(label.into(), px(self.config.ui.size), &[run], None)
                    .width
            })
            .max()
            .unwrap_or_default();
        let needed = f32::from(widest).ceil()
            + NAVIGATION_PADDING * 2.
            + NAVIGATION_BORDER
            + ITEM_PADDING * 2.
            + ITEM_GAP
            + ICON_SIZE;
        self.navigation_width
            .set_default(needed.max(crate::panel_resize::SETTINGS_NAVIGATION.width(0.)));
        let theme = &self.theme;
        div()
            .id("settings-navigation")
            .role(Role::Navigation)
            .aria_label(fl!("settings-sections-label"))
            .relative()
            .w(px(self.navigation_width.width(self.viewport_width)))
            .on_drag_move(
                cx.listener(|this, event: &DragMoveEvent<PanelDrag>, _, cx| {
                    if *event.drag(cx) == PanelDrag::SettingsNavigation
                        && this.navigation_width.drag(
                            Side::start(),
                            event.bounds,
                            event.event.position.x,
                        )
                    {
                        cx.notify();
                    }
                }),
            )
            .child(handle(
                "settings-navigation-resize",
                Side::start(),
                PanelDrag::SettingsNavigation,
                cx.listener(|this, _, _, cx| {
                    this.navigation_width.reset();
                    cx.notify();
                }),
            ))
            .h_full()
            .flex_none()
            .flex()
            .flex_col()
            .p(px(NAVIGATION_PADDING))
            .gap(px(2.))
            .bg(rgb(mix(theme.surface, theme.background, 25)))
            .border_e(px(NAVIGATION_BORDER))
            .border_color(rgb(theme.active))
            .child(
                div()
                    .debug_selector(|| "settings-nav-search".into())
                    .mb(px(8.))
                    .child(self.nav_search.clone()),
            )
            .child(
                self.scroll_area(
                    scrollbar::Pane::Sections,
                    div()
                        .id("settings-sections")
                        .debug_selector(|| "settings-sections".into())
                        // Picking a section swaps the page beside it, as a tab does.
                        .role(Role::TabList)
                        .aria_label(fl!("settings-sections"))
                        .aria_orientation(Orientation::Vertical)
                        .flex_1()
                        .min_h_0()
                        .flex()
                        .flex_col()
                        .gap(px(2.))
                        .overflow_y_scroll()
                        .track_scroll(&self.navigation_scroll)
                        // Flat, so `Section::nav_item` can scroll to a page.
                        .children(std::iter::once(None).chain(Group::ALL.map(Some)).flat_map(
                            |group| {
                                group
                                    .map(|group| self.group_label(group).into_any_element())
                                    .into_iter()
                                    .chain(
                                        Section::ALL
                                            .iter()
                                            .copied()
                                            .enumerate()
                                            .filter(move |(_, section)| section.group() == group)
                                            .map(|(index, section)| {
                                                self.navigation_row(index, section, cx)
                                                    .into_any_element()
                                            })
                                            .collect::<Vec<_>>(),
                                    )
                            },
                        )),
                    cx,
                ),
            )
    }

    fn group_label(&self, group: Group) -> Div {
        div()
            .debug_selector(move || format!("settings-group-{}", group.slug()))
            .px(px(10.))
            .pt(px(10.))
            .pb(px(4.))
            .text_size(rems(0.625))
            .text_color(rgb(self.theme.subtext()))
            .child(group.label().to_uppercase())
    }

    fn navigation_row(
        &self,
        index: usize,
        section: Section,
        cx: &mut Context<Self>,
    ) -> Stateful<Div> {
        let theme = &self.theme;
        let selected = self.section == section;
        let id: SharedString = section.selector().into();
        let mut context = KeyContext::default();
        context.set("settings-section", id.clone());
        div()
            .key_context(context)
            .child(search::track_focus_bounds(id, px(0.)))
            .id(("settings-section", index))
            .map(|row| ui::focusable(theme, row, true))
            .role(Role::Tab)
            .aria_label(section.label())
            .aria_selected(selected)
            .relative()
            .map(|row| {
                #[cfg(all(feature = "integration-test", target_os = "macos"))]
                let row = row.child(native::probe(index));
                row
            })
            .debug_selector(move || section.selector())
            .row()
            .items_center()
            .gap(px(ITEM_GAP))
            .px(px(ITEM_PADDING))
            .py(px(6.))
            .rounded(px(corners::CONTROL))
            .cursor_pointer()
            .when(selected, |el| el.bg(rgb(theme.primary_wash())))
            .hover(|el| el.bg(rgb(theme.active)))
            .child(
                svg()
                    .path(section.icon())
                    .size(px(ICON_SIZE))
                    .flex_none()
                    .text_color(rgb(if selected {
                        theme.primary()
                    } else {
                        theme.subtext()
                    })),
            )
            .child(
                div()
                    .debug_selector(move || format!("{}-label", section.selector()))
                    .flex_1()
                    .min_w_0()
                    .truncate()
                    .child(section.label()),
            )
            .on_click(
                cx.listener(move |this, _, window, cx| this.select_section(section, window, cx)),
            )
    }

    fn pending_drafts(&self) -> Vec<String> {
        [
            (self.theme_dirty(), fl!("settings-draft-theme")),
            (self.layout_intent.is_some(), fl!("settings-draft-layout")),
            (self.sidebar_dirty(), fl!("settings-draft-sidebar")),
        ]
        .into_iter()
        .filter_map(|(dirty, name)| dirty.then_some(name))
        .collect()
    }

    pub(super) fn footer_text(&self) -> String {
        let pending = self.pending_drafts();
        if let Some(error) = &self.error {
            error.clone()
        } else if self.persistence.loading() {
            fl!("settings-loading")
        } else if let Some(status) = &self.status {
            status.clone()
        } else if pending.is_empty() {
            fl!("settings-saved")
        } else {
            fl!("settings-unsaved", drafts = pending.join(", "))
        }
    }

    pub(super) fn footer(&self, cx: &mut Context<Self>) -> Div {
        let pending = self.pending_drafts();
        let text = self.footer_text();
        div()
            .flex_none()
            .min_h(px(36.))
            .px(px(16.))
            .row()
            .items_center()
            .gap(px(12.))
            .border_t_1()
            .border_color(rgb(self.theme.active))
            .text_size(rems(0.6875))
            .when(!pending.is_empty() && self.error.is_none(), |footer| {
                footer.child(
                    div()
                        .size(px(7.))
                        .rounded_full()
                        .bg(rgb(self.theme.ink(0xe3b341))),
                )
            })
            .child(
                div()
                    .id("settings-footer-status")
                    .debug_selector(|| "settings-footer-status".into())
                    .role(Role::Status)
                    .aria_label(SharedString::from(text.clone()))
                    .flex_1()
                    .min_w_0()
                    .child(text),
            )
            .when(!pending.is_empty(), |footer| {
                footer.child(
                    self.control_choice(
                        "settings-footer-save",
                        fl!("settings-save-now"),
                        false,
                        !self.busy(),
                    )
                    .debug_selector(|| "settings-footer-save".into())
                    .role(Role::Button)
                    .aria_label(fl!("settings-save-now"))
                    .flex_none()
                    .px(px(10.))
                    .py(px(3.))
                    .on_click(cx.listener(|this, _, _, cx| this.save_pending_drafts(cx))),
                )
            })
            .child(
                self.control_choice(
                    "settings-footer-reload",
                    fl!("settings-reload"),
                    false,
                    true,
                )
                .debug_selector(|| "settings-footer-reload".into())
                .role(Role::Button)
                .aria_label(fl!("settings-reload"))
                .flex_none()
                .px(px(10.))
                .py(px(3.))
                .on_click(cx.listener(|this, _, _, cx| this.reload(cx))),
            )
    }
}
