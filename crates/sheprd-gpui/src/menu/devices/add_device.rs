// ABOUTME: Owns the device setup form and its confirmed background work.
// ABOUTME: Keeps the Core host form closed when the running app has Core disabled.
//! The Add Device dialog: its form, and the Core request that adds the
//! machine off the UI thread (see `core`).

use super::Page;

pub(super) mod core;

use crate::i18n::fl;
use crate::{SheprdWindow, search_input::SearchInput};
use gpui::{prelude::*, *};
use sheprd_client::protocol::{ClientKeyCode, ClientKeyKind, ClientPaneInputEvent};

pub(in crate::menu) struct Setup {
    pub(super) fields: [Entity<SearchInput>; 3],
    pub(super) step: Step,
    /// Why the last add failed. Each source of Add Device keeps its own, so
    /// a job finishing while another source shows stays here.
    pub(super) error: Option<String>,
}

/// Where adding a device stands. Each step after `Form` belongs to the request
/// that was submitted, not to whatever the fields hold now.
pub(super) enum Step {
    Form,
    CoreConnecting(core::Request),
    CoreAdding {
        request: core::Request,
        reply: core::Reply,
    },
    CoreWaiting(core::Request),
    CoreUnknown,
    /// Saved; the next frame closes the dialog (see `poll_device_setup`).
    Saved,
}

impl SheprdWindow {
    /// Set the host form's error, which stays with that form.
    pub(super) fn set_device_error(&mut self, error: Option<String>) {
        if let Some(setup) = &mut self.menu.device_setup {
            setup.error = error;
        }
    }

    /// Show an empty Add Device form with the target field focused.
    pub(in crate::menu) fn open_add_device(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if !self.core_enabled() {
            return;
        }
        core::Gate::install(cx);
        let fields = std::array::from_fn(|index| {
            let input = cx.new(SearchInput::new);
            input.update(cx, |input, cx| {
                input.set_appearance(self.config.ui.clone(), self.theme.clone(), cx);
                match index {
                    0 => input.set_placeholder_with(|| fl!("add-device-target-placeholder"), cx),
                    1 => input.set_placeholder_with(|| fl!("add-device-label-placeholder"), cx),
                    // Herdr's default session's name.
                    _ => input.set_placeholder("default", cx), // i18n: data
                }
            });
            input
        });
        window.focus(&fields[0].read(cx).focus.clone(), cx);
        let (step, error) = match cx.global::<core::Gate>().state() {
            core::State::Idle => (Step::Form, None),
            core::State::InFlight(request) => (Step::CoreWaiting(request), None),
            core::State::Unknown(request) => {
                let error = fl!(
                    "add-device-core-unknown",
                    target = sheprd_client::text::safe_text(&request.target, 160)
                );
                (Step::CoreUnknown, Some(error))
            }
        };
        self.menu.device_setup = Some(Setup {
            fields,
            step,
            error,
        });
        self.menu.page = Some(Page::AddDevice);
    }

    /// Route a key to the Add Device form. Returns `false` when the key is
    /// left to propagate: during composition, or when the form ignores it.
    pub(super) fn add_device_key(
        &mut self,
        event: &KeyDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        let Some(form) = &self.menu.device_setup else {
            return false;
        };
        if matches!(
            form.step,
            Step::CoreConnecting(_)
                | Step::CoreAdding { .. }
                | Step::CoreWaiting(_)
                | Step::CoreUnknown
        ) {
            if event.keystroke.key == "escape" {
                self.dismiss_menu(window, cx);
            }
            return true;
        }
        if form
            .fields
            .iter()
            .any(|field| field.read(cx).is_composing())
        {
            return false;
        }
        let key = event.keystroke.key.as_str();
        let modifiers = event.keystroke.modifiers;
        match key {
            "tab" => {
                let index = form
                    .fields
                    .iter()
                    .position(|field| field.read(cx).focus.is_focused(window))
                    .unwrap_or(0);
                let next = (index + if modifiers.shift { 2 } else { 1 }) % 3;
                window.focus(&form.fields[next].read(cx).focus.clone(), cx);
            }
            "enter" => self.submit_core_device(window, cx),
            "escape" => self.dismiss_menu(window, cx),
            _ => return false,
        }
        true
    }

    pub(in crate::menu) fn render_add_device(
        &self,
        compact: bool,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let theme = &self.theme;
        let frame = |pane: Div, cx: &mut Context<Self>| {
            self.add_device_frame(
                "device-setup-dialog",
                "device-setup-close",
                compact,
                pane,
                cx,
            )
        };
        let Some(setup) = &self.menu.device_setup else {
            return frame(div(), cx);
        };
        let mut body = div()
            .id("device-setup-body")
            .debug_selector(|| "device-setup-body".into())
            .flex_1()
            .min_h_0()
            .overflow_y_scroll()
            .p(px(16.))
            .flex()
            .flex_col()
            .gap(px(12.))
            .child(
                div()
                    .flex_none()
                    .text_color(rgb(theme.subtext()))
                    .child(fl!("add-device-core-intro")),
            );
        for (label, field) in [
            fl!("add-device-target"),
            fl!("add-device-label"),
            fl!("add-device-session"),
        ]
        .into_iter()
        .zip(&setup.fields)
        {
            let row = div().flex_none().flex().flex_col().gap(px(6.)).child(label);
            let core_pending = matches!(
                setup.step,
                Step::CoreConnecting(_)
                    | Step::CoreAdding { .. }
                    | Step::CoreWaiting(_)
                    | Step::CoreUnknown
            );
            body = body.child(if core_pending {
                row.child(sheprd_client::text::safe_text(field.read(cx).text(), 160))
            } else {
                row.child(field.clone())
            });
        }
        if let Some(error) = &setup.error {
            body = body.child(
                div()
                    .flex_none()
                    .text_color(crate::menu::danger(theme))
                    .child(error.clone()),
            );
        }
        let status = match &setup.step {
            Step::Form | Step::CoreUnknown | Step::Saved => None,
            Step::CoreConnecting(request) => Some(fl!(
                "add-device-core-connecting",
                target = sheprd_client::text::safe_text(&request.target, 160)
            )),
            Step::CoreAdding { request, .. } => Some(fl!(
                "add-device-core-adding",
                target = sheprd_client::text::safe_text(&request.target, 160)
            )),
            Step::CoreWaiting(request) => Some(fl!(
                "add-device-core-waiting",
                target = sheprd_client::text::safe_text(&request.target, 160)
            )),
        };
        if let Some(status) = status {
            body = body.child(
                div()
                    .debug_selector(|| "device-setup-status".into())
                    .flex_none()
                    .text_color(rgb(theme.muted))
                    .child(status),
            );
        }
        let button = |id: &'static str, label: SharedString, enabled: bool| {
            div()
                .id(id)
                .debug_selector(move || id.into())
                .p(px(8.))
                .rounded(px(crate::config::corners::CONTROL))
                .bg(rgb(theme.active))
                .when(enabled, |button| {
                    button.cursor_pointer().hover(|s| {
                        s.bg(rgb(theme.active).blend(rgba((theme.foreground << 8) | 0x20)))
                    })
                })
                .when(!enabled, |button| button.text_color(rgb(theme.muted)))
                .child(label)
        };
        let ready = matches!(setup.step, Step::Form);
        let footer = div()
            .debug_selector(|| "device-setup-footer".into())
            .flex_none()
            .p(px(16.))
            .border_t_1()
            .border_color(rgb(theme.active))
            .flex()
            .justify_end()
            .gap(px(8.))
            .child(
                button(
                    "device-setup-submit",
                    if ready {
                        fl!("add-device-submit")
                    } else {
                        fl!("add-device-submit-checking")
                    }
                    .into(),
                    ready,
                )
                .on_click(cx.listener(|this, _, window, cx| this.submit_core_device(window, cx))),
            );
        frame(
            div()
                .flex_1()
                .min_h_0()
                .flex()
                .flex_col()
                .child(body)
                .child(footer),
            cx,
        )
    }

    /// Advance a Core add in progress, and close the dialog once the device
    /// is saved.
    pub(crate) fn poll_device_setup(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.poll_core_device(window, cx);
        let saved = matches!(
            self.menu.device_setup.as_ref().map(|form| &form.step),
            Some(Step::Saved)
        );
        if !saved {
            return;
        }
        // While another source of Add Device shows, only the finished host
        // form goes; the next visit to it starts a fresh one.
        if self.menu.page != Some(Page::AddDevice) {
            self.menu.device_setup = None;
            cx.notify();
        } else {
            self.dismiss_menu(window, cx);
        }
    }
}

/// The Enter key, which submits a line typed into a pane's shell.
pub(crate) fn enter() -> ClientPaneInputEvent {
    ClientPaneInputEvent::Key {
        code: ClientKeyCode::Enter,
        modifiers: 0,
        kind: ClientKeyKind::Press,
        repeat_count: 1,
        shifted_codepoint: None,
        generated_text: None,
        tracks_release: false,
        physical_key_id: None,
        windows_record: None,
    }
}
