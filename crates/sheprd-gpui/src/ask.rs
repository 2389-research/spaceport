// ABOUTME: Owns the app-wide Ask Spaceport conversation independently of windows.
// ABOUTME: Creates helpers only for submitted questions or confirmed interrupted recovery.
mod error;
mod feature;
mod model;
mod view;
mod worker;

use crate::chat::{link::Link, session::Session};
pub(crate) use feature::Feature;
use gpui::*;
use sheprd_chat_protocol::{
    ClientMessage, Command, CommandId, Cursor, ServerMessage, Status, Transcript, VERSION,
    ask::{Checkpoint, Outcome},
};
use std::{path::PathBuf, time::Duration};
pub(crate) use view::View;
use worker::{Event, Worker};

struct Shared(Entity<Controller>);
impl Global for Shared {}

pub(crate) struct Controller {
    session: Session,
    worker: Option<Worker>,
    link: Option<Link>,
    pending: Option<String>,
    cancel_when_live: bool,
    active: bool,
    working_seen: bool,
    configured: bool,
    loading: bool,
    problem: Option<String>,
    failed_draft: Option<String>,
    awaiting_ack: Option<(CommandId, String)>,
    _poll: Task<()>,
}

impl Controller {
    fn new(worker: Option<Worker>, configured: bool, cx: &mut Context<Self>) -> Self {
        let loading = worker.is_some();
        let poll = if worker.is_some() {
            cx.spawn(async move |this, cx| {
                loop {
                    cx.background_executor()
                        .timer(Duration::from_millis(33))
                        .await;
                    if this.update(cx, |this, cx| this.poll(cx)).is_err() {
                        break;
                    }
                }
            })
        } else {
            Task::ready(())
        };
        Self {
            session: Session::default(),
            worker,
            link: None,
            pending: None,
            cancel_when_live: false,
            active: false,
            working_seen: false,
            configured,
            loading,
            problem: None,
            failed_draft: None,
            awaiting_ack: None,
            _poll: poll,
        }
    }

    fn restore(&mut self, checkpoint: Option<Box<Checkpoint>>, cx: &mut Context<Self>) {
        self.loading = false;
        self.session = Session::default();
        self.active = false;
        self.cancel_when_live = false;
        self.working_seen = false;
        self.awaiting_ack = None;
        self.problem = None;
        let Some(saved) = checkpoint else {
            cx.notify();
            return;
        };
        self.configured = true;
        let outcome = saved.outcome;
        let endpoint = saved
            .endpoint
            .parse()
            .ok()
            .and_then(|endpoint| sheprd_chat_protocol::endpoint_address(&endpoint));
        self.set_transcript(saved.transcript);
        match outcome {
            Outcome::Interrupted => {
                self.active = true;
                self.launch();
            }
            Outcome::Answering => {
                self.active = true;
                if let Some(endpoint) = endpoint {
                    self.attach(endpoint);
                } else {
                    self.problem = Some(crate::i18n::fl!("ask-unknown"));
                    self.active = false;
                }
            }
            Outcome::Unknown => self.problem = Some(crate::i18n::fl!("ask-unknown")),
            _ => {}
        }
        cx.notify();
    }

    fn set_transcript(&mut self, transcript: Transcript) {
        self.session.begin();
        let _ = self.session.receive(ServerMessage::Welcome {
            agent: "Pi".into(),
            pane: None,
            version: VERSION,
        });
        let _ = self.session.receive(ServerMessage::Snapshot {
            cursor: Cursor::new_epoch(),
            transcript: Box::new(transcript),
        });
    }

    fn launch(&mut self) {
        self.loading = true;
        let result = self
            .worker
            .as_ref()
            .ok_or(error::Error::NoBroker)
            .and_then(|worker| {
                worker
                    .commands
                    .try_send(worker::Command::Launch)
                    .map_err(|_| error::Error::Busy)
            });
        if let Err(error) = result {
            self.fail(error);
        }
    }

    fn attach(&mut self, endpoint: PathBuf) {
        self.session.begin();
        match Link::start(endpoint, None, []) {
            Ok(link) => {
                self.link = Some(link);
                self.loading = false;
            }
            Err(error) => self.fail(error.into()),
        }
    }

    fn fail(&mut self, error: error::Error) {
        self.loading = false;
        self.active = false;
        self.remember_draft();
        self.problem = Some(format!("{error:#}"));
    }

    fn remember_draft(&mut self) {
        if let Some(text) = self
            .pending
            .take()
            .or_else(|| self.awaiting_ack.take().map(|(_, text)| text))
        {
            self.failed_draft = Some(text);
        }
        self.awaiting_ack = None;
    }

    fn submit(&mut self, text: String, cx: &mut Context<Self>) -> bool {
        if self.active
            || self.loading
            || !self.configured
            || text.trim().is_empty()
            || text.len() > sheprd_chat_protocol::MAX_PROMPT_SIZE
        {
            return false;
        }
        self.pending = Some(text);
        self.failed_draft = None;
        self.active = true;
        self.working_seen = false;
        self.problem = None;
        if self.link.is_none() {
            self.launch();
        }
        cx.notify();
        self.active
    }

    fn stop(&mut self, cx: &mut Context<Self>) {
        self.pending = None;
        self.cancel_when_live = self.active;
        if self.active
            && let Some(worker) = &self.worker
            && worker.commands.try_send(worker::Command::Stop).is_err()
        {
            self.problem = Some(error::Error::Busy.to_string());
            cx.notify();
            return;
        }
        if let Some(link) = &self.link {
            if let Err(error) = link.send(ClientMessage::Command {
                id: CommandId::random(),
                command: Command::Cancel,
            }) {
                self.fail(error.into());
            } else {
                self.cancel_when_live = false;
                self.working_seen = true;
            }
        } else if self.active {
            self.problem = Some(crate::i18n::fl!("ask-stop-pending"));
        }
        cx.notify();
    }

    fn new_chat(&mut self, configured: bool, cx: &mut Context<Self>) {
        if self.loading {
            return;
        }
        self.stop(cx);
        self.cancel_when_live = false;
        self.failed_draft = None;
        self.configured = configured;
        self.awaiting_ack = None;
        self.link = None;
        self.loading = true;
        if let Some(worker) = &self.worker {
            if worker.commands.try_send(worker::Command::New).is_err() {
                self.fail(error::Error::Busy);
            }
        } else {
            self.session = Session::default();
            self.active = false;
            self.loading = false;
        }
        cx.notify();
    }

    fn poll(&mut self, cx: &mut Context<Self>) {
        let events = self
            .worker
            .as_ref()
            .map(|worker| worker.events.try_iter().collect::<Vec<_>>())
            .unwrap_or_default();
        let mut changed = !events.is_empty();
        for event in events {
            match event {
                Event::Restored(saved) => self.restore(saved, cx),
                Event::Ready(endpoint) => self.attach(endpoint),
                Event::StopSaved => {}
                Event::Failed(error) => self.fail(error),
            }
        }
        if let Some(link) = &self.link {
            let polled = link.poll(128);
            changed |= !polled.messages.is_empty() || polled.end.is_some();
            for message in polled.messages {
                let rejection = match &message {
                    ServerMessage::Ack {
                        rejected: Some(rejected),
                        ..
                    } => Some(crate::i18n::Localize::localize(rejected)),
                    _ => None,
                };
                if let ServerMessage::Ack { id, rejected } = &message
                    && self
                        .awaiting_ack
                        .as_ref()
                        .is_some_and(|(waiting, _)| waiting == id)
                {
                    let receipt = self.awaiting_ack.take();
                    if rejected.is_some() {
                        self.failed_draft = receipt.map(|(_, text)| text);
                    }
                }
                if matches!(
                    message,
                    ServerMessage::Ack {
                        rejected: Some(_),
                        ..
                    }
                ) {
                    self.active = false;
                }
                if let Err(error) = self.session.receive(message) {
                    self.fail(error.into());
                    break;
                }
                if rejection.is_some() {
                    self.problem = rejection;
                }
                self.working_seen |= matches!(self.session.transcript().status, Status::Working);
                if matches!(
                    self.session.transcript().status,
                    Status::Failed { .. } | Status::Exited { .. }
                ) {
                    self.active = false;
                    self.problem = match &self.session.transcript().status {
                        Status::Failed { message } | Status::Exited { message } => {
                            Some(message.clone())
                        }
                        _ => self.problem.take(),
                    };
                    self.remember_draft();
                }
                if self.working_seen
                    && !matches!(
                        self.session.transcript().status,
                        Status::Working | Status::Starting
                    )
                {
                    self.active = false;
                }
            }
            if let Some(error) = polled.end {
                self.link = None;
                self.session.lost(&error);
                self.problem = Some(if self.active {
                    crate::i18n::fl!("ask-unknown")
                } else {
                    error.to_string()
                });
                self.active = false;
                self.remember_draft();
            }
        }
        if self.session.live() && self.cancel_when_live {
            self.cancel_when_live = false;
            self.working_seen = true;
            if let Some(link) = &self.link
                && let Err(error) = link.send(ClientMessage::Command {
                    id: CommandId::random(),
                    command: Command::Cancel,
                })
            {
                self.fail(error.into());
            }
            changed = true;
        }
        if self.session.live()
            && matches!(self.session.transcript().status, Status::Idle)
            && let Some(text) = self.pending.as_ref()
        {
            let id = CommandId::random();
            let text = text.clone();
            let message = ClientMessage::Command {
                id,
                command: Command::Prompt {
                    text: text.clone(),
                    attachments: Vec::new(),
                },
            };
            let result = self
                .link
                .as_ref()
                .ok_or(crate::chat::Error::NotAttached)
                .and_then(|link| link.send(message));
            if result.is_ok() {
                self.awaiting_ack = Some((id, text));
            }
            self.dispatched(result);
            changed = true;
        }
        if changed {
            cx.notify();
        }
    }

    fn dispatched(&mut self, result: crate::chat::Result<()>) {
        match result {
            Ok(()) => self.pending = None,
            Err(error) => self.fail(error.into()),
        }
    }
}

pub(crate) fn install(enabled: bool, configured: bool, cx: &mut App) {
    Feature::install(enabled, cx);
    if !enabled {
        return;
    }
    if cx.has_global::<Shared>() {
        return;
    }
    let worker = crate::preferences::state_dir()
        .zip(crate::control::observation_endpoint(cx))
        .and_then(|(root, broker)| {
            Worker::start(root.join("ask"), broker)
                .map_err(|error| tracing::warn!(%error, "Ask worker failed"))
                .ok()
        });
    let controller = cx.new(|cx| Controller::new(worker, configured, cx));
    cx.set_global(Shared(controller));
    cx.on_app_quit(|cx| {
        let owner = cx.global::<Shared>().0.clone();
        let worker = owner.update(cx, |owner, _| {
            owner.link = None;
            owner.worker.take()
        });
        cx.background_executor().spawn(async move {
            if let Some(worker) = worker {
                worker.finish();
            }
        })
    })
    .detach();
}

fn controller(cx: &mut App) -> Entity<Controller> {
    if !Feature::current(cx).enabled() {
        return cx.new(|cx| Controller::new(None, false, cx));
    }
    if !cx.has_global::<Shared>() {
        let owner = cx.new(|cx| Controller::new(None, false, cx));
        cx.set_global(Shared(owner));
    }
    cx.global::<Shared>().0.clone()
}

pub(crate) fn status(cx: &App) -> Option<sheprd_control_protocol::protocol::app::AskStatus> {
    use sheprd_control_protocol::protocol::app::{AskStatus, MAX_ASK_STATUS_ERROR_CHARS};
    if !Feature::current(cx).enabled() {
        return None;
    }
    let owner = cx.try_global::<Shared>()?.0.read(cx);
    Some(AskStatus {
        agent: owner.configured.then(|| "Pi".to_owned()),
        answering: owner.active,
        configured: owner.configured,
        error: owner
            .problem
            .as_ref()
            .map(|error| crate::notifications::safe_text(error, MAX_ASK_STATUS_ERROR_CHARS)),
        history_present: !owner.session.transcript().entries.is_empty(),
        loading: owner.loading,
    })
}

#[cfg(all(feature = "integration-test", target_os = "macos"))]
pub(crate) fn fixture_transcript(cx: &App) -> Option<Transcript> {
    Some(
        cx.try_global::<Shared>()?
            .0
            .read(cx)
            .session
            .transcript()
            .clone(),
    )
}

#[cfg(feature = "integration-test")]
pub(crate) fn fixture(transcript: Option<Transcript>, cx: &mut App) {
    Feature::install(true, cx);
    controller(cx).update(cx, |owner, cx| {
        owner._poll = Task::ready(());
        owner.worker = None;
        owner.link = None;
        owner.loading = false;
        owner.configured = transcript.is_some();
        owner.active = transcript
            .as_ref()
            .is_some_and(|transcript| matches!(transcript.status, Status::Working));
        owner.set_transcript(transcript.unwrap_or_default());
        cx.notify();
    });
}

#[cfg(test)]
mod tests;
