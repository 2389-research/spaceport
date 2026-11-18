// ABOUTME: Shares usage panel content between interactive and captured observations.
// ABOUTME: Keeps navigation, links, and scroll ownership in the active panel.
//! The usage panel a reported status bar segment opens, with its account,
//! limits, and links to the service's own pages.

use super::{
    Paint,
    model::Provider,
    paint::{Metadata, ReportView},
    text,
    ui::Ui,
};
use crate::{i18n::fl, menu::Page, window::SheprdWindow};
use gpui::{prelude::*, *};

pub(crate) const PANEL_WIDTH: f32 = 340.;
/// The gap between the status bar and the panel it opened.
pub(crate) const PANEL_GAP: f32 = 6.;

impl SheprdWindow {
    pub(crate) fn open_usage(
        &mut self,
        provider: Provider,
        anchor: Point<Pixels>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.open_menu(window, cx) {
            self.menu.anchor = anchor;
            self.menu.page = Some(Page::Usage(provider));
        }
    }

    /// Shows `provider` in the panel and leads the status bar with it.
    fn choose_usage(&mut self, provider: Provider) {
        self.menu.page = Some(Page::Usage(provider));
        self.usage.choose(provider);
    }

    /// Left and right step through the tabs, as the pointer does, so the
    /// status bar follows the keyboard too.
    pub(crate) fn usage_key(
        &mut self,
        provider: Provider,
        event: &KeyDownEvent,
        cx: &mut Context<Self>,
    ) -> bool {
        let providers: Vec<_> = self
            .usage
            .current()
            .map(|entry| {
                entry
                    .tabs(&self.config.usage)
                    .iter()
                    .map(|r| r.provider)
                    .collect()
            })
            .unwrap_or_default();
        let Some(index) = providers.iter().position(|p| *p == provider) else {
            return false;
        };
        let count = providers.len();
        let next = match event.keystroke.key.as_str() {
            "left" => (index + count - 1) % count,
            "right" => (index + 1) % count,
            _ => return false,
        };
        self.choose_usage(providers[next]);
        cx.notify();
        true
    }

    pub(crate) fn render_usage_panel(&self, provider: Provider, cx: &mut Context<Self>) -> Div {
        let (metadata, report) = self.usage_metadata(provider);
        self.usage_panel_content(&metadata, report.map(ReportView::Current), true, cx)
    }

    pub(crate) fn render_usage_paint(&self, paint: &Paint, cx: &mut Context<Self>) -> Div {
        self.usage_panel_content(
            &paint.metadata,
            paint.report.as_ref().map(ReportView::Frozen),
            false,
            cx,
        )
    }

    fn usage_panel_content(
        &self,
        metadata: &Metadata,
        report: Option<ReportView<'_>>,
        active: bool,
        cx: &mut Context<Self>,
    ) -> Div {
        let theme = &self.theme;
        let font = &self.config.ui;
        let small = px(font.size * 0.9);
        let provider = metadata.provider;
        let rule = || div().h(px(1.)).my(px(6.)).bg(rgb(theme.active));
        let mut view = div().flex().flex_none().flex_col();
        let meta = provider.meta();
        let account = report
            .map(|report| report.standard().account.clone())
            .unwrap_or_default();
        view =
            view.child(
                div()
                    .debug_selector(|| "usage-panel-header".into())
                    .px(px(8.))
                    .py(px(6.))
                    .flex()
                    .justify_between()
                    .gap(px(12.))
                    .child(
                        div()
                            .flex_1()
                            .min_w_0()
                            .child(
                                div()
                                    .text_size(px(font.size * 1.25))
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .child(meta.name),
                            )
                            .child(
                                div()
                                    .truncate()
                                    .text_size(small)
                                    .text_color(rgb(theme.muted))
                                    .child(fl!(
                                        "usage-panel-subtitle",
                                        updated = metadata.updated.as_str(),
                                        host = metadata.host.as_str()
                                    )),
                            ),
                    )
                    .child(
                        div()
                            .flex_none()
                            .max_w(px(PANEL_WIDTH * 0.6))
                            .flex()
                            .flex_col()
                            .items_end()
                            .text_color(rgb(theme.muted))
                            .children(account.email.map(|email| {
                                div().max_w_full().truncate().child(text::words(&email))
                            }))
                            .children(
                                account
                                    .plan
                                    .map(|plan| div().text_size(small).child(text::words(&plan))),
                            ),
                    ),
            );
        for error in &metadata.errors {
            view = view.child(
                div()
                    .px(px(8.))
                    .pb(px(4.))
                    .text_size(small)
                    .text_color(rgb(theme.ink(theme.palette[3])))
                    .child(error.clone()),
            );
        }
        let ui = Ui {
            theme: self.theme.clone(),
            font_size: font.size,
            now: metadata.now,
        };
        if let Some(report) = report {
            view = view.child(rule()).child(report.render(&ui));
        } else {
            view = view
                .child(rule())
                .child(fl!("usage-core-unavailable-detail"));
        }
        let body = view
            .child(rule())
            .children(meta.dashboard.map(|url| {
                self.usage_action(
                    "usage-dashboard",
                    "icons/chart.svg",
                    fl!("usage-dashboard"),
                    active,
                    move |_, _, cx| cx.open_url(url),
                )
            }))
            .children(meta.status_page.map(|url| {
                self.usage_action(
                    "usage-status",
                    "icons/pulse.svg",
                    fl!("usage-status-page"),
                    active,
                    move |_, _, cx| cx.open_url(url),
                )
            }));
        let body = div()
            .id("usage-panel")
            .debug_selector(|| "usage-panel".into())
            .flex()
            .flex_col()
            .min_h_0()
            .px(px(6.))
            .py(px(4.))
            .when(active, |panel| {
                panel
                    .overflow_y_scroll()
                    .track_scroll(&self.menu.usage_scroll)
            })
            .when(!active, |panel| panel.overflow_hidden())
            .child(body.when(!active, |body| body.relative().top(metadata.offset.y)));
        // The tabs sit on the panel's bottom edge, beside the status bar it
        // rises from: switching agents changes the panel's height above them,
        // so they never move under the pointer.
        div()
            .flex()
            .flex_col()
            .min_h_0()
            .child(body.flex_shrink_1())
            .when(metadata.tabs.len() > 1, |panel| {
                panel.child(
                    div()
                        .flex_none()
                        .px(px(6.))
                        .pb(px(6.))
                        .child(div().h(px(1.)).mb(px(6.)).bg(rgb(theme.active)))
                        .child(self.usage_tabs(&metadata.tabs, provider, active, cx)),
                )
            })
    }

    fn usage_tabs(
        &self,
        providers: &[Provider],
        selected: Provider,
        active: bool,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let theme = &self.theme;
        div()
            .flex()
            .flex_wrap()
            .children(providers.iter().copied().map(|provider| {
                let chosen = provider == selected;
                let (background, text) = if chosen {
                    let wash = theme.primary_wash();
                    (Some(wash), theme.text_on(wash))
                } else {
                    (None, theme.muted)
                };
                let key = provider.id();
                // Two tabs share the row; more wrap four to a row, as CodexBar's do.
                let width = if providers.len() <= 2 { 0.5 } else { 0.25 };
                div()
                    .id(SharedString::from(format!("usage-tab-{key}")))
                    .debug_selector(move || format!("usage-tab-{key}"))
                    .w(relative(width))
                    .min_w_0()
                    .flex()
                    .flex_col()
                    .items_center()
                    .gap(px(2.))
                    .py(px(6.))
                    .rounded(px(crate::config::corners::CONTROL))
                    .when(active, |tab| tab.cursor_pointer())
                    .text_color(rgb(text))
                    .when_some(background, |tab, background| tab.bg(rgb(background)))
                    .when(active && !chosen, |tab| {
                        tab.hover(|s| s.bg(rgb(theme.active)))
                    })
                    .child(
                        svg()
                            .path(super::icons::path(provider))
                            .size(px(16.))
                            .text_color(rgb(text)),
                    )
                    .child(div().max_w_full().truncate().child(provider.name()))
                    .when(active, |tab| {
                        tab.on_click(cx.listener(move |this, _, _, cx| {
                            cx.stop_propagation();
                            this.choose_usage(provider);
                            cx.notify();
                        }))
                    })
            }))
    }

    fn usage_action(
        &self,
        id: &'static str,
        icon: &'static str,
        label: String,
        active: bool,
        action: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> impl IntoElement {
        let theme = &self.theme;
        div()
            .id(id)
            .debug_selector(move || id.into())
            .px(px(8.))
            .py(px(6.))
            .flex()
            .items_center()
            .gap(px(8.))
            .rounded(px(crate::config::corners::CONTROL))
            .when(active, |button| {
                button.cursor_pointer().hover(|s| s.bg(rgb(theme.active)))
            })
            .child(
                svg()
                    .path(icon)
                    .size(px(14.))
                    .flex_none()
                    .text_color(rgb(theme.foreground)),
            )
            .child(label)
            .when(active, |button| {
                button.on_click(move |event, window, cx| {
                    cx.stop_propagation();
                    action(event, window, cx);
                })
            })
    }
}
