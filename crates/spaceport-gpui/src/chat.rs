// ABOUTME: Attaches the native chat view to agent conversations owned by Herdr panes.
// ABOUTME: Discovers helper executables and keeps chat input and permissions synchronized.
//! A native view of a `herdr-acp` chat, drawn over the pane the program runs
//! in. The program is the agent's only ACP client and draws its own terminal
//! view; it also listens on a private endpoint (a socket on Unix, a named
//! pipe on Windows), and this window attaches there as one more client of
//! the same conversation. Both views stay
//! interactive: a prompt, a cancel, or a permission answer may come from
//! either, and the first answer to a request wins.
//!
//! The pane names its program through a metadata token holding an
//! [`EndpointId`]; the address is derived from it (in this user's own state
//! directory on Unix), never read from daemon data, and only for a daemon on
//! this machine that this user owns. Each attached pane has a worker
//! ([`link`]) that does the blocking I/O, a [`session`] that applies what it
//! reads, and a view drawn from prepared state. A pane whose endpoint cannot
//! be reached shows its terminal cells as before.
// i18n: migrated

mod chats;
mod core_pane;
mod discovery;
mod files;
mod glide;
mod harnesses;
pub(crate) mod page;
pub(crate) mod pasted;
mod prepared;
#[cfg(feature = "integration-test")]
pub(crate) mod preview;
mod reader;
mod rich;
mod selection;
mod start;
mod terminal;
mod text;
mod view;
pub(crate) mod visual;
pub(crate) mod widgets;
mod window;

pub(crate) use chats::{ChatInput, Chats, LauncherDeck, Toward};
// The attach side and the model below the views, at the paths the rest of the
// chat reads them at.
pub(crate) use spaceport_chat_core::env::{check_env, env_text, parse_env};
pub(crate) use spaceport_chat_core::session::Refusal;
pub(crate) use spaceport_chat_core::{
    MAX_CHATS, env, existing, highlight, link, list, session, summary,
};
#[cfg(feature = "integration-test")]
pub(crate) use start::native as native_start;
pub(crate) use start::{Error as StartError, Program, Receipt};

/// The wall clock in milliseconds since the Unix epoch, which the program
/// on this machine stamps a running turn with.
pub(super) fn now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |elapsed| {
            u64::try_from(elapsed.as_millis()).unwrap_or(u64::MAX)
        })
}
pub(crate) use core_pane::key as core_pane_key;

use spaceport_chat_protocol::EndpointId;

/// Connection failures in a row after which the pane shows its terminal.
const MAX_FAILURES: u32 = 3;

pub(crate) type Result<T, E = Error> = std::result::Result<T, E>;

/// The failures a chat has: the attach side's own, which the model below the
/// views reports, and the ones only the app can have.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    // keep-sorted start
    /// Connecting to the program, and the transcript it carries.
    #[error(transparent)]
    Attach(#[from] spaceport_chat_core::Error),
    #[error(transparent)]
    Environment(#[from] env::Error),
    #[error(transparent)]
    Pasted(#[from] pasted::Error),
    #[error(transparent)]
    Start(#[from] start::Error),
    // keep-sorted end
}

impl Error {
    /// Whether trying again cannot help, so the pane keeps its terminal. Only
    /// a failure to attach can be permanent; the app's own are retried.
    fn permanent(&self) -> bool {
        match self {
            Self::Attach(error) => error.permanent(),
            Self::Environment(_) | Self::Pasted(_) | Self::Start(_) => false,
        }
    }

    /// Whether the client's copy no longer fits the program's stream, so the
    /// next connection asks for a whole snapshot.
    fn desynced(&self) -> bool {
        match self {
            Self::Attach(error) => error.desynced(),
            Self::Environment(_) | Self::Pasted(_) | Self::Start(_) => false,
        }
    }
}

/// The pane a chat belongs to, where its transcript comes from, and how the
/// window reaches that source. Every field is part of the chat's identity:
/// a pane whose target changes gets a new connection and a new transcript.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub(crate) struct Target {
    /// The chat's key in the window: a Herdr pane's id, or for a herdr-core
    /// terminal the key [`core_pane::key`] makes. Code that acts on the pane
    /// itself matches [`Self::surface`] instead.
    pub(crate) pane_id: String,
    pub(crate) route: Route,
    pub(crate) source: Source,
    pub(crate) surface: Surface,
}

/// Where a chat pane's transcript comes from.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub(crate) enum Source {
    /// An agent CLI the user started themselves, watched read-only by a
    /// `sheprd-acp observe` the window starts. Two panes may observe one
    /// session, so the binding names its pane too.
    Observed(existing::Binding),
    /// `sheprd-acp` running in the pane, reached at its attach endpoint. A
    /// `[chat.mirrors]` entry is one as well; its transcript says whether it
    /// takes prompts.
    Program(EndpointId),
}

/// Which daemon's pane a chat is drawn over.
#[derive(Clone, Debug, Default, Eq, Hash, PartialEq)]
pub(crate) enum Surface {
    /// A pane of the selected Herdr, drawn over its surface frame.
    #[default]
    Herdr,
    /// A herdr-core terminal, drawn over its pane in a herdr-core tab.
    Core(spaceport_backend::TerminalId),
}

impl Target {
    /// Whether the chat's program runs on this machine, so the paths in its
    /// conversation name files here.
    pub(crate) fn is_local(&self) -> bool {
        self.route == Route::Local
    }
}

/// How the window reaches a chat's program.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub(crate) enum Route {
    /// Its socket in this user's private directory on this machine.
    Local,
    /// `herdr-acp connect` on the SSH host the daemon runs on, carried over
    /// `ssh`; the program there is reached as that host's user.
    Ssh(String),
}

/// Folders a remote `sheprd-acp` is looked for in before the remote `PATH`,
/// which a noninteractive SSH shell often leaves short. `~/.herdr/bin` is
/// where `cargo install --root ~/.herdr` puts it. A Mac has it only inside the
/// app bundle, which never joins `PATH`. Older supported bundle names remain discoverable.
const REMOTE_PROGRAM_DIRS: &[&str] = &[
    "$HOME/.herdr/bin",
    "$HOME/.cargo/bin",
    "$HOME/.local/bin",
    "$HOME/Applications/Spaceport.app/Contents/MacOS", // i18n: data
    "$HOME/Applications/Herdr.app/Contents/MacOS",     // i18n: data
    "$HOME/Applications/Sheprd.app/Contents/MacOS",    // i18n: data
    "/Applications/Spaceport.app/Contents/MacOS",      // i18n: data
    "/Applications/Herdr.app/Contents/MacOS",          // i18n: data
    "/Applications/Sheprd.app/Contents/MacOS",         // i18n: data
];

/// The remote `/bin/sh` script that runs the first `sheprd-acp` found with
/// `args`, or says where it looked and exits 127. `args` must be
/// shell-safe; the script has no single quotes, so it may be wrapped in them.
pub(crate) fn remote_program_script(args: &str) -> String {
    let candidates: String = ["sheprd-acp", "herdr-acp"]
        .into_iter()
        .flat_map(|program| {
            REMOTE_PROGRAM_DIRS
                .iter()
                .map(move |dir| format!("\"{dir}/{program}\" "))
                .chain(std::iter::once(format!("\"$(command -v {program})\" ")))
        })
        .collect();
    format!(
        // i18n: data
        "for p in {candidates}; do [ -x \"$p\" ] && exec \"$p\" {args}; done; \
         echo \"sheprd-acp is not installed here: install it in ~/.herdr/bin\" >&2; exit 127"
    )
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests;
