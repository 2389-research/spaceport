//! What a notice this crate raises cannot word alone: an agent notice's task
//! and place, and a refused command as the person reading it sees it. Both
//! need what only the app has — the sidebar's naming of a workspace, and the
//! wording of a client error — so the app hands them down as `fn` items, and
//! every other word a notice says is this crate's own.

use spaceport_client::protocol::ClientShellSnapshot;

/// An agent notice's task and place for `pane`, or nothing when the snapshot
/// knows neither and the daemon's own body stays.
pub type Describe = fn(&ClientShellSnapshot, &str) -> Option<String>;

/// A refused command as the person reading it sees it.
pub type Localize = fn(&spaceport_client::Error) -> String;
