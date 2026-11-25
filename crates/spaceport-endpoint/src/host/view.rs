//! One listed host as a row paints it.
//!
//! [`HostView`] is the endpoint a sidebar row is handed once everything a
//! socket owns is left behind: the identity and label the app words, the phase
//! of the link and the status of the connection, and the two folds the user
//! made on it. Nothing here is a target, a handle, a message or a word, so a
//! painter reads it without reaching a connection.

use super::Phase;
use spaceport_live::ConnectionStatus;
use std::collections::HashSet;

/// One listed host, as the row and rail renderers read it. The app prepares
/// one from the endpoint it drives; every field is the app's or the link's,
/// copied out rather than borrowed through a connection.
#[derive(Clone, Copy, Debug)]
pub struct HostView<'a> {
    pub id: &'a str,
    pub label: &'a str,
    pub phase: Phase,
    /// What the host's own connection last reported. The window keeps the
    /// selected host's state separately, as [`Host::live`](super::Host::live)
    /// documents, and reads this copy only for the status a row shows.
    pub status: ConnectionStatus,
    /// Whether the user folded this host's rows away.
    pub collapsed: bool,
    /// The repositories whose worktrees the user folded within this host.
    pub collapsed_repos: &'a HashSet<String>,
}
