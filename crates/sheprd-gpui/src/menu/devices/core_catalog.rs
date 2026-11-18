// ABOUTME: Renders prepared Core machine and remote grouping facts in the device picker.
// ABOUTME: Keeps Core-specific catalog controls hidden until the app has opted in.
//! Core-owned machine facts in the existing device picker.
//! Group facts remain read only; connected machine rows select the active document.

use crate::{SheprdWindow, core_terminals, i18n::fl};
use gpui::{prelude::*, *};
use std::fmt::{self, Display};

mod paint;
pub(super) use paint::CatalogPaint;

const MAX_DISPLAY_MACHINES: usize = 50;
const MAX_DISPLAY_CHARS: usize = 160;

struct LimitedText {
    text: String,
    remaining: usize,
    truncated: bool,
}

impl fmt::Write for LimitedText {
    fn write_str(&mut self, value: &str) -> fmt::Result {
        for character in value.chars() {
            if self.remaining == 0 {
                self.truncated = true;
                break;
            }
            self.text.push(character);
            self.remaining -= 1;
        }
        Ok(())
    }
}

pub(super) fn short(value: impl Display) -> String {
    let mut shown = LimitedText {
        text: String::new(),
        remaining: MAX_DISPLAY_CHARS,
        truncated: false,
    };
    if fmt::write(&mut shown, format_args!("{value}")).is_err() {
        return fl!("core-terminals-machine-not-current").to_string();
    }
    if shown.truncated {
        shown.text.push('…');
    }
    shown.text
}

pub(super) fn unavailable_reason(error: Option<&impl Display>) -> String {
    error
        .map(short)
        .unwrap_or_else(|| fl!("core-terminals-machine-not-current").to_string())
}

impl SheprdWindow {
    pub(super) fn core_machine_catalog_paint(&self, cx: &App) -> Option<CatalogPaint> {
        self.prepare_core_catalog(cx).map(|(paint, _)| paint)
    }

    pub(super) fn render_core_machine_catalog(
        &self,
        cx: &mut Context<Self>,
    ) -> Option<Stateful<Div>> {
        let (paint, actions) = self.prepare_core_catalog(cx)?;
        Some(self.core_catalog_content(&paint, Some(actions), cx))
    }

    pub(super) fn render_core_machine_catalog_paint(
        &self,
        paint: &CatalogPaint,
        cx: &mut Context<Self>,
    ) -> Stateful<Div> {
        self.core_catalog_content(paint, None, cx)
    }

    fn core_catalog_content(
        &self,
        paint: &CatalogPaint,
        actions: Option<Vec<String>>,
        cx: &mut Context<Self>,
    ) -> Stateful<Div> {
        let theme = &self.theme;
        let mut section = div()
            .id("core-machines")
            .flex()
            .flex_col()
            .gap(px(4.))
            .border_t_1()
            .border_color(rgb(theme.active))
            .pt(px(8.))
            .child(
                div()
                    .px(px(8.))
                    .text_color(rgb(theme.muted))
                    .child(fl!("core-terminals-machine-heading")),
            );
        let mut actions = actions.unwrap_or_default().into_iter();
        match &paint.body {
            paint::Body::Machines { rows, more } => {
                if rows.is_empty() {
                    section = section.child(
                        div()
                            .px(px(8.))
                            .text_color(rgb(theme.muted))
                            .child(fl!("core-terminals-machine-empty")),
                    );
                }
                for (index, machine) in rows.iter().enumerate() {
                    let action = actions.next().filter(|_| machine.connected);
                    let id = SharedString::from(format!("core-machine-{index}"));
                    section = section.child(
                        div()
                            .id(id.clone())
                            .debug_selector(move || id.to_string())
                            .px(px(8.))
                            .py(px(4.))
                            .flex()
                            .flex_col()
                            .min_w_0()
                            .when(machine.expanded, |row| row.bg(rgb(theme.active)))
                            .when_some(action, |row, machine_id| {
                                row.role(Role::Button)
                                    .aria_label(machine.label.clone())
                                    .cursor_pointer()
                                    .on_click(cx.listener(move |_, _, _, cx| {
                                        cx.stop_propagation();
                                        core_terminals::select_remote_machine(cx, &machine_id);
                                        cx.notify();
                                    }))
                            })
                            .child(div().min_w_0().truncate().child(machine.label.clone()))
                            .child(
                                div()
                                    .min_w_0()
                                    .truncate()
                                    .text_size(px(self.config.ui.size * 0.85))
                                    .text_color(rgb(theme.muted))
                                    .child(machine.detail.clone()),
                            ),
                    );
                    if let Some(groups) = &machine.groups {
                        section = section.child(self.core_catalog_groups(groups));
                    }
                }
                if let Some(more) = more {
                    section = section.child(
                        div()
                            .id("core-machines-more")
                            .debug_selector(|| "core-machines-more".into())
                            .px(px(8.))
                            .text_color(rgb(theme.muted))
                            .child(more.clone()),
                    );
                }
            }
            paint::Body::Unavailable { reason, stale } => {
                section = section.child(
                    div()
                        .id("core-machines-unavailable")
                        .debug_selector(|| "core-machines-unavailable".into())
                        .px(px(8.))
                        .text_color(rgb(theme.muted))
                        .child(reason.clone()),
                );
                if *stale {
                    section = section.child(
                        div()
                            .id("core-remote-groups-stale")
                            .debug_selector(|| "core-remote-groups-stale".into())
                            .px(px(8.))
                            .text_color(rgb(theme.muted))
                            .child(fl!("core-terminals-machine-groups-stale")),
                    );
                }
            }
        }
        section
    }

    fn core_catalog_groups(&self, paint: &paint::Groups) -> Stateful<Div> {
        let mut groups = div()
            .id("core-remote-groups")
            .debug_selector(|| "core-remote-groups".into())
            .flex()
            .flex_col()
            .gap(px(3.))
            .pl(px(16.))
            .pr(px(8.))
            .text_color(rgb(self.theme.muted))
            .child(fl!("core-terminals-machine-groups"));
        match paint {
            paint::Groups::Ready { labels, more } => {
                if labels.is_empty() {
                    groups = groups.child(fl!("core-terminals-machine-groups-empty"));
                }
                for (index, label) in labels.iter().enumerate() {
                    let selector = format!("core-remote-group-{index}");
                    groups = groups.child(
                        div()
                            .id(SharedString::from(selector.clone()))
                            .debug_selector(move || selector.clone())
                            .min_w_0()
                            .truncate()
                            .child(label.clone()),
                    );
                }
                if let Some(more) = more {
                    groups = groups.child(
                        div()
                            .id("core-remote-groups-more")
                            .debug_selector(|| "core-remote-groups-more".into())
                            .child(more.clone()),
                    );
                }
            }
            paint::Groups::Message { selector, text } => {
                let selector = *selector;
                groups = groups.child(
                    div()
                        .id(selector)
                        .debug_selector(move || selector.into())
                        .child(text.clone()),
                );
            }
        }
        groups
    }
}
