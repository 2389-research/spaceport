//! What the window knows of one chat: the transcript as of its cursor, and
//! the commands the program has not acknowledged yet. Pure state, applied
//! from a link's messages, so ordering and fencing are tested without a
//! socket.

use super::{Error, MAX_PENDING, Result};
use sheprd_chat_protocol::{
    ClientMessage, Command, CommandId, Cursor, EntryId, Rejection, Replica, ServerMessage, Touched,
    Transcript, VERSION,
};
use std::collections::HashMap;

/// Connections a command reaches before it is given up: a program that
/// drops the connection on a command it cannot read would otherwise get it
/// again on every reconnect and never come back. Only connections the
/// program greeted count, so an outage alone never drops a prompt.
const SENDS: u8 = 2;

/// A command sent and not yet acknowledged, resent on a new connection.
#[derive(Clone, Debug, PartialEq, Eq)]
struct Pending {
    command: Command,
    id: CommandId,
    /// Greeted connections it went out on.
    sends: u8,
}

#[derive(Debug, Default)]
pub(crate) struct Session {
    /// The agent the program greeted with, until a transcript names one.
    agent: String,
    pending: Vec<Pending>,
    /// The last command the program refused, shown until the next one.
    rejection: Option<Rejection>,
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
    welcomed: bool,
}

impl Session {
    /// A new connection: it must greet before anything else counts.
    pub(crate) fn begin(&mut self) {
        self.welcomed = false;
    }

    /// The connection ended. After a change that did not fit, the next one
    /// asks for a snapshot; the transcript stays on screen meanwhile.
    pub(crate) fn lost(&mut self, error: &Error) {
        self.welcomed = false;
        // `receive` forgot the position already when it returned such an
        // error; this covers one that reaches the session any other way.
        if error.desynced() {
            self.replica.lose();
        }
    }

    /// Where the next connection resumes.
    pub(crate) fn since(&self) -> Option<Cursor> {
        self.replica.since()
    }

    /// Whether a transcript has arrived since the session began.
    pub(crate) fn synced(&self) -> bool {
        self.snapshots > 0
    }

    /// Greeted, and holding the program's current transcript.
    pub(crate) fn live(&self) -> bool {
        self.welcomed && self.replica.since().is_some()
    }

    pub(crate) fn transcript(&self) -> &Transcript {
        self.replica.transcript()
    }

    /// A log observer cannot infer live execution state from historical text.
    /// Its daemon supplies status without changing the transcript wire cursor.
    pub(crate) fn set_observed_status(&mut self, status: sheprd_chat_protocol::Status) -> bool {
        let transcript = self.replica.transcript_mut();
        if !transcript.read_only || transcript.status == status {
            return false;
        }
        transcript.status = status;
        true
    }

    pub(crate) fn agent(&self) -> &str {
        let transcript = self.transcript();
        if transcript.agent.is_empty() {
            &self.agent
        } else {
            &transcript.agent
        }
    }

    pub(crate) fn rejection(&self) -> Option<Rejection> {
        self.rejection
    }

    pub(crate) fn snapshots(&self) -> u64 {
        self.snapshots
    }

    /// Whether a snapshot changed the last entry since this was last asked.
    /// A snapshot drops the revisions, so only it can tell a reply that grew
    /// while the connection was down from the same reply sent again.
    pub(crate) fn take_tail_changed(&mut self) -> bool {
        std::mem::take(&mut self.tail_changed)
    }

    /// Each entry with the revision it last changed at. A sub-agent's call
    /// lists its steps, so it changes with the latest of them too.
    pub(crate) fn keys(&self) -> Vec<(EntryId, u64)> {
        let revision = |id: &EntryId| self.revisions.get(id).copied().unwrap_or(0);
        let mut steps: HashMap<&str, u64> = HashMap::new();
        for entry in &self.transcript().entries {
            if let sheprd_chat_protocol::EntryKind::Tool(tool) = &entry.kind
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
                    sheprd_chat_protocol::EntryKind::Tool(tool) => {
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
    /// One that reached [`SENDS`] greeted connections, each of which ended
    /// without an answer, is given up and shown as refused.
    pub(crate) fn resend(&mut self) -> Vec<ClientMessage> {
        let before = self.pending.len();
        self.pending.retain(|pending| pending.sends < SENDS);
        if self.pending.len() < before {
            self.rejection = Some(Rejection::Unsupported);
        }
        self.pending
            .iter()
            .map(|pending| ClientMessage::Command {
                command: pending.command.clone(),
                id: pending.id,
            })
            .collect()
    }

    /// Records `command` as waiting and returns the message that sends it.
    pub(crate) fn submit(&mut self, command: Command) -> Result<ClientMessage> {
        if self.transcript().read_only {
            return Err(Error::ReadOnly);
        }
        if self.pending.len() >= MAX_PENDING {
            return Err(Error::Busy);
        }
        let id = CommandId::random();
        self.rejection = None;
        self.pending.push(Pending {
            command: command.clone(),
            id,
            // Out now on a greeted connection, else with the next one's resends.
            sends: u8::from(self.welcomed),
        });
        Ok(ClientMessage::Command { command, id })
    }

    /// Applies one message, returning whether anything shown changed. An
    /// error ends the connection; the session keeps the transcript it had,
    /// and after a change that did not follow or fit, no stream position.
    pub(crate) fn receive(&mut self, message: ServerMessage) -> Result<bool> {
        match message {
            ServerMessage::Welcome { agent, version, .. } => {
                if version != VERSION {
                    return Err(Error::Version(version));
                }
                self.agent = agent;
                self.welcomed = true;
                // The resends went out right after the greeting.
                for pending in &mut self.pending {
                    pending.sends = pending.sends.saturating_add(1);
                }
                Ok(true)
            }
            _ if !self.welcomed => Err(Error::Handshake),
            ServerMessage::Snapshot { cursor, transcript } => {
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
                            sheprd_chat_protocol::Error::StreamGap { after, got } => {
                                Error::Stream { after, got }
                            }
                            misfit => Error::Apply(misfit),
                        })?;
                self.touch(touched);
                Ok(true)
            }
            ServerMessage::Ack { id, rejected } => {
                self.pending.retain(|pending| pending.id != id);
                // Another client answering first is how a race ends, not
                // a failure worth showing.
                let shown = rejected.filter(|rejection| *rejection != Rejection::UnknownPermission);
                if shown.is_some() {
                    self.rejection = shown;
                }
                Ok(shown.is_some())
            }
            ServerMessage::Bye { reason } => Err(Error::Bye(reason)),
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
