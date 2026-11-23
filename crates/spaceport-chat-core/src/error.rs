//! The failures the chat model below the views reports as one type.
//!
//! The attach-side variants are what a connection to the program and the
//! transcript it carries can fail with; the app wraps this type in its own
//! `chat::Error` for the failures it alone can have (a pasted file, starting
//! a program).

use spaceport_chat_protocol::{ByeReason, Cursor};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    // keep-sorted start
    /// A change that does not fit the transcript this client holds.
    #[error("a chat change does not fit the transcript")]
    Apply(#[source] spaceport_chat_protocol::Error),
    #[error("too many chat commands are waiting for an answer")]
    Busy,
    #[error("the chat program left: {0:?}")]
    Bye(ByeReason),
    #[error("the chat program closed the connection")]
    Closed,
    #[error("could not reach the chat program")]
    Connect(#[source] std::io::Error),
    /// The `[chat.env]` table a chat's tab would be created with.
    #[error(transparent)]
    Environment(#[from] crate::env::Error),
    /// Discovering the agent conversation already running in a pane.
    #[error(transparent)]
    Existing(#[from] crate::existing::Error),
    #[error("the chat program did not greet first")]
    Handshake,

    #[error("no private directory holds chat sockets")]
    NoEndpointDir,
    #[error("the chat view is not attached")]
    NotAttached,
    #[error("the chat view fell behind the program")]
    Overflow,
    #[error("could not read from the chat program")]
    Read(#[source] spaceport_chat_protocol::Error),
    #[error("This is a read-only transcript; use the agent's terminal to reply")]
    ReadOnly,
    #[error("could not reach the chat program over SSH")]
    Remote(#[source] spaceport_client::Error),
    #[error("could not start the chat worker")]
    Spawn(#[source] std::io::Error),
    #[error("chat change {got:?} does not follow {after:?}")]
    Stream { after: Option<Cursor>, got: Cursor },
    #[error("the chat program speaks protocol version {0}")]
    Version(u32),
    // keep-sorted end
}

impl Error {
    /// Whether trying again cannot help, so the pane keeps its terminal.
    pub fn permanent(&self) -> bool {
        matches!(
            self,
            Self::Version(_) | Self::Bye(ByeReason::Version { .. }) | Self::NoEndpointDir
        )
    }

    /// Whether the client's copy no longer fits the program's stream, so the
    /// next connection asks for a whole snapshot.
    pub fn desynced(&self) -> bool {
        matches!(self, Self::Stream { .. } | Self::Apply(_))
    }
}

pub type Result<T, E = Error> = std::result::Result<T, E>;
