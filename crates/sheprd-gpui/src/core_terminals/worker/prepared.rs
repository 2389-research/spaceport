// ABOUTME: Reserves Core command capacity while clipboard input is prepared.
// ABOUTME: Cancels one clipboard slot while preserving every other accepted command.
use super::{Command, Link, MAX_COMMANDS, Mailbox};
use sheprd_backend::ClientPaneInputEvent;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
    mpsc,
};

pub(super) struct Envelope {
    queued: Queued,
    _capacity: Capacity,
}

enum Queued {
    Ready {
        command: Command,
        fallback: Option<Fallback>,
    },
    Preparing {
        receiver: mpsc::Receiver<Option<ClientPaneInputEvent>>,
        cancellation: Cancellation,
    },
}

struct Capacity(Arc<Mailbox>);

impl Drop for Capacity {
    fn drop(&mut self) {
        self.0.queued.fetch_sub(1, Ordering::AcqRel);
        if self.0.blocked.swap(false, Ordering::AcqRel) {
            self.0.changed.store(true, Ordering::Release);
        }
    }
}

/// A conditional release is sent only if the clipboard probe used its original key.
#[derive(Clone, Default)]
pub(crate) struct Fallback(Arc<AtomicBool>);

impl Fallback {
    pub(crate) fn select(&self) {
        self.0.store(true, Ordering::Release);
    }
}

pub(crate) struct Preparation {
    sender: Option<mpsc::SyncSender<Option<ClientPaneInputEvent>>>,
    cancellation: Cancellation,
}

#[derive(Clone, Default)]
pub(crate) struct Cancellation(Arc<AtomicBool>);

impl Cancellation {
    pub(crate) fn cancel(&self) {
        self.0.store(true, Ordering::Release);
    }
}

impl Preparation {
    pub(crate) fn cancellation(&self) -> Cancellation {
        self.cancellation.clone()
    }
    pub(crate) fn finish(mut self, input: Option<ClientPaneInputEvent>) {
        if let Some(sender) = self.sender.take() {
            let _ = sender.try_send(input);
        }
    }
}

impl Drop for Preparation {
    fn drop(&mut self) {
        if self.sender.is_some() {
            self.cancellation().cancel();
        }
    }
}

pub(super) enum Next {
    Apply(Command),
    Skip,
    Wait,
}

impl Envelope {
    pub(super) fn poll(&mut self) -> Next {
        match &self.queued {
            Queued::Ready { command, fallback } => {
                if fallback
                    .as_ref()
                    .is_some_and(|gate| !gate.0.load(Ordering::Acquire))
                {
                    Next::Skip
                } else {
                    Next::Apply(command.clone())
                }
            }
            Queued::Preparing {
                receiver,
                cancellation,
            } => {
                if cancellation.0.load(Ordering::Acquire) {
                    return Next::Skip;
                }
                match receiver.try_recv() {
                    Ok(Some(input)) => Next::Apply(Command::Input(input)),
                    Ok(None) | Err(mpsc::TryRecvError::Disconnected) => Next::Skip,
                    Err(mpsc::TryRecvError::Empty) => Next::Wait,
                }
            }
        }
    }
}

impl Link {
    pub(crate) fn prepare(&self) -> Option<Preparation> {
        let (sender, receiver) = mpsc::sync_channel(1);
        let cancellation = Cancellation::default();
        self.enqueue(
            Queued::Preparing {
                receiver,
                cancellation: cancellation.clone(),
            },
            true,
        )
        .then(|| Preparation {
            sender: Some(sender),
            cancellation,
        })
    }

    pub(crate) fn send_release(&self, input: ClientPaneInputEvent, fallback: Fallback) -> bool {
        self.enqueue(
            Queued::Ready {
                command: Command::Input(input),
                fallback: Some(fallback),
            },
            true,
        )
    }

    pub(super) fn enqueue_command(&self, command: Command) -> bool {
        let input = matches!(command, Command::Input(_));
        self.enqueue(
            Queued::Ready {
                command,
                fallback: None,
            },
            input,
        )
    }

    fn enqueue(&self, queued: Queued, input: bool) -> bool {
        if self
            .mailbox
            .queued
            .try_update(Ordering::AcqRel, Ordering::Acquire, |used| {
                (used < MAX_COMMANDS).then_some(used + 1)
            })
            .is_err()
        {
            if !self.mailbox.blocked.swap(true, Ordering::AcqRel) {
                self.mailbox.changed.store(true, Ordering::Release);
            }
            if input {
                self.failure(crate::i18n::fl!("core-terminals-input-busy"));
            }
            return false;
        }
        let envelope = Envelope {
            queued,
            _capacity: Capacity(self.mailbox.clone()),
        };
        match self.commands.try_send(envelope) {
            Ok(()) => true,
            Err(mpsc::TrySendError::Disconnected(_)) => {
                self.failure(crate::i18n::fl!("core-terminals-input-unavailable"));
                false
            }
            Err(mpsc::TrySendError::Full(_)) => {
                self.failure(crate::i18n::fl!("core-terminals-input-busy"));
                false
            }
        }
    }
}
