//! The section list and the footer: the crate draws them, and this window
//! answers them with its sections, its scroll position and its drafts.
use super::{SettingsWindow, scrollbar};
use crate::panel_resize::{PanelDrag, Side, handle};
use gpui::{prelude::*, *};
use spaceport_settings_window::navigation as nav;

impl SettingsWindow {
    pub(super) fn navigation(&self, cx: &mut Context<Self>) -> Stateful<Div> {
        let theme = &self.theme;
        let width = px(self.navigation_width.width(self.viewport_width));
        let resize = handle(
            "settings-navigation-resize",
            Side::start(),
            PanelDrag::SettingsNavigation,
            cx.listener(|this, _, _, cx| {
                this.navigation_width.reset();
                cx.notify();
            }),
        )
        .into_any_element();
        #[cfg(all(feature = "integration-test", target_os = "macos"))]
        let probe = |index: usize| Some(super::native::probe(index).into_any_element());
        #[cfg(not(all(feature = "integration-test", target_os = "macos")))]
        let probe = |_: usize| None;
        let view = cx.entity().downgrade();
        let column = nav::sections(
            theme,
            self.section,
            &self.navigation_scroll,
            probe,
            move |section, window, cx| {
                let _ = view.update(cx, |this, cx| this.select_section(section, window, cx));
            },
        );
        let sections = self.scroll_area(scrollbar::Pane::Sections, column, cx);
        nav::panel(
            theme,
            width,
            resize,
            self.nav_search.clone().into_any_element(),
            sections.into_any_element(),
        )
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
    }

    /// What this window is doing, as the footer reads it.
    fn footer_status(&self) -> nav::Status<'_> {
        nav::Status {
            drafts: nav::Drafts {
                layout: self.layout_intent.is_some(),
                sidebar: self.sidebar_dirty(),
                theme: self.theme_dirty(),
            },
            error: self.error.as_deref(),
            loading: self.persistence.loading(),
            message: self.status.as_deref(),
        }
    }

    #[cfg(test)]
    pub(super) fn footer_text(&self) -> String {
        nav::status_text(&self.footer_status())
    }

    pub(super) fn footer(&self, cx: &mut Context<Self>) -> Div {
        let status = self.footer_status();
        let view = cx.entity().downgrade();
        let saving = view.clone();
        nav::footer(
            &self.theme,
            &status,
            self.busy(),
            move |_, _, cx| {
                let _ = saving.update(cx, |this, cx| this.save_pending_drafts(cx));
            },
            move |_, _, cx| {
                let _ = view.update(cx, |this, cx| this.reload(cx));
            },
        )
    }
}
