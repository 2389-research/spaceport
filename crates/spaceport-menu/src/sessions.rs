//! The rows under an unfolded device in the device picker: a selectable row
//! with what it paints, a note that explains the list, and the device and host
//! a session is managed through. The app words every one of them and does the
//! scanning, so a row here names no sentence.

use spaceport_client::RemoteHost;

/// A line under an unfolded device, in the order it is painted. Only a row can
/// be selected, so navigation walks the `Row` entries and skips notes.
pub enum Entry {
    /// A muted note, carrying the selector diagnostics and tests look for.
    Muted {
        selector: String,
        text: String,
    },
    Error(String),
    /// A selectable line, carrying what it paints: a row resolved again at
    /// paint time could describe a session its own scan has already replaced.
    Row {
        row: Row,
        spec: Spec,
    },
}

impl Entry {
    /// The row this line chooses, for the ones that choose.
    pub fn row(&self) -> Option<&Row> {
        match self {
            Self::Row { row, .. } => Some(row),
            _ => None,
        }
    }
}

/// What a row attaches to. A device keeps its endpoint id rather than an index,
/// so a catalog change between paint and click cannot pick the wrong one, and it
/// names the session itself, because a device has many.
#[derive(Clone, Debug, PartialEq)]
pub enum Row {
    Local(String),
    Device { id: String, session: String },
    Add(Target),
}

/// What one row paints. Both kinds share one shape so they cannot drift apart.
pub struct Spec {
    pub label: String,
    pub detail: String,
    /// Running locally, or connected remotely.
    pub online: bool,
    pub checked: bool,
    pub enabled: bool,
}

/// What a session is managed through: this machine, or a device and the host
/// it reaches. A picker form captures that, never a mutable row ordinal.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Target {
    Local,
    Device { id: String, host: RemoteHost },
}
