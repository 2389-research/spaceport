//! A usable terminal connection can still fail the local socket's trust
//! checks. The classes and the socket they name are kept apart from transport
//! failure here, as `SshFailure` is, so the app can present the diagnosis
//! once the handshake agrees; the words it is presented in are the app's.

use std::path::{Path, PathBuf};

/// Which local endpoint check refused the connection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum UntrustedEndpoint {
    /// The peer is another user, or its credentials could not be read.
    #[error("The daemon's user could not be verified as your user.")]
    PeerUser,
    /// The standard session socket is missing or is not a socket (e.g. a symlink).
    #[error("The standard local endpoint is missing or is not a socket.")]
    NotSocket,
    /// The dialed socket is not the standard session socket.
    #[error("This is not the standard local session socket.")]
    OtherSocket,
    /// The socket is not owned by this user or is group/world-writable.
    #[error("The daemon socket has untrusted ownership or write permissions.")]
    SocketPermissions,
    /// The socket's directory is not owned by this user or is world-writable.
    #[error("The daemon socket directory must be owned by your user and not world-writable.")]
    DirectoryPermissions,
}

/// A connection that passed the transport but failed a trust check, with the
/// socket path the check refused.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LocalPeerWarning {
    reason: UntrustedEndpoint,
    socket: PathBuf,
}

impl LocalPeerWarning {
    pub fn new(reason: UntrustedEndpoint, socket: PathBuf) -> Self {
        Self { reason, socket }
    }

    pub fn reason(&self) -> UntrustedEndpoint {
        self.reason
    }

    pub fn socket(&self) -> &Path {
        &self.socket
    }
}
