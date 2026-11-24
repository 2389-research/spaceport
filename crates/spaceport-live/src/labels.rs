//! The words the reducer writes into a connection's live state: a notice with
//! no title, the two notices the reducer itself raises, how a refused command
//! reads, and the task and place an agent notice names. A crate below the app
//! has no messages (`docs/i18n.md`), so this names the shape only: the app
//! reads the words in the language it shows and hands them down.

use spaceport_client::protocol::ClientShellSnapshot;

/// The live state's words, in the language the app showed when they were read.
#[derive(Clone, Debug)]
pub struct Labels {
    /// What an activation the daemon did not acknowledge says.
    pub activation_invalid: String,
    /// An agent notice's task and place for `pane`, or nothing when the
    /// snapshot knows neither and the daemon's own body stays.
    pub describe: fn(&ClientShellSnapshot, &str) -> Option<String>,
    /// The notice over a connection whose host key was never verified.
    pub host_key_unverified: String,
    /// A refused command as the person reading it sees it.
    pub localize: fn(&spaceport_client::Error) -> String,
    /// What a notice with no title says.
    pub notification_untitled: String,
}
