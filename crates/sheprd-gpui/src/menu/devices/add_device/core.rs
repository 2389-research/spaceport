//! Core-owned machine addition, with one app-wide pending outcome.
//! An unconfirmed request fences every window until this app exits.

use super::Step;
use crate::{SheprdWindow, core_terminals, i18n::fl};
use gpui::{App, Context, Global, Window};
use sheprd_core_client::{CoreBackend, Error as CoreError, wire::Machine};
use std::{
    io,
    sync::{Arc, Mutex, PoisonError, mpsc},
};

#[cfg(test)]
mod tests;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate::menu::devices) struct Request {
    pub target: String,
    name: Option<String>,
    session: Option<String>,
}

impl Request {
    pub(super) fn new(target: &str, name: &str, session: &str) -> Option<Self> {
        let target = target.trim();
        if target.is_empty() {
            return None;
        }
        let optional = |text: &str| (!text.trim().is_empty()).then(|| text.trim().to_owned());
        Some(Self {
            target: target.to_owned(),
            name: optional(name),
            session: optional(session),
        })
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) enum State {
    #[default]
    Idle,
    InFlight(Request),
    Unknown(Request),
}

#[derive(Clone, Default)]
pub(super) struct Gate(Arc<Mutex<State>>);

impl Global for Gate {}

impl Gate {
    pub(super) fn install(cx: &mut App) {
        if cx.try_global::<Self>().is_none() {
            cx.set_global(Self::default());
        }
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, State> {
        self.0.lock().unwrap_or_else(PoisonError::into_inner)
    }

    pub(super) fn state(&self) -> State {
        self.lock().clone()
    }

    pub(super) fn begin(&self, request: Request) -> Result<(), State> {
        let mut state = self.lock();
        match &*state {
            State::Idle => {
                *state = State::InFlight(request);
                Ok(())
            }
            other => Err(other.clone()),
        }
    }

    pub(super) fn finish(&self, result: &Result<Machine, CoreError>) {
        let mut state = self.lock();
        let State::InFlight(request) = &*state else {
            return;
        };
        *state = match result {
            Ok(machine) if !machine.id.is_empty() => State::Idle,
            // Core refuses before inserting the machine; Unsupported and Busy
            // are client-side checks before a request reaches Core.
            Err(CoreError::Refused { .. } | CoreError::Unsupported(_) | CoreError::Busy) => {
                State::Idle
            }
            _ => State::Unknown(request.clone()),
        };
    }

    pub(super) fn failed_to_start(&self) {
        *self.lock() = State::Idle;
    }

    pub(super) fn worker_lost(&self) {
        let mut state = self.lock();
        if let State::InFlight(request) = &*state {
            *state = State::Unknown(request.clone());
        }
    }
}

pub(super) type Reply = mpsc::Receiver<Result<Machine, CoreError>>;

fn run(backend: CoreBackend, request: Request, gate: Gate) -> io::Result<Reply> {
    let (send, receive) = mpsc::sync_channel(1);
    std::thread::Builder::new()
        .name("core-machine-add".into())
        .spawn(move || {
            let result = backend.core().machine_add(
                &request.target,
                request.name.as_deref(),
                request.session.as_deref(),
                false,
            );
            gate.finish(&result);
            let _ = send.send(result);
        })?;
    Ok(receive)
}

fn reason(error: &CoreError) -> String {
    let message = sheprd_client::text::safe_text(&error.to_string(), 160);
    match error {
        CoreError::Refused {
            hint: Some(hint), ..
        } => {
            let hint = sheprd_client::text::safe_text(hint.as_str(), 512);
            if hint.is_empty() {
                message
            } else {
                format!("{message} — {hint}")
            }
        }
        _ => message,
    }
}

fn unknown(request: &Request) -> String {
    fl!(
        "add-device-core-unknown",
        target = sheprd_client::text::safe_text(&request.target, 160)
    )
}

impl SheprdWindow {
    pub(super) fn submit_core_device(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(form) = &self.menu.device_setup else {
            return;
        };
        if !matches!(form.step, Step::Form) {
            return;
        }
        let request = Request::new(
            form.fields[0].read(cx).text(),
            form.fields[1].read(cx).text(),
            form.fields[2].read(cx).text(),
        );
        let Some(request) = request else {
            self.set_device_error(Some(fl!("add-device-core-target-required")));
            cx.notify();
            return;
        };
        window.focus(&self.menu.focus, cx);
        if self.core_terminals.hub.backend().is_none() {
            self.core_terminals.hub.connect(core_terminals::Start::Join);
            if let Some(form) = &mut self.menu.device_setup {
                form.step = Step::CoreConnecting(request);
                form.error = None;
            }
            cx.notify();
            return;
        }
        self.start_core_device(request, window, cx);
    }

    fn start_core_device(&mut self, request: Request, window: &mut Window, cx: &mut Context<Self>) {
        window.focus(&self.menu.focus, cx);
        let gate = cx.global::<Gate>().clone();
        match gate.begin(request.clone()) {
            Ok(()) => {}
            Err(State::InFlight(other)) => {
                if let Some(form) = &mut self.menu.device_setup {
                    form.step = Step::CoreWaiting(other);
                    form.error = None;
                }
                cx.notify();
                return;
            }
            Err(State::Unknown(other)) => {
                if let Some(form) = &mut self.menu.device_setup {
                    form.error = Some(unknown(&other));
                    form.step = Step::CoreUnknown;
                }
                cx.notify();
                return;
            }
            Err(State::Idle) => return,
        }
        let Some(backend) = self.core_terminals.hub.backend().cloned() else {
            gate.failed_to_start();
            return;
        };
        match run(backend, request.clone(), gate.clone()) {
            Ok(reply) => {
                if let Some(form) = &mut self.menu.device_setup {
                    form.step = Step::CoreAdding { request, reply };
                    form.error = None;
                }
            }
            Err(error) => {
                gate.failed_to_start();
                if let Some(form) = &mut self.menu.device_setup {
                    form.error = Some(fl!(
                        "add-device-core-worker-failed",
                        reason = sheprd_client::text::safe_text(&error.to_string(), 160)
                    ));
                }
            }
        }
        cx.notify();
    }

    pub(super) fn poll_core_device(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(form) = &self.menu.device_setup else {
            return;
        };
        if let Step::CoreConnecting(request) = &form.step {
            let request = request.clone();
            match self.core_terminals.hub.state() {
                core_terminals::State::Ready(_) => self.start_core_device(request, window, cx),
                core_terminals::State::Unavailable(error) => {
                    if let Some(form) = &mut self.menu.device_setup {
                        form.step = Step::Form;
                        form.error = Some(fl!(
                            "add-device-core-unavailable",
                            reason = sheprd_client::text::safe_text(&error.to_string(), 160)
                        ));
                    }
                    cx.notify();
                }
                core_terminals::State::Idle => {
                    if let Some(form) = &mut self.menu.device_setup {
                        form.step = Step::Form;
                        form.error = Some(fl!("add-device-core-disconnected"));
                    }
                    cx.notify();
                }
                core_terminals::State::Connecting(_) => {}
            }
            return;
        }
        if matches!(form.step, Step::CoreWaiting(_)) {
            match cx.global::<Gate>().state() {
                State::Idle => {
                    if let Some(form) = &mut self.menu.device_setup {
                        form.step = Step::Form;
                        form.error = None;
                    }
                    cx.notify();
                }
                State::Unknown(request) => {
                    if let Some(form) = &mut self.menu.device_setup {
                        form.error = Some(unknown(&request));
                        form.step = Step::CoreUnknown;
                    }
                    cx.notify();
                }
                State::InFlight(_) => {}
            }
            return;
        }
        let Step::CoreAdding { request, reply } = &form.step else {
            return;
        };
        let request = request.clone();
        let outcome = reply.try_recv();
        let Some(form) = &mut self.menu.device_setup else {
            return;
        };
        match outcome {
            Ok(Ok(machine)) if !machine.id.is_empty() => {
                form.step = Step::Saved;
                form.error = None;
            }
            Ok(Err(
                error @ (CoreError::Refused { .. } | CoreError::Unsupported(_) | CoreError::Busy),
            )) => {
                form.step = Step::Form;
                form.error = Some(reason(&error));
            }
            Ok(Err(_)) | Ok(Ok(_)) | Err(mpsc::TryRecvError::Disconnected) => {
                cx.global::<Gate>().worker_lost();
                form.error = Some(unknown(&request));
                form.step = Step::CoreUnknown;
            }
            Err(mpsc::TryRecvError::Empty) => return,
        }
        cx.notify();
    }
}
