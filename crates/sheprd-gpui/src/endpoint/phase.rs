// ABOUTME: Projects the stored endpoint link into stable command and picker words.
// ABOUTME: Keeps public phase selectors unchanged across lifecycle transitions.
// i18n: migrated
use super::link::Failure;
use super::{Endpoint, Link};
use crate::i18n::fl;
use sheprd_client::VersionMismatch;

/// Where an endpoint's link stands.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Phase {
    Disabled,
    Detached,
    /// The daemon refused the handshake as too old for this app.
    DaemonOutdated,
    /// The daemon refused the handshake as too new for this app.
    ClientOutdated,
    /// This computer has no Herdr to start.
    HerdrMissing,
    Reconnecting,
    Connecting,
    Online,
}

impl Phase {
    /// The phase as `endpoint-status` selects on it.
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Disabled => "disabled",
            Self::Detached => "detached",
            Self::DaemonOutdated => "daemon-outdated",
            Self::ClientOutdated => "client-outdated",
            Self::HerdrMissing => "herdr-missing",
            Self::Reconnecting => "reconnecting",
            Self::Connecting => "connecting",
            Self::Online => "online",
        }
    }

    /// The phase as `herdr-gpui` commands report it, in English.
    pub(crate) fn status(self) -> &'static str {
        match self {
            Self::DaemonOutdated => "Herdr update needed", // i18n: data
            Self::ClientOutdated => "app update needed",   // i18n: data
            Self::HerdrMissing => "Herdr not installed",   // i18n: data
            phase => phase.as_str(),
        }
    }
}

impl Endpoint {
    /// The stored lifecycle, independent of a window's older worker projection.
    pub(crate) fn phase(&self) -> Phase {
        let why = match &self.link {
            Link::Disabled => return Phase::Disabled,
            Link::Detached { .. } => return Phase::Detached,
            Link::Up { .. } => return Phase::Online,
            Link::Waiting { why, .. } | Link::Dialing { why, .. } => why,
        };
        match why {
            Some(Failure::Version {
                mismatch: VersionMismatch::DaemonOutdated { .. },
                ..
            }) => Phase::DaemonOutdated,
            Some(Failure::Version {
                mismatch: VersionMismatch::ClientOutdated { .. },
                ..
            }) => Phase::ClientOutdated,
            Some(Failure::Missing { .. }) => Phase::HerdrMissing,
            Some(Failure::Transport { .. }) => Phase::Reconnecting,
            None => Phase::Connecting,
        }
    }

    /// The endpoint's state as `herdr-gpui` commands report it, in English.
    pub fn status(&self) -> &'static str {
        self.phase().status()
    }

    /// The endpoint's state as the device picker shows it.
    pub fn status_label(&self) -> String {
        fl!("endpoint-status", state = self.phase().as_str())
    }

    /// Whether a live transport has delivered a snapshot. Narrower than
    /// [`Phase::Online`], which a link still waiting for its first snapshot
    /// already is.
    pub(super) fn serving(&self) -> bool {
        self.connection
            .handle
            .as_ref()
            .is_some_and(|handle| !handle.is_disconnected())
            && self.live.status.is_connected()
            && self.live.snapshot.is_some()
    }

    /// Whether the endpoint wants a connection and has no transport.
    pub(super) fn may_dial(&self) -> bool {
        matches!(self.link, Link::Waiting { .. }) && self.connection.retry_ready()
    }
}
