//! The activity bar's Usage section: reported plan limits for each host.
//! An absent authoritative feed is shown as unavailable.

use super::{Host, text, ui::Ui};
use crate::{
    SheprdWindow,
    i18n::{Localize, fl},
};
use gpui::{prelude::*, *};
use std::time::SystemTime;

const CARD_WIDTH: f32 = 320.;

impl SheprdWindow {
    pub(crate) fn render_usage_section(&self, _cx: &mut Context<Self>) -> AnyElement {
        let theme = &self.theme;
        let font = &self.config.ui;
        let now = SystemTime::now();
        let header = div()
            .debug_selector(|| "usage-section-header".into())
            .flex_none()
            .px_5()
            .py_3()
            .flex()
            .items_center()
            .gap_3()
            .border_b_1()
            .border_color(rgb(theme.active))
            .child(
                div()
                    .text_size(px(font.size + 3.))
                    .font_weight(FontWeight::SEMIBOLD)
                    .child(fl!("usage-section-title")),
            )
            .child(div().flex_1());
        let mut body = div().p_5().flex().flex_col().gap_5();
        if !self.config.usage.show {
            body = body.child(
                div()
                    .debug_selector(|| "usage-section-off".into())
                    .text_color(rgb(theme.muted))
                    .child(fl!("usage-off")),
            );
        }
        let ui = Ui {
            theme: self.theme.clone(),
            font_size: font.size,
            now,
        };
        for (index, endpoint) in self.endpoints.iter().enumerate() {
            let Some(host) = Host::of(&endpoint.connection.target).filter(|_| endpoint.enabled)
            else {
                continue;
            };
            if !self.config.usage.show {
                break;
            }
            body = body.child(self.usage_host(index, &endpoint.label, &host, &ui));
        }
        div()
            .id("usage-section")
            .debug_selector(|| "usage-section".into())
            .flex()
            .flex_col()
            .flex_1()
            .size_full()
            .min_w_0()
            .min_h_0()
            .bg(rgb(theme.background))
            .text_color(rgb(theme.foreground))
            .text_size(px(font.size))
            .child(header)
            .child(
                div()
                    .id("usage-section-scroll")
                    .flex_1()
                    .min_h_0()
                    .overflow_y_scroll()
                    .child(body),
            )
            .into_any_element()
    }

    fn usage_host(&self, index: usize, label: &str, host: &Host, ui: &Ui) -> Div {
        let theme = &self.theme;
        let small = ui.small();
        let entry = self.usage.entry(host);
        let updated = entry
            .and_then(|entry| entry.updated)
            .map(|at| text::updated(ui.now.duration_since(at).unwrap_or_default()))
            .unwrap_or_else(|| fl!("usage-unavailable"));
        let readings = entry
            .map(|entry| entry.tabs(&self.config.usage))
            .unwrap_or_default();
        let selector = format!("usage-section-host-{index}");
        let mut cards = div().flex().flex_wrap().gap_3();
        if readings.is_empty() {
            cards = cards.child(
                div()
                    .debug_selector(|| "usage-section-unavailable".into())
                    .text_color(rgb(theme.muted))
                    .child(fl!("usage-core-unavailable-detail")),
            );
        }
        for reading in readings {
            let meta = reading.provider.meta();
            let account = reading
                .report
                .as_ref()
                .map(|report| report.account.clone())
                .unwrap_or_default();
            let mut card = div()
                .w(px(CARD_WIDTH))
                .flex_none()
                .p_2()
                .flex()
                .flex_col()
                .gap_1()
                .rounded(px(crate::config::corners::PANEL))
                .bg(rgb(theme.surface))
                .child(
                    div()
                        .px(px(8.))
                        .pt(px(4.))
                        .flex()
                        .items_center()
                        .gap_2()
                        .child(
                            div()
                                .flex_1()
                                .font_weight(FontWeight::SEMIBOLD)
                                .child(meta.name),
                        )
                        .children(account.plan.map(|plan| {
                            div()
                                .text_size(small)
                                .text_color(rgb(theme.muted))
                                .child(text::words(&plan))
                        })),
                );
            if let Some(error) = reading.error.as_deref().map(Localize::localize) {
                card = card.child(
                    div()
                        .px(px(8.))
                        .text_size(small)
                        .text_color(rgb(theme.ink(theme.palette[3])))
                        .child(error),
                );
            }
            if let Some(report) = reading.report.as_ref() {
                card = card.child(super::details::render(report, ui));
            }
            cards = cards.child(card);
        }
        div()
            .debug_selector(move || selector.clone())
            .flex()
            .flex_col()
            .gap_2()
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .child(
                        svg()
                            .path("icons/computer.svg")
                            .size(px(13.))
                            .text_color(rgb(theme.subtext())),
                    )
                    .child(
                        div()
                            .font_weight(FontWeight::SEMIBOLD)
                            .child(label.to_owned()),
                    )
                    .child(
                        div()
                            .text_size(small)
                            .text_color(rgb(theme.muted))
                            .child(updated),
                    )
                    .when_some(
                        entry
                            .and_then(|entry| entry.error.as_deref())
                            .map(Localize::localize),
                        |row, error| {
                            row.child(
                                div()
                                    .text_size(small)
                                    .text_color(rgb(theme.ink(theme.palette[3])))
                                    .child(error),
                            )
                        },
                    ),
            )
            .child(cards)
    }
}
