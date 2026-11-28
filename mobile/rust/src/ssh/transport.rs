// ABOUTME: Authenticates an SSH connection and opens a non-PTY exec channel.
// ABOUTME: The Herdr worker owns the socket and cancels the SSH pump through its guard.

use super::*;
use crate::MobileError;
use russh::{
    Channel, ChannelMsg, client,
    keys::{HashAlg, PublicKeyOrCertificate},
};
use std::sync::{OnceLock, Weak};
use std::{future::Future, sync::atomic::Ordering};
use tokio::runtime::Handle;
use tokio::{
    io::copy,
    time::{sleep, timeout},
};

pub(super) struct HostKey(Option<String>);

#[derive(Default)]
pub(crate) struct ConnectedSsh(OnceLock<(Handle, Weak<client::Handle<HostKey>>)>);

impl ConnectedSsh {
    pub(super) fn connect(&self, session: &Arc<client::Handle<HostKey>>) {
        let _ = self.0.set((Handle::current(), Arc::downgrade(session)));
    }

    pub(super) fn get(&self) -> Result<(&Handle, Arc<client::Handle<HostKey>>), MobileError> {
        let (runtime, handle) = self.0.get().ok_or(MobileError::Disconnected)?;
        Ok((runtime, handle.upgrade().ok_or(MobileError::Disconnected)?))
    }
}

impl From<russh::Error> for SshError {
    fn from(_: russh::Error) -> Self {
        // russh errors can include server-provided text. The FFI receives only a category.
        Self::Connection
    }
}

impl client::Handler for HostKey {
    type Error = SshError;

    async fn check_server_key(
        &mut self,
        key: &PublicKeyOrCertificate,
    ) -> Result<bool, Self::Error> {
        let fingerprint = key.public_key().fingerprint(HashAlg::Sha256).to_string();
        match self.0.as_ref() {
            Some(expected) if expected == &fingerprint => Ok(true),
            Some(_) => Err(SshError::ChangedHost { fingerprint }),
            None => Err(SshError::UnknownHost { fingerprint }),
        }
    }
}

const SETUP_LIMIT: Duration = Duration::from_secs(30);

pub(super) async fn bounded<T>(
    task: impl Future<Output = Result<T, SshError>>,
) -> Result<T, SshError> {
    timeout(SETUP_LIMIT, task)
        .await
        .map_err(|_| SshError::Timeout)?
}

pub(super) fn validate_host(host: &SshHost) -> Result<(), SshError> {
    if host.port == 0
        || host.hostname.is_empty()
        || host.hostname.len() > 255
        || host
            .hostname
            .contains(|c: char| c.is_whitespace() || c.is_control() || c == '/')
        || host.username.is_empty()
        || host.username.len() > 255
        || host.username.contains(char::is_control)
    {
        return Err(SshError::InvalidHost);
    }
    Ok(())
}

pub(super) async fn verified_transport(
    host: &SshHost,
) -> Result<client::Handle<HostKey>, SshError> {
    let config = Arc::new(client::Config {
        nodelay: true,
        keepalive_interval: Some(Duration::from_secs(30)),
        ..Default::default()
    });
    client::connect(
        config,
        (host.hostname.as_str(), host.port),
        HostKey(host.fingerprint.clone()),
    )
    .await
}

pub(super) async fn authenticate(
    host: &SshHost,
    auth: SshAuth,
) -> Result<client::Handle<HostKey>, SshError> {
    let mut session = verified_transport(host).await?;
    let authenticated = match auth {
        SshAuth::Password { password } => session
            .authenticate_password(&host.username, password)
            .await?
            .success(),
        SshAuth::SecureEnclave { public_key, signer } => {
            let mut key = enclave::SshSigner::new(&public_key, signer)?;
            let result = session
                .authenticate_publickey_with(&host.username, key.public_key(), None, &mut key)
                .await?;
            key.authenticated(result)
        }
    };
    if !authenticated {
        return Err(SshError::Authentication);
    }
    Ok(session)
}

pub(super) async fn exec(
    handle: &client::Handle<HostKey>,
    command: String,
) -> Result<Channel<client::Msg>, SshError> {
    let mut channel = handle.channel_open_session().await?;
    channel.exec(true, command).await?;
    loop {
        match channel.wait().await {
            Some(ChannelMsg::Success) => return Ok(channel),
            Some(ChannelMsg::WindowAdjusted { .. }) => continue,
            _ => return Err(SshError::Channel),
        }
    }
}

async fn cancelled(stop: &AtomicBool) {
    while !stop.load(Ordering::Acquire) {
        sleep(Duration::from_millis(25)).await;
    }
}

/// Dropping the sender wakes the pump even when neither stream can make progress.
#[derive(Debug)]
pub(crate) struct BridgeGuard {
    _cancel: tokio::sync::oneshot::Sender<()>,
}

pub(crate) fn open_bridge(
    host: SshHost,
    auth: SshAuth,
    command: String,
    stop: &AtomicBool,
    enrollment: Option<tokio::sync::oneshot::Receiver<EnrollmentRequest>>,
    terminals: Option<(Arc<PtyRoutes>, String)>,
) -> Result<(UnixStream, BridgeGuard), SshError> {
    validate_host(&host)?;
    let enrollment = enrollment.filter(|_| matches!(auth, SshAuth::Password { .. }));
    let runtime = runtime().map_err(|_| SshError::Connection)?;
    let result = runtime.block_on(async {
        tokio::select! {
            biased;
            _ = cancelled(stop) => Err(SshError::Cancelled),
            result = bounded(async {
                let session = authenticate(&host, auth).await?;
                let channel = exec(&session, command).await?;
                Ok((session, channel))
            }) => result,
        }
    });
    let (session, channel) = match result {
        Ok(connected) => connected,
        Err(error) => {
            // An OS DNS lookup might outlive cancellation.
            runtime.shutdown_background();
            return Err(error);
        }
    };
    let (worker, peer) = UnixStream::pair().map_err(|_| SshError::Connection)?;
    peer.set_nonblocking(true)
        .map_err(|_| SshError::Connection)?;
    let (cancel, detached) = tokio::sync::oneshot::channel();
    std::thread::Builder::new()
        .name("spaceport-ssh".into())
        .spawn(move || {
            runtime.block_on(async move {
                let Ok(mut peer) = tokio::net::UnixStream::from_std(peer) else {
                    return;
                };
                let (mut from_worker, mut to_worker) = peer.split();
                let (mut stdout, mut stdin) = tokio::io::split(channel.into_stream());
                let pump = async {
                    tokio::select! {
                        _ = detached => {},
                        _ = copy(&mut from_worker, &mut stdin) => {},
                        _ = copy(&mut stdout, &mut to_worker) => {},
                    }
                };
                tokio::pin!(pump);
                let session = Arc::new(session);
                let authorize = async {
                    if let Some(request) = enrollment
                        && let Ok(request) = request.await
                    {
                        request.run(&host, &session).await;
                    }
                };
                let attach = async {
                    if let Some((routes, named_session)) = terminals {
                        routes.serve(session.clone(), &named_session).await;
                    }
                    std::future::pending::<()>().await;
                };
                let extras = async {
                    tokio::join!(authorize, attach);
                };
                tokio::select! { biased; _ = &mut pump => {}, _ = extras => {} }
            });
            runtime.shutdown_background();
        })
        .map_err(|_| SshError::Connection)?;
    Ok((worker, BridgeGuard { _cancel: cancel }))
}
