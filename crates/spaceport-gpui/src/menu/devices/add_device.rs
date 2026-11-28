// ABOUTME: Owns the device setup form and its confirmed background work.
// ABOUTME: Keeps the Core host form closed when the running app has Core disabled.
//! The Add Device dialog: its form's state, the words it says, and the Core
//! request that adds the machine off the UI thread (see `core`). The paint is
//! `spaceport_menu::devices`'s.

use super::Page;

pub(super) mod core;

use crate::i18n::fl;
use crate::{SpaceportWindow, search_input::SearchInput};
use gpui::{prelude::*, *};
use spaceport_client::protocol::{ClientKeyCode, ClientKeyKind, ClientPaneInputEvent};
use spaceport_menu::devices::add_device::{self, Action, Form};
use spaceport_menu::devices::{Field, Submit, Value};

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

impl Step {
    /// Whether a request was submitted: what the fields hold now is not what
    /// it carries.
    fn pending(&self) -> bool {
        !matches!(self, Self::Form | Self::Saved)
    }
}

impl SpaceportWindow {
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
                    target = spaceport_client::text::safe_text(&request.target, 160)
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
        if form.step.pending() {
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

    /// The form, as `spaceport_menu` paints it.
    fn device_form(&self, setup: &Setup, cx: &App) -> Form {
        let pending = setup.step.pending();
        let ready = matches!(setup.step, Step::Form);
        let status = match &setup.step {
            Step::CoreConnecting(request) => Some(fl!(
                "add-device-core-connecting",
                target = spaceport_client::text::safe_text(&request.target, 160)
            )),
            Step::CoreAdding { request, .. } => Some(fl!(
                "add-device-core-adding",
                target = spaceport_client::text::safe_text(&request.target, 160)
            )),
            Step::CoreWaiting(request) => Some(fl!(
                "add-device-core-waiting",
                target = spaceport_client::text::safe_text(&request.target, 160)
            )),
            _ => None,
        };
        Form {
            intro: fl!("add-device-core-intro"),
            fields: [
                fl!("add-device-target"),
                fl!("add-device-label"),
                fl!("add-device-session"),
            ]
            .into_iter()
            .zip(&setup.fields)
            .map(|(label, field)| Field {
                label,
                // A request was sent: it carries what was typed, not what the
                // fields hold now.
                value: match pending {
                    true => Value::Typed(spaceport_client::text::safe_text(
                        field.read(cx).text(),
                        160,
                    )),
                    false => Value::Input(field.clone().into_any_element()),
                },
            })
            .collect(),
            error: setup.error.clone(),
            status,
            submit: Submit {
                selector: "device-setup-submit".to_owned(),
                label: match ready {
                    true => fl!("add-device-submit"),
                    false => fl!("add-device-submit-checking"),
                },
                enabled: ready,
            },
            danger: crate::menu::danger(&self.theme),
        }
    }

    pub(in crate::menu) fn render_add_device(&self, compact: bool, cx: &mut Context<Self>) -> Div {
        let pane = match self
            .menu
            .device_setup
            .as_ref()
            .map(|setup| self.device_form(setup, cx))
        {
            Some(form) => add_device::body(&self.theme, form, |action, submit| match action {
                Action::Submit => submit.on_click(
                    cx.listener(|this, _, window, cx| this.submit_core_device(window, cx)),
                ),
            }),
            None => div(),
        };
        self.add_device_frame(
            "device-setup-dialog",
            "device-setup-close",
            compact,
            pane,
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
