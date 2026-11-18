// ABOUTME: Keeps operation failures visible when window chrome is hidden.
// ABOUTME: Renders feedback from the existing notice owners without copying their state.
use super::{SheprdWindow, render::Notice};
use crate::{i18n::fl, window::chrome_focus::Region};
use gpui::{prelude::*, *};

impl SheprdWindow {
    pub(super) fn render_flash(&self, cx: &mut Context<Self>) -> Option<Div> {
        use crate::config::ClipboardToastPosition::*;
        if self.menu.page.is_some() {
            return None;
        }
        let (flash, shown) = self.flash_shown()?;
        let active = self.flash.is_some();
        let position = self.config.clipboard_toast.position;
        let edge = px(12. - (1. - shown) * super::flash::SLIDE);
        let strip = if self.activity_section().is_none()
            && self
                .active_group()
                .is_some_and(|group| !self.strip_hidden(group, cx))
        {
            self.tab_strip_height()
        } else {
            0.
        };
        let (top, tab_bottom) = match self.tab_bar_position() {
            crate::herdr_settings::TabBarPosition::Top => (strip, 0.),
            crate::herdr_settings::TabBarPosition::Bottom => (0., strip),
        };
        let bottom = if self.status_bar_visible && !self.zen_layout() {
            self.status_bar_height()
        } else {
            0.
        };
        Some(
            div()
                .absolute()
                .opacity(shown)
                .map(|row| match position {
                    TopLeft | TopCenter | TopRight => row.top(edge + px(top)),
                    BottomLeft | BottomCenter | BottomRight => {
                        row.bottom(edge + px(bottom + tab_bottom))
                    }
                })
                .map(|row| match position {
                    TopLeft | BottomLeft => row.justify_start(),
                    TopCenter | BottomCenter => row.justify_center(),
                    TopRight | BottomRight => row.justify_end(),
                })
                .left_0()
                .right_0()
                .px(px(12.))
                .flex()
                .overflow_hidden()
                .child(
                    div()
                        .id("gesture-feedback")
                        .debug_selector(|| "flash".into())
                        .when(active, |card| {
                            card.role(match flash.tone {
                                super::flash::Tone::Warning => Role::Alert,
                                _ => Role::Status,
                            })
                            .aria_label(flash.text.clone())
                            .a11y_synthetic_children(|builder| {
                                builder.parent_node().set_live(accesskit::Live::Polite);
                            })
                            .on_hover(cx.listener(
                                |this, hovered, _, cx| {
                                    this.pause_flash(*hovered, std::time::Instant::now());
                                    cx.notify();
                                },
                            ))
                        })
                        .min_w_0()
                        .flex()
                        .items_center()
                        .gap(px(8.))
                        .px(px(12.))
                        .py(px(6.))
                        .rounded(px(crate::config::corners::CONTROL))
                        .border_1()
                        .border_color(rgb(flash.accent(&self.theme)))
                        .bg(rgb(self.theme.surface))
                        .text_color(rgb(self.theme.foreground))
                        .child(
                            div()
                                .size(px(6.))
                                .flex_none()
                                .rounded_full()
                                .bg(rgb(flash.accent(&self.theme))),
                        )
                        .child(div().truncate().child(flash.text.clone())),
                ),
        )
    }

    pub(super) fn resume_hidden_feedback(&mut self) {
        let now = std::time::Instant::now();
        if self.menu.page.is_some() {
            self.pause_flash(false, now);
        }
        if !self.operation_warning_shown() {
            self.notices.pause(false, now);
            self.live.notices.pause(false, now);
        } else if self.notices.is_some() {
            self.live.notices.pause(false, now);
        }
    }

    pub(super) fn operation_warning(&self) -> Option<&str> {
        self.notices.text().or_else(|| self.live.notices.text())
    }

    pub(super) fn operation_warning_shown(&self) -> bool {
        self.menu.page.is_none()
            && (!self.status_bar_visible
                || self.zen_active()
                || (!self.live.status.is_connected() && self.notices.is_some()))
            && self.operation_warning().is_some()
    }

    pub(super) fn operation_warning_card(&self, cx: &mut Context<Self>) -> Option<Notice> {
        if !self.operation_warning_shown() {
            return None;
        }
        let text = SharedString::from(self.operation_warning()?.to_owned());
        Some(Notice {
            shrinks: true,
            card: div()
                .id("operation-warning")
                .debug_selector(|| "operation-warning".into())
                .role(Role::Alert)
                .aria_label(text.clone())
                .on_hover(cx.listener(|this, hovered, _, cx| {
                    let now = std::time::Instant::now();
                    let local = this.notices.is_some();
                    this.notices.pause(*hovered && local, now);
                    this.live.notices.pause(*hovered && !local, now);
                    cx.notify();
                }))
                .a11y_synthetic_children(|builder| {
                    builder.parent_node().set_live(accesskit::Live::Assertive);
                })
                .w_full()
                .max_w(px(420.))
                .min_h_0()
                .overflow_y_scroll()
                .p(px(12.))
                .rounded(px(crate::config::corners::PANEL))
                .border_1()
                .border_color(rgb(self.theme.ink(self.theme.palette[3])))
                .bg(rgb(self.theme.surface))
                .text_color(rgb(self.theme.foreground))
                .child(text)
                .child(
                    self.chrome_stop(
                        Region::Notices,
                        div()
                            .id("operation-warning-history")
                            .role(Role::Button)
                            .aria_label(fl!("commands-messages"))
                            .cursor_pointer()
                            .mt(px(8.))
                            .px(px(8.))
                            .py(px(4.))
                            .rounded(px(crate::config::corners::CONTROL))
                            .border_1()
                            .border_color(rgb(self.theme.active))
                            .child(fl!("commands-messages"))
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.open_messages(window, cx);
                            })),
                    ),
                )
                .into_any_element(),
        })
    }
}
