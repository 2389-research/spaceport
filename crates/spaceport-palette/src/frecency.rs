//! The destination a row is remembered by: how often and how recently it was
//! chosen, so a typed query ranks familiar rows higher, as snacks.picker's
//! frecency does. The history itself is the app's, kept per user on this
//! machine and never sent to the daemon.
//!
//! Workspaces are remembered by host, launch path, and branch, which survive
//! a daemon restart. Tab and pane IDs do not, so those are remembered by the
//! daemon boot they were chosen in: a reused ID after a restart never
//! inherits another pane's history, and the old entry simply ages out.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Bytes in any one field of a key. A longer key is not remembered, so every
/// key the store accepts fits in the file it reads back; the app sizes that
/// file from this bound.
pub const MAX_FIELD_BYTES: usize = 1024;

/// A destination the palette remembers choosing.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Key {
    Workspace {
        endpoint: String,
        root: String,
        branch: Option<String>,
    },
    Tab {
        endpoint: String,
        boot: String,
        id: String,
    },
    Pane {
        endpoint: String,
        boot: String,
        id: String,
    },
    Project {
        path: PathBuf,
    },
}

impl Key {
    pub fn valid(&self) -> bool {
        let fits = |text: &str| !text.is_empty() && text.len() <= MAX_FIELD_BYTES;
        match self {
            Self::Workspace {
                endpoint,
                root,
                branch,
            } => fits(endpoint) && fits(root) && branch.as_deref().is_none_or(fits),
            Self::Tab { endpoint, boot, id } | Self::Pane { endpoint, boot, id } => {
                fits(endpoint) && fits(boot) && fits(id)
            }
            Self::Project { path } => path.to_str().is_some_and(fits),
        }
    }
}
