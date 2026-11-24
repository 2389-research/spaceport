// ABOUTME: Holds a connection's live state at the path the window reads it at.
// ABOUTME: Words the state writes itself, and the status bar's text, are the app's.
//! A connection's live state, as the window reads it. The value and the rules
//! over it are `spaceport-live`'s (`docs/notes-endpoint-extraction.md`); this
//! module is the path the rest of the window has always named them by, plus the
//! words a crate below the app cannot read: the status bar's line, and the
//! [`Labels`] every applied event writes its notices from.
// i18n: migrated
use crate::i18n::{Localize, fl};
use std::time::Instant;

#[cfg(test)]
pub(crate) use spaceport_live::{
    AgentRecency, MAX_DAEMON_VERSION_CHARS, MAX_WINDOW_TITLE_CHARS, SurfaceActivation,
};
pub(crate) use spaceport_live::{
    AnnouncementDismissal, Answer, ConnectionStatus, DialogResponse, LiveState, RenameResult,
    activation,
};

/// What the status bar says of the connection; `Display` is the log's.
pub(crate) fn status_label(status: ConnectionStatus) -> String {
    match status {
        ConnectionStatus::Connecting => fl!("status-bar-connecting"),
        ConnectionStatus::StartingDaemon => fl!("status-bar-starting-daemon"),
        ConnectionStatus::AwaitingSnapshot => fl!("status-bar-awaiting-snapshot"),
        ConnectionStatus::Connected => fl!("status-bar-connected"),
        ConnectionStatus::Disconnected => fl!("status-bar-disconnected"),
        ConnectionStatus::Detached => fl!("status-bar-detached"),
    }
}

/// The bar's text as it reads at `now`: a caller that already holds an instant
/// passes it so nothing here reads a second one.
pub(crate) fn status_text(live: &LiveState, local_error: Option<&str>, now: Instant) -> String {
    let error = if live.status.is_connected() {
        local_error.or(live.notices.text_at(now))
    } else {
        live.notices.connection_reason()
    };
    match error {
        Some(error) => fl!(
            "status-bar-status-error",
            status = status_label(live.status),
            error = error
        ),
        None => status_label(live.status),
    }
}

/// The words the live state writes into itself, in the language this window
/// shows: read once for each event the reducer applies.
pub(crate) fn labels() -> spaceport_live::Labels {
    spaceport_live::Labels {
        activation_invalid: fl!("state-activation-invalid"),
        describe: crate::notifications::context::describe,
        host_key_unverified: fl!("state-host-key-unverified"),
        localize: |error: &spaceport_client::Error| error.localize(),
        notification_untitled: fl!("notifications-untitled"),
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests;
