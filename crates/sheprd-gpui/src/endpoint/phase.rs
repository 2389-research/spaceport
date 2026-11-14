//! Where an endpoint's link stands, derived in one place from the fields its
//! lifecycle is kept in, and the words commands and the device picker use.
// i18n: migrated
use super::Endpoint;
use crate::{LiveState, i18n::fl};
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
            phase => phase.as_str(),
        }
    }
}

impl Endpoint {
    /// Where the link stands by the endpoint's own copy of its state.
    pub(crate) fn phase(&self) -> Phase {
        self.phase_with(&self.live)
    }

    /// Where the link stands when `live` is the endpoint's current state:
    /// the window's copy for the selected endpoint, see
    /// [`crate::SheprdWindow::live_of`].
    pub(crate) fn phase_with(&self, live: &LiveState) -> Phase {
        if !self.enabled {
            Phase::Disabled
        } else if self.detached {
            Phase::Detached
        } else if let Some(mismatch) = &self.version_mismatch {
            // A refused handshake is an outage too, but says what fixes it.
            match mismatch {
                VersionMismatch::DaemonOutdated { .. } => Phase::DaemonOutdated,
                VersionMismatch::ClientOutdated { .. } => Phase::ClientOutdated,
            }
        } else if self.outage.is_some() {
            // Before `live`: a stopped handle can precede its disconnect state.
            Phase::Reconnecting
        } else if live.status.is_connected() {
            Phase::Online
        } else if live.error.is_some() {
            Phase::Reconnecting
        } else {
            Phase::Connecting
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
        self.connection.handle.is_some()
            && self.live.status.is_connected()
            && self.live.snapshot.is_some()
    }

    /// Whether the endpoint wants a connection and has no transport.
    pub(super) fn may_dial(&self) -> bool {
        self.enabled && !self.detached && self.connection.handle.is_none()
    }
}
