use crate::{Method, frame, hello};
use std::{fmt, io, path::PathBuf, process::ExitStatus};

pub type Result<T, E = Error> = std::result::Result<T, E>;

/// Core's suggested next step, safe to show only through an explicit UI path.
/// Wire text is kept on one line and bounded before it enters shared errors.
#[derive(Clone, PartialEq, Eq)]
pub struct RefusalHint(String);

impl RefusalHint {
    const MAX_CHARS: usize = 512;

    /// Returns bounded display text; a hint is guidance, never a command to run.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub(crate) fn from_wire(value: String) -> Option<Self> {
        if value.is_empty() {
            return None;
        }
        let mut chars = value.chars();
        let mut safe: String = chars
            .by_ref()
            .take(Self::MAX_CHARS)
            .map(|ch| {
                if ch.is_control() || matches!(ch, '\u{2028}' | '\u{2029}') {
                    ' '
                } else {
                    ch
                }
            })
            .collect();
        if chars.next().is_some() {
            safe.pop();
            safe.push('…');
        }
        Some(Self(safe))
    }
}

impl fmt::Debug for RefusalHint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("RefusalHint([redacted])")
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    // keep-sorted start
    #[error(transparent)]
    Backend(#[from] sheprd_backend::Error),
    #[error("too many herdr-core requests are waiting for an answer")]
    Busy,
    #[error(transparent)]
    Compat(#[from] hello::Mismatch),
    #[error("cannot reach herdr-core at {}", path.display())]
    Connect {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error(transparent)]
    Frame(#[from] frame::Error),
    #[error("herdr-core sent an invalid {0} answer")]
    Invalid(Method, #[source] serde_json::Error),
    #[error("terminal attachment name must be a basename of at most 255 bytes")]
    InvalidPutName,
    #[error("herdr-core I/O failed")]
    Io(#[from] io::Error),
    #[error("invalid herdr-core message")]
    Json(#[from] serde_json::Error),
    #[error("the connection to herdr-core was lost")]
    Lost,
    #[error("no home folder to find herdr-core in")]
    NoHome,
    #[error("herdr-core broke the protocol: {0}")]
    Protocol(#[from] Violation),
    #[error("{size}-byte terminal attachment is over the {limit}-byte limit")]
    PutTooLarge { size: usize, limit: usize },
    /// `kind` and `message` are Core's words; `hint` is explicit UI guidance.
    #[error("herdr-core refused {method}: {message}")]
    Refused {
        method: Method,
        kind: Option<String>,
        message: String,
        hint: Option<RefusalHint>,
    },
    #[error("HC_SESSION {0:?} cannot be a folder name")]
    Session(String),
    #[error("cannot start herdr-core at {}", binary.display())]
    Start {
        binary: PathBuf,
        #[source]
        source: io::Error,
    },
    /// `stderr` is the start command's bounded output, for diagnostics.
    #[error("herdr-core did not start ({status})")]
    StartFailed { status: ExitStatus, stderr: String },
    #[error("herdr-core did not start within {0:?}")]
    StartTimeout(std::time::Duration),
    #[error("herdr-core did not answer {0} in time")]
    Timeout(Method),
    #[error("herdr-core does not support {0}")]
    Unsupported(Method),
    // keep-sorted end
}

/// How the core broke the protocol; the connection ends.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum Violation {
    // keep-sorted start
    #[error("a terminal stream sent an invalid resize frame")]
    InvalidResize,
    #[error("a hello reply longer than {0} bytes")]
    LongHello(usize),
    #[error("an event line longer than {0} bytes")]
    LongLine(usize),
    #[error("a stream sent past its credit")]
    PastCredit,
    #[error("a terminal stream sent too many resize frames without a read")]
    TooManyResizes,
    #[error("a stream sent a resize without negotiated notifications")]
    UnexpectedResize,
    #[error("the hello reply was not framed")]
    Unframed,
    // keep-sorted end
}
