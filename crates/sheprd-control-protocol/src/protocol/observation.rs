// ABOUTME: Defines the three read-only observations of app-connected sessions.
// ABOUTME: Bounds inert excerpts and keeps opaque source identity separate from labels.
mod error;
pub use error::Error;
use serde::{Deserialize, Serialize};
use sheprd_client::protocol::AgentStatus;

pub const MAX_SESSIONS: usize = 64;
pub const MAX_EXCERPT_ITEMS: usize = 200;
pub const MAX_EXCERPT_BYTES: usize = 16 * 1024;
pub const MAX_OFFSET: usize = 1_000_000;

/// Issued by this running app. It is never a path, address, or command.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Target(pub String);
impl Target {
    pub fn validate(&self) -> Result<(), Error> {
        if self.0.is_empty()
            || self.0.len() > 64
            || !self
                .0
                .bytes()
                .all(|c| c.is_ascii_alphanumeric() || c == b'-')
        {
            return Err(Error::InvalidTarget);
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Bounds {
    #[serde(default)]
    pub offset: usize,
    #[serde(default = "default_items")]
    pub limit: usize,
    #[serde(default = "default_bytes")]
    pub max_bytes: usize,
}
const fn default_items() -> usize {
    80
}
const fn default_bytes() -> usize {
    MAX_EXCERPT_BYTES
}
const fn default_sessions() -> usize {
    32
}
impl Default for Bounds {
    fn default() -> Self {
        Self {
            offset: 0,
            limit: default_items(),
            max_bytes: default_bytes(),
        }
    }
}
impl Bounds {
    pub fn validated(self) -> Result<Self, Error> {
        if self.offset > MAX_OFFSET || self.limit == 0 || self.max_bytes == 0 {
            return Err(Error::InvalidBounds);
        }
        Ok(Self {
            limit: self.limit.min(MAX_EXCERPT_ITEMS),
            max_bytes: self.max_bytes.min(MAX_EXCERPT_BYTES),
            ..self
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case", deny_unknown_fields)]
pub enum Operation {
    Sessions {
        #[serde(default)]
        offset: usize,
        #[serde(default = "default_sessions")]
        limit: usize,
        #[serde(default)]
        status: Option<AgentStatus>,
    },
    Conversation {
        target: Target,
        #[serde(default)]
        bounds: Bounds,
    },
    Terminal {
        target: Target,
        #[serde(default)]
        bounds: Bounds,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Source {
    pub target: Target,
    pub window: u64,
    pub host: String,
    pub workspace: String,
    pub label: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Session {
    pub source: Source,
    pub status: AgentStatus,
    pub conversation: bool,
    pub terminal: bool,
    pub snapshot_revision: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UnavailableReason {
    InvalidBounds,
    InvalidTarget,
    UnknownTarget,
    StaleTarget,
    AmbiguousTarget,
    Disconnected,
    NotObserved,
    Unsupported,
    Disabled,
    Capacity,
    WindowClosed,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "scope", rename_all = "snake_case")]
pub enum Coverage {
    Capacity,
    Connection {
        window: u64,
        host: String,
        /// None means a connected snapshot was observed; it does not prove full remote coverage.
        unavailable: Option<UnavailableReason>,
    },
    Runtime {
        name: String,
        unavailable: UnavailableReason,
    },
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExcerptKind {
    Conversation,
    Terminal,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "result", rename_all = "snake_case")]
pub enum Reply {
    Sessions {
        sessions: Vec<Session>,
        coverage: Vec<Coverage>,
        coverage_truncated: bool,
        next_offset: Option<usize>,
        /// App read time, not a daemon-side freshness timestamp.
        observed_at_ms: u64,
    },
    Excerpt {
        source: Box<Source>,
        kind: ExcerptKind,
        text: String,
        offset: usize,
        next_offset: Option<usize>,
        truncated: bool,
        /// App read time. Content comes only from the prepared connected projection.
        observed_at_ms: u64,
        snapshot_revision: u64,
    },
    Unavailable {
        target: Option<Target>,
        reason: UnavailableReason,
    },
}
