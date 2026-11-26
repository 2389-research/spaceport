//! The attach side's failures, from `error-chat.ftl`: the transcript a chat
//! holds, the program it reaches, and the environment its tab is created with.

use crate::Error;
use spaceport_i18n::{Localize, fl};

impl Localize for Error {
    fn localize(&self) -> String {
        match self {
            // keep-sorted start
            Self::Apply(_) => fl!("error-chat-apply"),
            Self::Busy => fl!("error-chat-busy"),
            // The chat protocol's own name for the reason.
            Self::Bye(reason) => fl!("error-chat-bye", reason = format!("{reason:?}")),
            Self::Closed => fl!("error-chat-closed"),
            Self::Connect(_) => fl!("error-chat-connect"),
            Self::Environment(error) => error.localize(),
            // The existing-agent probe is shown in its own words: the app
            // has no message for it and no path attaches to it.
            Self::Existing(error) => error.to_string(),
            Self::Handshake => fl!("error-chat-handshake"),
            Self::NoEndpointDir => fl!("error-chat-no-endpoint-dir"),
            Self::NotAttached => fl!("error-chat-not-attached"),
            Self::Overflow => fl!("error-chat-overflow"),
            Self::Read(_) => fl!("error-chat-read"),
            Self::ReadOnly => fl!("error-chat-read-only"),
            Self::Remote(_) => fl!("error-chat-remote"),
            Self::Spawn(_) => fl!("error-chat-spawn"),
            // Stream positions, as the chat protocol writes them.
            Self::Stream { after, got } => fl!(
                "error-chat-stream",
                got = format!("{got:?}"),
                after = format!("{after:?}")
            ),
            Self::Version(version) => fl!("error-chat-version", version = *version),
            // keep-sorted end
        }
    }
}
