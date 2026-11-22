// ABOUTME: Enrolls a phone key on an explicitly selected POSIX SSH account.
// ABOUTME: Checks the result through a separate pinned SSH session using only the key.

use super::{
    transport::{authenticate, bounded, exec, validate_host},
    *,
};
use russh::{ChannelMsg, client};
use tokio::sync::oneshot;

// Pass the public key on stdin. Neither its comment nor profile text enters shell syntax.
// Existing restrictions survive even when they prevent the fresh key login.
const INSTALL: &str = r#"/bin/sh -c '
set -eu
umask 077
cd "${HOME:?}"
IFS= read -r key
blob=${key#ecdsa-sha2-nistp256 }
[ "$blob" != "$key" ]
[ ! -L .ssh ]
mkdir -p .ssh
cd .ssh
[ ! -L authorized_keys ]
if [ -e authorized_keys ]; then
    [ -f authorized_keys ]
    if grep -Fq "$blob" authorized_keys; then
        exit 0
    else
        [ "$?" = 1 ]
    fi
fi
printf "\n%s\n" "$key" >> authorized_keys
'"#;

#[uniffi::export]
pub fn authorize_enclave_phone(
    host: SshHost,
    password: String,
    public_key: Vec<u8>,
    signer: Arc<dyn EnclaveSigner>,
) -> Result<(), SshError> {
    validate_host(&host)?;
    enclave_public_key(public_key.clone())?;

    let runtime = runtime().map_err(|_| SshError::Connection)?;
    let result = runtime.block_on(async {
        let password_session = bounded(authenticate(&host, SshAuth::Password { password })).await?;
        install_and_verify(&host, &password_session, public_key, signer).await
    });
    runtime.shutdown_background();
    result
}

pub(crate) struct EnrollmentRequest {
    public_key: Vec<u8>,
    signer: Arc<dyn EnclaveSigner>,
    reply: oneshot::Sender<Result<(), SshError>>,
}

impl EnrollmentRequest {
    pub(crate) fn submit(
        sender: oneshot::Sender<Self>,
        public_key: Vec<u8>,
        signer: Arc<dyn EnclaveSigner>,
    ) -> Result<(), SshError> {
        let (reply, response) = oneshot::channel();
        sender
            .send(Self {
                public_key,
                signer,
                reply,
            })
            .map_err(|_| SshError::Cancelled)?;
        response.blocking_recv().map_err(|_| SshError::Cancelled)?
    }

    pub(super) async fn run(self, host: &SshHost, session: &client::Handle<transport::HostKey>) {
        let result = install_and_verify(host, session, self.public_key, self.signer).await;
        let _ = self.reply.send(result);
    }
}

async fn install_and_verify(
    host: &SshHost,
    password_session: &client::Handle<transport::HostKey>,
    public_key: Vec<u8>,
    signer: Arc<dyn EnclaveSigner>,
) -> Result<(), SshError> {
    let public = enclave_public_key(public_key.clone())?;
    let mut channel = bounded(exec(password_session, INSTALL.into())).await?;
    // An interrupted write may already have authorized the key. Verify without repeating it.
    let install = bounded(async {
        channel.data_bytes(format!("{public}\n")).await?;
        channel.eof().await?;
        loop {
            match channel.wait().await {
                Some(ChannelMsg::ExitStatus { exit_status: 0 }) => return Ok(()),
                Some(ChannelMsg::ExitStatus { .. } | ChannelMsg::Failure) => {
                    return Err(SshError::Enrollment);
                }
                Some(ChannelMsg::Close) | None => return Err(SshError::UnverifiedEnrollment),
                _ => (),
            }
        }
    })
    .await;
    drop(channel);
    if matches!(install, Err(SshError::Enrollment)) {
        return install;
    }
    match bounded(authenticate(
        host,
        SshAuth::SecureEnclave { public_key, signer },
    ))
    .await
    {
        Ok(_) => Ok(()),
        Err(trust @ (SshError::UnknownHost { .. } | SshError::ChangedHost { .. })) => Err(trust),
        Err(_) => Err(SshError::UnverifiedEnrollment),
    }
}
