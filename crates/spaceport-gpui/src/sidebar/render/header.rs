//! Shared headings for the sidebar's workspace and agent sections.

use crate::{
    SpaceportWindow,
    config::{FontConfig, Theme},
    direction::Logical,
    sidebar::{label_text, layout::SidebarLook, line_height},
};
use gpui::{prelude::*, *};

/// A section's heading: `key` names its debug selectors, `label` is its
/// text, cased as the look's headers are.
pub(in crate::sidebar) fn header(
    key: &'static str,
    label: &str,
    font: &FontConfig,
    theme: &Theme,
    look: SidebarLook,
) -> Div {
    div()
        .debug_selector(|| format!("header-{key}"))
        .flex_none()
        .h(px(line_height(font) + 2. * look.density.header_padding()))
        .px(px(look.content_x()))
        .row()
        .items_center()
        .text_size(px(font.size))
        .text_color(rgb(theme.muted))
        .child(
            div()
                .min_w_0()
                .truncate()
                .debug_selector(|| format!("header-label-{key}"))
                .child(label_text(&look.header_label(label))),
        )
}

impl SpaceportWindow {
    pub(crate) fn set_workspaces_expanded(
        &mut self,
        expanded: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.sidebar_workspaces_expanded == expanded {
            return;
        }
        if !expanded {
            self.cancel_workspace_drag(cx);
            self.cancel_device_drag(cx);
            if matches!(self.sidebar_drag, Some(crate::sidebar::SidebarDrag::Split)) {
                self.sidebar_drag = None;
            }
            if self.sidebar_search.focus(cx).contains_focused(window, cx) {
                window.focus(&self.focus, cx);
            }
        }
        self.sidebar_workspaces_expanded = expanded;
        cx.notify();
    }

    pub(in crate::sidebar) fn workspace_disclosure(
        &self,
        button: Div,
        expanded: bool,
        cx: &Context<Self>,
    ) -> Stateful<Div> {
        let toggle = move |this: &mut Self, window: &mut Window, cx: &mut Context<Self>| {
            this.set_workspaces_expanded(!expanded, window, cx);
        };
        let foreground = self.theme.foreground;
        let button = button
            .id("workspaces-disclosure")
            .debug_selector(|| "workspaces-disclosure".into())
            .cursor_pointer()
            .text_color(rgb(self.theme.muted))
            .hover(move |style| style.text_color(rgb(foreground)))
            .child(if expanded {
                "▾"
            } else {
                crate::direction::glyph("▸")
            });
        crate::sidebar::accessibility::expandable(
            self.sidebar_button(button, crate::i18n::fl!("sidebar-workspaces-label")),
            expanded,
            cx.entity().downgrade(),
            toggle,
        )
        .on_click(cx.listener(move |this, _, window, cx| toggle(this, window, cx)))
    }

    pub(super) fn agent_section_heading(
        &self,
        section: crate::sidebar::AgentSection,
        look: SidebarLook,
        cx: &Context<Self>,
    ) -> Stateful<Div> {
        let (key, label) = section.heading();
        let expanded = !self.agent_section_folds.contains(&section);
        let foreground = self.theme.foreground;
        let heading = header(key, &label, &self.config.sidebar, &self.theme, look)
            .id(key)
            .justify_between()
            .cursor_pointer()
            .hover(move |style| style.text_color(rgb(foreground)))
            .child(div().flex_none().child(label_text(if expanded {
                "▾"
            } else {
                crate::direction::glyph("▸")
            })));
        let toggle = move |this: &mut Self, _: &mut Window, cx: &mut Context<Self>| {
            if !this.agent_section_folds.remove(&section) {
                this.agent_section_folds.insert(section);
            }
            cx.notify();
        };
        crate::sidebar::accessibility::expandable(
            self.sidebar_button(heading, label),
            expanded,
            cx.entity().downgrade(),
            toggle,
        )
        .on_click(cx.listener(move |this, _, window, cx| toggle(this, window, cx)))
    }
}
