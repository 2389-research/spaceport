//! The link as the words commands and pickers name it by.
use super::link::{Failure, Link};
use spaceport_client::VersionMismatch;

/// Where a host's link stands.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase {
    Disabled,
    Detached,
    /// The daemon refused the handshake as too old for this app.
    DaemonOutdated,
    /// The daemon refused the handshake as too new for this app.
    ClientOutdated,
    /// This computer has no Herdr to start.
    HerdrMissing,
    /// Herdr explicitly stopped and requires an explicit reconnect.
    HerdrStopped,
    Reconnecting,
    Connecting,
    Online,
}

impl Phase {
    /// The phase as `endpoint-status` selects on it.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Disabled => "disabled",
            Self::Detached => "detached",
            Self::DaemonOutdated => "daemon-outdated",
            Self::ClientOutdated => "client-outdated",
            Self::HerdrMissing => "herdr-missing",
            Self::HerdrStopped => "herdr-stopped",
            Self::Reconnecting => "reconnecting",
            Self::Connecting => "connecting",
            Self::Online => "online",
        }
    }

    /// Whether this host wants a connection: only a disabled one does not, so
    /// the phase is where every caller reads it, including the rows of a
    /// [`HostView`], which hold no host.
    ///
    /// [`HostView`]: super::HostView
    pub fn enabled(self) -> bool {
        self != Self::Disabled
    }

    /// The phase as `herdr-gpui` commands report it, in English. Command output
    /// is data rather than a word the app shows in the user's language, which
    /// the window words as `endpoint-status`.
    pub fn status(self) -> &'static str {
        match self {
            Self::DaemonOutdated => "Herdr update needed", // i18n: data
            Self::ClientOutdated => "app update needed",   // i18n: data
            Self::HerdrMissing => "Herdr not installed",   // i18n: data
            Self::HerdrStopped => "Herdr stopped",         // i18n: data
            phase => phase.as_str(),
        }
    }
}

pub(super) fn of(link: &Link) -> Phase {
    let why = match link {
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
        Some(Failure::Stopped { .. }) => Phase::HerdrStopped,
        Some(Failure::Transport { .. }) => Phase::Reconnecting,
        None => Phase::Connecting,
    }
}
