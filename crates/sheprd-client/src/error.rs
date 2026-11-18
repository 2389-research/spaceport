// ABOUTME: Defines typed client failures and their plain-language English diagnostics.
// ABOUTME: Preserves error sources, operation context, and transport retry categories.
use std::{
    io,
    path::{Path, PathBuf},
};

pub type Result<T, E = Error> = std::result::Result<T, E>;

/// The storage step that failed. Replacement retains both source and destination paths.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StorageOperation {
    Open,
    Metadata,
    Read,
    Decode,
    Encode,
    Validate,
    CreateDirectory,
    Create,
    Write,
    Sync,
    Replace { destination: PathBuf },
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Paths are available to diagnostic callers but omitted from presentation text.
    #[error("{source}")]
    Storage {
        operation: StorageOperation,
        path: PathBuf,
        #[source]
        source: Box<Error>,
    },
    #[error("Could not communicate with Herdr: {0}")]
    Io(#[from] io::Error),
    #[error("{0}")]
    Protocol(#[from] sheprd_protocol::Error),
    #[error("Spaceport could not read Herdr's reply: {0}")]
    Json(#[from] serde_json::Error),
    // Schema details may contain secret-bearing unknown field names. Keep the
    // source for diagnostics, but never include it in presentation text.
    #[error("The saved connections file could not be read.")]
    CatalogSchema(#[source] serde_json::Error),
    #[error("The saved connection choice could not be read.")]
    SelectionSchema(#[source] serde_json::Error),
    #[error("Too many commands are waiting. Let them finish before trying again.")]
    Full,
    #[error("Spaceport is disconnected from Herdr.")]
    Disconnected,
    #[error(transparent)]
    Clipboard(#[from] crate::clipboard::Error),
    #[error(
        "Spaceport could not finish sending terminal input. Some input may have arrived. Choose Terminal → Reconnect."
    )]
    InputWriteTimeout,
    #[error("The connection is not ready for this command. Wait for it to connect.")]
    MissingBootId,
    #[error("This command does not match the current connection. Choose Terminal → Reconnect.")]
    CommandBoot,
    #[error("This version of Herdr does not support this action.")]
    UnsupportedMethod,
    #[error("This version of Herdr cannot safely share these terminal views.")]
    UnsupportedSurfaceInterest,
    #[error("The terminal view needs a width and height. Resize the window and try again.")]
    EmptySurface,
    #[error("This terminal size is larger than Herdr supports. Resize the window and try again.")]
    GeometryLimit,
    #[error("invalid session name")]
    InvalidSession,
    #[error("This remote connection has no local socket file.")]
    NoLocalSocket,
    #[error("The connection stopped.")]
    Cancelled,
    #[error("Spaceport stopped receiving connection updates.")]
    EventReceiverDropped,
    #[error("The connection to Herdr closed.")]
    SocketClosed,
    #[error("Herdr stopped partway through a terminal update. Choose Terminal → Reconnect.")]
    PartialFrameTimeout,
    #[error("Herdr sent terminal data Spaceport could not read. Choose Terminal → Reconnect.")]
    FramePrefix,
    #[error("Herdr sent terminal data Spaceport could not read. Choose Terminal → Reconnect.")]
    FrameLength,
    #[error("Herdr stopped answering connection checks. Choose Terminal → Reconnect.")]
    HealthTimeout,
    #[error("Herdr did not finish opening the connection in time. Choose Terminal → Reconnect.")]
    HandshakeTimeout,
    #[error(
        "Herdr did not confirm the request. It may have happened; check the result before trying again."
    )]
    RequestTimeout,
    #[error("Herdr did not send the expected connection information. Choose Terminal → Reconnect.")]
    ExpectedWelcome,
    /// The daemon answered the endpoint hello with a pre-endpoint welcome.
    #[error(
        "This Herdr version is too old for Spaceport. Run `herdr update`, then choose Terminal → Reconnect."
    )]
    LegacyDaemon,
    /// A local daemon closed after the hello without any welcome, as every
    /// release before the endpoint protocol does.
    #[error(
        "Herdr closed the connection without answering. It may be older than the required version {}. Run `herdr update`, then choose Terminal → Reconnect.",
        crate::compat::MIN_HERDR_VERSION
    )]
    ClosedBeforeWelcome,
    #[error(
        "{}", crate::compat::generation_message(*.generation, .server_version)
    )]
    EndpointGeneration {
        generation: u32,
        server_version: String,
    },
    #[error("Herdr sent unsupported connection information. Choose Terminal → Reconnect.")]
    WelcomeKind,
    #[error("{code}: {message}")]
    WelcomeRejected { code: String, message: String },
    #[error("This version of Herdr uses a terminal format Spaceport does not support.")]
    IncompatibleCodecs,
    #[error(
        "Herdr {server_version} cannot safely share these terminal views. Run `herdr update`, then choose Terminal → Reconnect."
    )]
    MissingSurfaceInterest { server_version: String },
    #[error(
        "Herdr {server_version} on this device cannot report connection health. Run `herdr update` there, then choose Terminal → Reconnect."
    )]
    MissingHealthCheck { server_version: String },
    #[error("Herdr restarted or sent older session state. Choose Terminal → Reconnect.")]
    SnapshotIdentity,
    #[error("Herdr sent a screen before its session information. Choose Terminal → Reconnect.")]
    SurfaceBeforeSnapshot,
    #[error(
        "Herdr's terminal screen does not match its session information. Choose Terminal → Reconnect."
    )]
    SurfaceIdentity,
    #[error("Herdr sent a screen update before the initial screen. Choose Terminal → Reconnect.")]
    PatchBeforeBaseline,
    #[error("Herdr sent a screen update before the initial screen. Choose Terminal → Reconnect.")]
    EncodedSurfaceBeforeBaseline,
    #[error(
        "Herdr sent a terminal format this connection did not agree to use. Choose Terminal → Reconnect."
    )]
    SurfaceEncodingNotNegotiated,
    #[error("Herdr's reply belongs to another connection. Choose Terminal → Reconnect.")]
    ResponseBoot,
    #[error("Herdr's reply was too large for Spaceport to read.")]
    ResponseLimit,
    #[error("Herdr sent a reply without a matching request. Choose Terminal → Reconnect.")]
    UnsolicitedResponse,
    #[error("Herdr's reply did not match the waiting request. Choose Terminal → Reconnect.")]
    ResponseId,
    #[error("{0}")]
    ServerShutdown(String),
    #[error(transparent)]
    Sessions(#[from] crate::sessions::Error),
    #[error(transparent)]
    Ssh(#[from] crate::ssh::Error),
    #[error(transparent)]
    TerminalAttach(#[from] crate::terminal_attach::Error),
    #[error(transparent)]
    Wsl(#[from] crate::wsl::Error),
    /// Every Herdr installed on the SSH host or WSL distribution was skipped
    /// as unable to serve this client. Fields describe the first one;
    /// `version` is bounded text.
    #[error(
        "{}", crate::compat::bridge_message(.generation.unwrap_or(0), .version.as_deref())
    )]
    BridgeIncompatible {
        generation: Option<u32>,
        version: Option<String>,
    },
    #[error("remote command failed ({0})")]
    RemoteCommand(std::process::ExitStatus),
    #[error("remote command returned unexpected output")]
    RemoteOutput,
    #[error("SSH file transfer requires a Linux or macOS client")]
    UploadUnsupported,
    #[error("SSH file transfer accepts at most 256 paths")]
    UploadPathLimit,
    #[error("file transfer requires a UTF-8 basename without controls")]
    UploadName,
    #[error("file transfer source is not a regular file")]
    UploadNotFile,
    #[error("file transfer total size exceeds u64")]
    UploadSizeOverflow,
    #[error("file transfer source changed length")]
    UploadSourceChanged,
    #[error("SSH file transfer cancelled")]
    UploadCancelled,
    #[error("SSH file transfer made no progress for 30 seconds")]
    UploadTimeout,
    #[error("invalid or excessive SSH file transfer response")]
    UploadResponse,
    #[error("cleanup requires unchanged absolute paths returned by upload_files")]
    UploadCleanupPath,
    #[error("SSH file transfer failed; check host trust, authentication, and remote storage")]
    UploadExit { status: std::process::ExitStatus },
    #[error("SSH file transfer I/O failed")]
    UploadIo(#[source] io::Error),
    #[error("{source}; remote temporary-file cleanup also failed")]
    UploadCleanup {
        #[source]
        source: Box<Error>,
        cleanup: Box<Error>,
    },
    #[error("host scripts require a Linux or macOS client")]
    ScriptUnsupported,
    #[error("could not start host script")]
    ScriptSpawn(#[source] io::Error),
    #[error("could not reach host for script: {0}")]
    ScriptConnect(#[source] io::Error),
    #[error("host script I/O failed")]
    ScriptIo(#[source] io::Error),
    #[error("could not read host script input")]
    ScriptInput(#[source] io::Error),
    #[error("could not write host script output")]
    ScriptOutput(#[source] io::Error),
    #[error("host script output exceeds limit")]
    ScriptOutputLimit,
    #[error("host script cancelled")]
    ScriptCancelled,
    #[error("host script made no progress before its deadline")]
    ScriptTimeout,
    #[error("host script worker panicked")]
    ScriptWorker,
    /// `stderr` is a bounded, control-free tail kept for diagnostics.
    #[error("host script failed ({status}): {stderr}")]
    ScriptExit {
        status: std::process::ExitStatus,
        stderr: String,
    },
    #[error("could not start SSH for the port forward")]
    ForwardSpawn(#[source] io::Error),
    #[error("no local port is free to forward to")]
    ForwardLocalPort(#[source] io::Error),
    #[error("could not create the port forward's private control directory")]
    ForwardControl(#[source] io::Error),
    #[error("SSH did not connect in time; check host trust and authentication")]
    ForwardTimeout,
    #[error("SSH could not listen on a local port ({0})")]
    ForwardRefused(std::process::ExitStatus),
    /// Stderr is discarded, as for every SSH child: it can carry banners or secrets.
    #[error("SSH port forward ended ({0}); check host trust, authentication, and the network")]
    ForwardExit(std::process::ExitStatus),
    #[error("The saved connection choice path is not a file.")]
    SelectionNotFile,
    #[error("The saved connection choice is too large.")]
    SelectionLimit,
    #[error("The saved connection choice uses an unsupported format.")]
    SelectionVersion,
    #[error("The selected connection is missing or disabled. Select another device.")]
    SelectionUnavailable,
    #[error("invalid storage path")]
    StoragePath,
    #[error("storage destination is not a regular file")]
    StorageDestinationNotFile,
    #[error("The saved connections path is not a file.")]
    CatalogNotFile,
    #[error("The saved connection list is too large.")]
    CatalogLimit,
    #[error("The saved connections use an unsupported format or contain too many entries.")]
    CatalogVersionOrCount,
    #[error("A saved connection has a missing, invalid, or repeated ID.")]
    ProfileId,
    #[error("A saved connection has an invalid name.")]
    ProfileLabel,
    /// An endpoint answered a request with an error. The message is the
    /// daemon's own display text, empty when it sent none; act on `code`,
    /// never on the message.
    #[error("{}", if .message.is_empty() { "Spaceport could not read Herdr's error explanation." } else { .message.as_str() })]
    Endpoint {
        code: crate::response::EndpointErrorCode,
        message: String,
    },
    #[error("Spaceport could not read Herdr's reply.")]
    ResponseSchema(#[source] serde_json::Error),
    #[error("Herdr's reply contained neither a result nor an explanation.")]
    ResponseMissingResult,
    #[error("Herdr replied to a different action. Choose Terminal → Reconnect.")]
    ResponseType,
}

impl Error {
    pub(crate) fn storage(
        operation: StorageOperation,
        path: &Path,
        source: impl Into<Error>,
    ) -> Self {
        Self::Storage {
            operation,
            path: path.to_owned(),
            source: Box::new(source.into()),
        }
    }

    /// Whether a host script's `ssh` was dropped before key exchange, as sshd
    /// drops connections past its `MaxStartups` while many are still
    /// authenticating. Nothing ran on the host, so the script may run again
    /// once the host has caught up. `ssh` exits 255 for its own failures, and
    /// OpenSSH before 8.0 printed `ssh_exchange_identification` instead.
    pub fn dropped_before_key_exchange(&self) -> bool {
        const MARKERS: [&str; 2] = [
            "kex_exchange_identification:",
            "ssh_exchange_identification:",
        ];
        matches!(
            self,
            Self::ScriptExit { status, stderr }
                if status.code() == Some(255)
                    && MARKERS.iter().any(|marker| stderr.contains(marker))
        )
    }

    /// Whether a host script failed for want of its host, not on it: the
    /// connection could not be made, went quiet past its deadline, or `ssh`
    /// itself failed, which it reports by exiting 255. What the host last
    /// answered may still hold, so a caller can keep it and ask again later.
    pub fn host_unreached(&self) -> bool {
        match self {
            Self::ScriptConnect(_) | Self::ScriptTimeout => true,
            Self::ScriptExit { status, .. } => status.code() == Some(255),
            _ => false,
        }
    }

    /// Preserve transport retry/cancellation categories, including the historical
    /// InvalidData category for session deadlines and validation failures.
    pub fn kind(&self) -> io::ErrorKind {
        match self {
            Self::Storage { source, .. } => source.kind(),
            Self::Io(error) | Self::ScriptConnect(error) => error.kind(),
            Self::UploadIo(error) => error.kind(),
            Self::UploadCleanup { source, .. } => source.kind(),
            Self::UploadUnsupported => io::ErrorKind::Unsupported,
            Self::UploadCancelled => io::ErrorKind::Interrupted,
            Self::UploadTimeout => io::ErrorKind::TimedOut,
            Self::UploadPathLimit | Self::UploadName | Self::UploadCleanupPath => {
                io::ErrorKind::InvalidInput
            }
            Self::Protocol(error) => error.kind(),
            Self::Json(error) => error.io_error_kind().unwrap_or(if error.is_eof() {
                io::ErrorKind::UnexpectedEof
            } else {
                io::ErrorKind::InvalidData
            }),
            Self::InvalidSession | Self::NoLocalSocket => io::ErrorKind::InvalidInput,
            Self::Clipboard(error) => error.kind(),
            Self::Cancelled => io::ErrorKind::Interrupted,
            Self::EventReceiverDropped | Self::Disconnected => io::ErrorKind::BrokenPipe,
            Self::SocketClosed | Self::ClosedBeforeWelcome => io::ErrorKind::UnexpectedEof,
            Self::Sessions(error) => error.kind(),
            Self::Ssh(error) => error.kind(),
            Self::Wsl(error) => error.kind(),
            Self::ForwardSpawn(error)
            | Self::ForwardLocalPort(error)
            | Self::ForwardControl(error) => error.kind(),
            Self::HealthTimeout | Self::ForwardTimeout => io::ErrorKind::TimedOut,
            Self::Full => io::ErrorKind::WouldBlock,
            _ => io::ErrorKind::InvalidData,
        }
    }
}
