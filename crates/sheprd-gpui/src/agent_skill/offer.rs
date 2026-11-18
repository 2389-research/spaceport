// ABOUTME: Presents the managed skill offer through one active and static template.
// ABOUTME: Leaves installation and dismissal actions out of departing content.
use crate::{SheprdWindow, i18n::fl};
use gpui::{prelude::*, *};

impl SheprdWindow {
    pub(crate) fn render_agent_skill_offer(&self, cx: &mut Context<Self>) -> Div {
        self.render_agent_skill_offer_mode(true, cx)
    }

    pub(crate) fn render_agent_skill_offer_mode(
        &self,
        active: bool,
        cx: &mut Context<Self>,
    ) -> Div {
        let theme = &self.theme;
        let button = |id: &'static str, label: String, primary: bool| {
            div()
                .id(id)
                .debug_selector(move || id.into())
                .p(px(8.))
                .rounded(px(crate::config::corners::CONTROL))
                .when(primary, |button| button.bg(rgb(theme.active)))
                .when(active, |button| {
                    button
                        .cursor_pointer()
                        .when(!primary, |button| button.hover(|s| s.bg(rgb(theme.active))))
                })
                .child(label)
        };
        div()
            .debug_selector(|| "agent-skill-offer".into())
            .child(
                div()
                    .p(px(8.))
                    .child(fl!("window-notices-skill-offer-title")),
            )
            .child(
                div()
                    .p(px(8.))
                    .child(fl!("window-notices-skill-offer-body")),
            )
            .child(
                div()
                    .p(px(8.))
                    .text_color(rgb(theme.muted))
                    .child(fl!("window-notices-skill-offer-detail")),
            )
            .child(
                div()
                    .flex()
                    .flex_wrap()
                    .gap(px(8.))
                    .p(px(8.))
                    .child(
                        button(
                            "agent-skill-install",
                            fl!("window-notices-skill-offer-install"),
                            true,
                        )
                        .when(active, |button| {
                            button.on_click(cx.listener(|this, _, window, cx| {
                                cx.stop_propagation();
                                this.install_browser_skill(window, cx);
                            }))
                        }),
                    )
                    .child(
                        button(
                            "agent-skill-decline",
                            fl!("window-notices-skill-offer-decline"),
                            false,
                        )
                        .when(active, |button| {
                            button.on_click(cx.listener(|this, _, window, cx| {
                                cx.stop_propagation();
                                this.dismiss_menu(window, cx);
                            }))
                        }),
                    ),
            )
    }
}
