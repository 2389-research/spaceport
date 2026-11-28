// ABOUTME: Connects the phone directly to the operator's SSH account.
// ABOUTME: Verifies the host before authenticating or executing Herdr's bridge.

use std::{
    os::unix::net::UnixStream,
    sync::{Arc, atomic::AtomicBool},
    time::Duration,
};

#[derive(Clone, uniffi::Record)]
pub struct SshHost {
    pub hostname: String,
    pub port: u16,
    pub username: String,
    pub fingerprint: Option<String>,
}

#[derive(uniffi::Enum)]
pub enum SshAuth {
    Password {
        password: String,
    },
    SecureEnclave {
        public_key: Vec<u8>,
        signer: Arc<dyn EnclaveSigner>,
    },
}

#[derive(Debug, thiserror::Error, uniffi::Error)]
pub enum SshError {
    #[error("Verify this host's key before connecting: {fingerprint}")]
    UnknownHost { fingerprint: String },
    #[error("Host key changed: {fingerprint}")]
    ChangedHost { fingerprint: String },
    #[error("SSH authentication failed.")]
    Authentication,
    #[error("SSH connection timed out.")]
    Timeout,
    #[error("SSH connection cancelled.")]
    Cancelled,
    #[error("Could not establish an SSH connection.")]
    Connection,
    #[error("The host refused the SSH command.")]
    Channel,
    #[error("The phone SSH key is invalid.")]
    Key,
    #[error("Could not authorize the phone's key.")]
    Enrollment,
    #[error("Could not verify the phone's key. It may already be authorized.")]
    UnverifiedEnrollment,
    #[error("Enter a host, port and SSH username.")]
    InvalidHost,
}

fn runtime() -> std::io::Result<tokio::runtime::Runtime> {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
}

mod images;
pub use images::{MobileFileTransfer, StagedImage};
mod pty;
pub use pty::MobilePty;
pub(crate) use pty::PtyRoutes;
mod enclave;
pub use enclave::{EnclaveSigner, enclave_public_key};
mod enroll;
pub(crate) use enroll::EnrollmentRequest;
pub use enroll::authorize_enclave_phone;
mod service_preview;
mod transport;
pub(crate) use transport::open_bridge;
#[cfg(test)]
mod tests;

/// Checks the negotiated server key without sending an SSH credential or opening a channel.
#[uniffi::export]
pub fn check_ssh_host(host: SshHost) -> Result<(), SshError> {
    transport::validate_host(&host)?;
    let runtime = runtime().map_err(|_| SshError::Connection)?;
    let result = runtime.block_on(transport::bounded(async {
        transport::verified_transport(&host).await?;
        Ok(())
    }));
    runtime.shutdown_background();
    result
}
