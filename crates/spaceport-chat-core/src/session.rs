// ABOUTME: Applies fenced chat transcripts and owns pending command delivery.
// ABOUTME: Distinguishes socket writes from queued work and acknowledgements.
//! What a client knows of one chat: the transcript as of its cursor, and
//! the commands the program has not acknowledged yet. Pure state, applied
//! from a link's messages, so ordering and fencing are tested without a
//! socket.

use super::{Error, MAX_PENDING, Result};
use spaceport_chat_protocol::{
    ClientMessage, Command, CommandId, Cursor, EntryId, Rejection, Replica, ServerMessage, Touched,
    Transcript, VERSION,
};
use std::collections::HashMap;

/// Complete writes before a command is given up: a program that
/// drops the connection on a command it cannot read would otherwise get it
/// again on every reconnect and never come back. Greetings never count.
const SENDS: u8 = 2;

/// A command waiting for its acknowledgement.
#[derive(Clone, Debug, PartialEq, Eq)]
struct Pending {
    command: Command,
    id: CommandId,
    delivery: Delivery,
    epoch: Option<uuid::Uuid>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Delivery {
    Queued { writes: u8 },
    Writing { writes: u8 },
    AwaitingAck { writes: u8 },
}

#[derive(Debug, Default, PartialEq, Eq)]
enum Attachment {
    #[default]
    Unwelcomed,
    Welcomed,
    Current,
}

/// Local delivery uncertainty is distinct from the program refusing a command.
#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
pub enum Refusal {
    #[error(
        "The chat command was not acknowledged. It may have run. Check the conversation before trying again."
    )]
    DeliveryUnconfirmed,
    #[error(transparent)]
    Remote(Rejection),
    #[error(
        "The conversation changed before the chat command was acknowledged. Check the conversation before trying again."
    )]
    SessionChanged,
}

enum Transition {
    Submit {
        command: Command,
        id: CommandId,
        epoch: Option<uuid::Uuid>,
    },
    Written(CommandId),
    Uncertain(CommandId),
    Retry,
    Dispatch,
    Epoch(uuid::Uuid),
    Ack {
        id: CommandId,
        rejected: Option<Rejection>,
    },
}

#[derive(Debug, Default)]
pub struct Session {
    /// The agent the program greeted with, until a transcript names one.
    agent: String,
    pending: Vec<Pending>,
    /// The last command the program refused, shown until the next one.
    rejection: Option<Refusal>,
    /// The transcript and the stream position it reflects. It holds no
    /// position when the next connection must start from a snapshot.
    replica: Replica,
    /// The revision each entry last changed at, so the view redraws only
    /// the entries that did.
    revisions: HashMap<EntryId, u64>,
    revision: u64,
    /// Snapshots applied, so the view starts its list over after one.
    snapshots: u64,
    /// Whether a snapshot brought a different last entry than the session
    /// held, until the view takes it.
    tail_changed: bool,
    /// Whether the current connection has greeted.
    attachment: Attachment,
}

impl Session {
    /// A new connection: it must greet before anything else counts.
    pub fn begin(&mut self) {
        self.attachment = Attachment::Unwelcomed;
        self.transition(Transition::Retry);
    }

    /// The connection ended. After a change that did not fit, the next one
    /// asks for a snapshot; the transcript stays on screen meanwhile.
    pub fn lost(&mut self, error: &Error) {
        self.attachment = Attachment::Unwelcomed;
        // `receive` forgot the position already when it returned such an
        // error; this covers one that reaches the session any other way.
        if error.desynced() {
            self.replica.lose();
        }
    }

    /// Where the next connection resumes.
    pub fn since(&self) -> Option<Cursor> {
        self.replica.since()
    }

    /// Whether a transcript has arrived since the session began.
    pub fn synced(&self) -> bool {
        self.snapshots > 0
    }

    /// Greeted, with the connection's current transcript epoch validated.
    pub fn live(&self) -> bool {
        self.attachment == Attachment::Current && self.replica.since().is_some()
    }

    pub fn transcript(&self) -> &Transcript {
        self.replica.transcript()
    }

    /// A log observer cannot infer live execution state from historical text.
    /// Its daemon supplies status without changing the transcript wire cursor.
    pub fn set_observed_status(&mut self, status: spaceport_chat_protocol::Status) -> bool {
        let transcript = self.replica.transcript_mut();
        if !transcript.read_only || transcript.status == status {
            return false;
        }
        transcript.status = status;
        true
    }

    pub fn agent(&self) -> &str {
        let transcript = self.transcript();
        if transcript.agent.is_empty() {
            &self.agent
        } else {
            &transcript.agent
        }
    }

    pub fn rejection(&self) -> Option<Refusal> {
        self.rejection
    }

    pub fn snapshots(&self) -> u64 {
        self.snapshots
    }

    /// Whether a snapshot changed the last entry since this was last asked.
    /// A snapshot drops the revisions, so only it can tell a reply that grew
    /// while the connection was down from the same reply sent again.
    pub fn take_tail_changed(&mut self) -> bool {
        std::mem::take(&mut self.tail_changed)
    }

    /// Each entry with the revision it last changed at. A sub-agent's call
    /// lists its steps, so it changes with the latest of them too.
    pub fn keys(&self) -> Vec<(EntryId, u64)> {
        let revision = |id: &EntryId| self.revisions.get(id).copied().unwrap_or(0);
        let mut steps: HashMap<&str, u64> = HashMap::new();
        for entry in &self.transcript().entries {
            if let spaceport_chat_protocol::EntryKind::Tool(tool) = &entry.kind
                && let Some(parent) = &tool.parent
            {
                let latest = steps.entry(parent).or_default();
                *latest = (*latest).max(revision(&entry.id));
            }
        }
        self.transcript()
            .entries
            .iter()
            .map(|entry| {
                let own = revision(&entry.id);
                let steps = match &entry.kind {
                    spaceport_chat_protocol::EntryKind::Tool(tool) => {
                        steps.get(tool.call_id.as_str()).copied().unwrap_or(0)
                    }
                    _ => 0,
                };
                (entry.id, own.max(steps))
            })
            .collect()
    }

    /// The commands to send again on a new connection, in order. The
    /// program acknowledges one it already ran without running it twice.
    /// One that reached [`SENDS`] complete writes, each followed by a loss
    /// without an answer, is given up and shown as refused.
    pub fn resend(&mut self) -> Vec<ClientMessage> {
        if self.attachment != Attachment::Current {
            return Vec::new();
        }
        let messages = self
            .pending
            .iter()
            .filter(|pending| matches!(pending.delivery, Delivery::Queued { .. }))
            .map(|pending| ClientMessage::Command {
                command: pending.command.clone(),
                id: pending.id,
            })
            .collect();
        self.transition(Transition::Dispatch);
        messages
    }

    /// Records `command` as waiting and returns the message that sends it.
    pub fn submit(&mut self, command: Command) -> Result<ClientMessage> {
        if self.transcript().read_only {
            return Err(Error::ReadOnly);
        }
        // A displayed transcript with no cursor has lost its epoch fence.
        // Keep new input in the composer until a snapshot restores it.
        if self.synced() && self.since().is_none() {
            return Err(Error::NotAttached);
        }
        if self.pending.len() >= MAX_PENDING {
            return Err(Error::Busy);
        }
        let id = CommandId::random();
        self.transition(Transition::Submit {
            command: command.clone(),
            id,
            epoch: self.since().map(|cursor| cursor.epoch),
        });
        Ok(ClientMessage::Command { command, id })
    }

    /// Applies one message, returning whether anything shown changed. An
    /// error ends the connection; the session keeps the transcript it had,
    /// and after a change that did not follow or fit, no stream position.
    pub fn receive(&mut self, message: ServerMessage) -> Result<bool> {
        match message {
            ServerMessage::Welcome { agent, version, .. } => {
                if version != VERSION {
                    return Err(Error::Version(version));
                }
                self.agent = agent;
                self.attachment = Attachment::Welcomed;
                Ok(true)
            }
            _ if self.attachment == Attachment::Unwelcomed => Err(Error::Handshake),
            ServerMessage::Snapshot { cursor, transcript } => {
                self.transition(Transition::Epoch(cursor.epoch));
                self.attachment = Attachment::Current;
                self.tail_changed |= self.transcript().entries.last() != transcript.entries.last();
                self.replica.reset(cursor, *transcript);
                self.revisions.clear();
                self.snapshots += 1;
                Ok(true)
            }
            ServerMessage::Change { change, cursor } => {
                let touched =
                    self.replica
                        .follow(&change, cursor)
                        .map_err(|error| match error {
                            spaceport_chat_protocol::Error::StreamGap { after, got } => {
                                Error::Stream { after, got }
                            }
                            misfit => Error::Apply(misfit),
                        })?;
                self.touch(touched);
                self.transition(Transition::Epoch(cursor.epoch));
                self.attachment = Attachment::Current;
                Ok(true)
            }
            ServerMessage::Ack { id, rejected } => {
                let before = self.rejection;
                self.transition(Transition::Ack { id, rejected });
                Ok(self.rejection != before)
            }
            ServerMessage::Bye { reason } => Err(Error::Bye(reason)),
        }
    }

    /// Applies writer receipts before a link's final end becomes visible.
    pub fn delivered(&mut self, outcome: super::link::WriteOutcome) -> bool {
        let before = self.rejection;
        self.transition(match outcome {
            super::link::WriteOutcome::Written(id) => Transition::Written(id),
            super::link::WriteOutcome::Uncertain(id) => Transition::Uncertain(id),
        });
        self.rejection != before
    }

    /// The sole writer of pending delivery state and command refusals.
    fn transition(&mut self, event: Transition) {
        match event {
            Transition::Submit { command, id, epoch } => {
                self.rejection = None;
                self.pending.push(Pending {
                    command,
                    id,
                    epoch,
                    delivery: Delivery::Queued { writes: 0 },
                });
            }
            Transition::Written(id) => {
                if let Some(pending) = self.pending.iter_mut().find(|pending| pending.id == id)
                    && let Delivery::Writing { writes } = pending.delivery
                {
                    pending.delivery = Delivery::AwaitingAck { writes: writes + 1 };
                }
            }
            Transition::Uncertain(id) => {
                let before = self.pending.len();
                self.pending.retain(|pending| pending.id != id);
                if self.pending.len() != before {
                    self.rejection = Some(Refusal::DeliveryUnconfirmed);
                }
            }
            Transition::Retry => {
                self.pending.retain_mut(|pending| {
                    let writes = match pending.delivery {
                        Delivery::AwaitingAck { writes } | Delivery::Writing { writes } => writes,
                        Delivery::Queued { .. } => return true,
                    };
                    if writes >= SENDS {
                        self.rejection = Some(Refusal::DeliveryUnconfirmed);
                        return false;
                    }
                    pending.delivery = Delivery::Queued { writes };
                    true
                });
            }
            Transition::Dispatch => {
                for pending in &mut self.pending {
                    if let Delivery::Queued { writes } = pending.delivery {
                        pending.delivery = Delivery::Writing { writes };
                    }
                }
            }
            Transition::Epoch(epoch) => {
                self.pending.retain_mut(|pending| {
                    if pending.epoch.is_some_and(|previous| previous != epoch) {
                        self.rejection = Some(Refusal::SessionChanged);
                        return false;
                    }
                    pending.epoch = Some(epoch);
                    true
                });
            }
            Transition::Ack { id, rejected } => {
                let before = self.pending.len();
                self.pending.retain(|pending| pending.id != id);
                // Only this client's command can change its refusal notice.
                if before != self.pending.len()
                    && let Some(rejected) = rejected.filter(|r| *r != Rejection::UnknownPermission)
                {
                    self.rejection = Some(Refusal::Remote(rejected));
                }
            }
        }
    }

    fn touch(&mut self, touched: Touched) {
        self.revision += 1;
        match touched {
            Touched::Entry(id) => {
                self.revisions.insert(id, self.revision);
            }
            Touched::Truncated { .. } => {
                let entries = &self.replica.transcript().entries;
                self.revisions
                    .retain(|id, _| entries.binary_search_by_key(id, |entry| entry.id).is_ok());
            }
            Touched::Other => {}
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests;
