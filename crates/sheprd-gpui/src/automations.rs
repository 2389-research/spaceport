//! The retained activity destination while Core has no job observation feed.
//! It reports unavailability without starting a scheduler or reading host scripts.

use crate::{SheprdWindow, i18n::fl};
use gpui::{prelude::*, *};

impl SheprdWindow {
    pub(crate) fn render_automations_section(&self, _: &mut Context<Self>) -> AnyElement {
        let theme = &self.theme;
        div()
            .id("automations-section")
            .debug_selector(|| "automations-section".into())
            .flex()
            .flex_col()
            .flex_1()
            .size_full()
            .min_w_0()
            .min_h_0()
            .bg(rgb(theme.background))
            .text_color(rgb(theme.foreground))
            .text_size(px(self.config.ui.size))
            .child(
                div()
                    .px_5()
                    .py_3()
                    .border_b_1()
                    .border_color(rgb(theme.active))
                    .child(fl!("sidebar-activity-automations")),
            )
            .child(
                div()
                    .id("automation-activity-unavailable")
                    .debug_selector(|| "automation-activity-unavailable".into())
                    .p_5()
                    .text_color(rgb(theme.subtext()))
                    .child(fl!("automations-unavailable")),
            )
            .into_any_element()
    }
}
