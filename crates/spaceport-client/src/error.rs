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
    #[error("Could not talk to the daemon: {0}")]
    Io(#[from] io::Error),
    #[error("{0}")]
    Protocol(#[from] spaceport_protocol::Error),
    #[error("The daemon's answer was not valid JSON: {0}")]
    Json(#[from] serde_json::Error),
    // Schema details may contain secret-bearing unknown field names. Keep the
    // source for diagnostics, but never include it in presentation text.
    #[error("The saved host list cannot be read; update Spaceport, or remove it to start over.")]
    CatalogSchema(#[source] serde_json::Error),
    #[error("The saved host selection cannot be read; remove it so Spaceport can write a new one.")]
    SelectionSchema(#[source] serde_json::Error),
    #[error("Too many commands are waiting for the daemon; try again.")]
    Full,
    #[error("The app is not connected to the daemon.")]
    Disconnected,
    #[error(transparent)]
    Clipboard(#[from] crate::clipboard::Error),
    #[error("The daemon did not take the terminal input in time; reconnect.")]
    InputWriteTimeout,
    #[error("That command needs the daemon's session first; reconnect and try again.")]
    MissingBootId,
    #[error("The daemon's session changed before the command could run; reconnect and try again.")]
    CommandBoot,
    #[error("This Herdr daemon does not offer that request; run `herdr update` and reconnect.")]
    UnsupportedMethod,
    #[error(
        "This Herdr daemon cannot send pane screens the way this app asks; run `herdr update` and reconnect."
    )]
    UnsupportedSurfaceInterest,
    #[error("A pane screen was asked for with no rows or columns; nothing was sent.")]
    EmptySurface,
    #[error("The pane's screen is larger than the daemon allows; make the pane or window smaller.")]
    GeometryLimit,
    #[error("That is not a valid session name.")]
    InvalidSession,
    #[error("A host reached over SSH has no local socket to connect to.")]
    NoLocalSocket,
    #[error("The connection was stopped.")]
    Cancelled,
    #[error("The connection's events stopped.")]
    EventReceiverDropped,
    #[error("The daemon closed the connection.")]
    SocketClosed,
    #[error("The daemon started a message and never finished it; reconnect.")]
    PartialFrameTimeout,
    #[error("The daemon sent a message this app cannot read (bad message prefix); reconnect.")]
    FramePrefix,
    #[error("The daemon sent a message this app cannot read (bad message length); reconnect.")]
    FrameLength,
    #[error("The host did not answer its health check in time; check the network and reconnect.")]
    HealthTimeout,
    #[error("The host did not finish connecting in time; check the network and reconnect.")]
    HandshakeTimeout,
    #[error("The daemon did not answer in time; the request was not sent again.")]
    RequestTimeout,
    #[error("The host did not greet this app as expected; run `herdr update` and reconnect.")]
    ExpectedWelcome,
    /// The daemon answered the endpoint hello with a pre-endpoint welcome.
    #[error(
        "This host's Herdr daemon is older than this app needs; run `herdr update` and reconnect."
    )]
    LegacyDaemon,
    /// A local daemon closed after the hello without any welcome, as every
    /// release before the endpoint protocol does.
    #[error(
        "The host's Herdr daemon closed the connection without answering; it is likely older than {}, which this app requires. Run `herdr update` and reconnect",
        crate::compat::MIN_HERDR_VERSION
    )]
    ClosedBeforeWelcome,
    #[error(
        "This app and the host's Herdr daemon (version {server_version}) speak different protocol versions ({ours} here, {generation} there); {advice}",
        ours = spaceport_protocol::endpoint::ENDPOINT_PROTOCOL_GENERATION,
        advice = crate::compat::generation_advice(*.generation)
    )]
    EndpointGeneration {
        generation: u32,
        server_version: String,
    },
    #[error(
        "The host's Herdr daemon sent a greeting this app cannot read; run `herdr update` and reconnect."
    )]
    WelcomeKind,
    #[error("The host's Herdr daemon refused the connection ({code}): {message}")]
    WelcomeRejected { code: String, message: String },
    #[error(
        "This app and the host's Herdr daemon cannot agree on a protocol; run `herdr update` and reconnect."
    )]
    IncompatibleCodecs,
    #[error(
        "The Herdr daemon on this host (version {server_version}) cannot send pane screens the way this app asks; run `herdr update` and reconnect"
    )]
    MissingSurfaceInterest { server_version: String },
    #[error(
        "The Herdr daemon on this SSH host (version {server_version}) has no health check this app can use; run `herdr update` there and reconnect"
    )]
    MissingHealthCheck { server_version: String },
    #[error("The daemon's session changed, or its screen data went backwards; reconnect.")]
    SnapshotIdentity,
    #[error("A pane screen arrived before the daemon's session data; reconnect.")]
    SurfaceBeforeSnapshot,
    #[error("A pane screen arrived for the wrong pane or an out-of-date revision; reconnect.")]
    SurfaceIdentity,
    #[error("A pane screen update arrived before the full screen it belongs to; reconnect.")]
    PatchBeforeBaseline,
    #[error("An encoded pane screen arrived before the full screen it belongs to; reconnect.")]
    EncodedSurfaceBeforeBaseline,
    #[error(
        "This Herdr daemon did not offer the screen encoding this app asked for; run `herdr update` and reconnect."
    )]
    SurfaceEncodingNotNegotiated,
    #[error("The daemon's session changed while the request was in flight; try again.")]
    ResponseBoot,
    #[error("The daemon's answer was larger than this app accepts.")]
    ResponseLimit,
    #[error("The daemon answered a request this app never sent; reconnect.")]
    UnsolicitedResponse,
    #[error("The daemon's answer did not match the request; reconnect.")]
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
        "The Herdr daemon{} on this host cannot serve this app; {}",
        crate::compat::version_note(.version.as_deref()),
        crate::compat::generation_advice(.generation.unwrap_or(0))
    )]
    BridgeIncompatible {
        generation: Option<u32>,
        version: Option<String>,
    },
    #[error("The command on the host failed ({0}).")]
    RemoteCommand(std::process::ExitStatus),
    #[error("The command on the host printed something this app did not expect.")]
    RemoteOutput,
    #[error("SSH file transfer needs this app running on Linux or macOS.")]
    UploadUnsupported,
    #[error("SSH file transfer accepts at most 256 paths")]
    UploadPathLimit,
    #[error("file transfer requires a UTF-8 basename without controls")]
    UploadName,
    #[error("file transfer source is not a regular file")]
    UploadNotFile,
    #[error("The files are too large to transfer at once.")]
    UploadSizeOverflow,
    #[error("file transfer source changed length")]
    UploadSourceChanged,
    #[error("SSH file transfer cancelled")]
    UploadCancelled,
    #[error("SSH file transfer made no progress for 30 seconds")]
    UploadTimeout,
    #[error("invalid or excessive SSH file transfer response")]
    UploadResponse,
    #[error("Cleaning up the files left on the host needs the paths the upload returned.")]
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
    #[error("Host scripts need this app running on Linux or macOS.")]
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
    #[error("Could not start SSH for the port forward.")]
    ForwardSpawn(#[source] io::Error),
    #[error("No local port is free to forward to.")]
    ForwardLocalPort(#[source] io::Error),
    #[error("Could not prepare the port forward's private folder.")]
    ForwardControl(#[source] io::Error),
    #[error("SSH did not connect in time; check host trust and authentication.")]
    ForwardTimeout,
    #[error("SSH could not listen on a local port ({0}).")]
    ForwardRefused(std::process::ExitStatus),
    /// Stderr is discarded, as for every SSH child: it can carry banners or secrets.
    #[error("The SSH port forward ended ({0}); check host trust, authentication, and the network.")]
    ForwardExit(std::process::ExitStatus),
    #[error(
        "The saved host selection is not a regular file; remove it so Spaceport can write a new one."
    )]
    SelectionNotFile,
    #[error("The saved host selection is larger than Spaceport stores; remove it to start over.")]
    SelectionLimit,
    #[error("The saved host selection is in a format this app does not know; update Spaceport.")]
    SelectionVersion,
    #[error("The selected host is not there, or it is turned off; pick another host.")]
    SelectionUnavailable,
    #[error("Spaceport's saved data has a path it cannot use.")]
    StoragePath,
    #[error("Spaceport's saved data is not a regular file.")]
    StorageDestinationNotFile,
    #[error(
        "The saved host list is not a regular file; remove it so Spaceport can write a new one."
    )]
    CatalogNotFile,
    #[error("The saved host list is larger than Spaceport stores; remove a host.")]
    CatalogLimit,
    #[error(
        "The saved host list is in a format this app does not know, or it lists too many hosts; update Spaceport or remove a host."
    )]
    CatalogVersionOrCount,
    #[error("A saved host has an ID this app cannot use, or two hosts share one; remove one.")]
    ProfileId,
    #[error("A saved host's label cannot be used.")]
    ProfileLabel,
    /// An endpoint answered a request with an error. The message is the
    /// daemon's own display text, empty when it sent none; act on `code`,
    /// never on the message.
    #[error("{}", if .message.is_empty() { "The daemon refused the request without saying why." } else { .message.as_str() })]
    Endpoint {
        code: crate::response::EndpointErrorCode,
        message: String,
    },
    #[error("The daemon's answer is not one this app can read.")]
    ResponseSchema(#[source] serde_json::Error),
    #[error("The daemon's answer carried neither a result nor a reason.")]
    ResponseMissingResult,
    #[error("The daemon answered a different request than the one sent.")]
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

    /// Only a lost stream or failed heartbeat qualifies for transport recovery.
    pub(crate) fn is_transport_loss(&self) -> bool {
        use io::ErrorKind::*;
        matches!(self, Self::SocketClosed | Self::HealthTimeout)
            || matches!(self, Self::Io(source) if matches!(source.kind(),
                BrokenPipe | ConnectionAborted | ConnectionReset | NotConnected | UnexpectedEof |
                NetworkDown | NetworkUnreachable | HostUnreachable | TimedOut))
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
