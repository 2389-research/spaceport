//! Core-owned machine facts in the existing device picker.
//! Rows are read only while the legacy device actions await migration.

use crate::{SheprdWindow, core_terminals, i18n::fl};
use gpui::{prelude::*, *};
use std::fmt::{self, Display};

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
    pub(super) fn render_core_machine_catalog(
        &self,
        cx: &mut Context<Self>,
    ) -> Option<Stateful<Div>> {
        let machines = core_terminals::machine_catalog(cx);
        let error = core_terminals::machine_catalog_error(cx);
        if !core_terminals::machine_catalog_started(cx) {
            return None;
        }
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
        if let Some(machines) = machines {
            if machines.is_empty() {
                section = section.child(
                    div()
                        .px(px(8.))
                        .text_color(rgb(theme.muted))
                        .child(fl!("core-terminals-machine-empty")),
                );
            }
            for (index, machine) in machines.iter().take(MAX_DISPLAY_MACHINES).enumerate() {
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
                        .child(
                            div()
                                .min_w_0()
                                .truncate()
                                .child(short(&machine.machine.name)),
                        )
                        .child(
                            div()
                                .min_w_0()
                                .truncate()
                                .text_size(px(self.config.ui.size * 0.85))
                                .text_color(rgb(theme.muted))
                                .child(format!(
                                    "{} · {}",
                                    short(&machine.machine.target),
                                    short(&machine.state)
                                )),
                        ),
                );
            }
            if machines.len() > MAX_DISPLAY_MACHINES {
                section = section.child(
                    div()
                        .id("core-machines-more")
                        .debug_selector(|| "core-machines-more".into())
                        .px(px(8.))
                        .text_color(rgb(theme.muted))
                        .child(fl!(
                            "core-terminals-machine-more",
                            count = machines.len() - MAX_DISPLAY_MACHINES
                        )),
                );
            }
        } else {
            let reason = unavailable_reason(error);
            section = section.child(
                div()
                    .id("core-machines-unavailable")
                    .debug_selector(|| "core-machines-unavailable".into())
                    .px(px(8.))
                    .text_color(rgb(theme.muted))
                    .child(fl!("core-terminals-machine-unavailable", reason = reason)),
            );
        }
        Some(section)
    }
}
