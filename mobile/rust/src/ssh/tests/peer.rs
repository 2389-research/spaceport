// ABOUTME: Provides a loopback SSH account with an ephemeral host key.
// ABOUTME: Echoes only the fixture's test command and counts authenticated calls.

use super::*;
use russh::{
    Channel, ChannelId,
    keys::{Algorithm, HashAlg, PrivateKey},
    server,
};
use std::os::unix::fs::DirBuilderExt;
use std::sync::{Arc, atomic::AtomicUsize, mpsc};

type OpenChannel = (Channel<server::Msg>, server::ChannelOpenHandle);
type DeferredOpen = Arc<std::sync::Mutex<Option<mpsc::Sender<OpenChannel>>>>;
type Lookup = (ChannelId, server::Handle);
type DeferredLookup = Arc<std::sync::Mutex<Option<mpsc::Sender<Lookup>>>>;

#[derive(Debug)]
pub(super) enum ChannelEvent {
    AllocatedPty(ChannelId),
    Attached,
    Closed(ChannelId),
}

pub(super) struct Server {
    next_open: DeferredOpen,
    next_lookup: DeferredLookup,
    pub channel_events: mpsc::Receiver<ChannelEvent>,
    port: u16,
    pub home: std::path::PathBuf,
    pub key_logins: Arc<AtomicUsize>,
    pub fingerprint: String,
    pub attempts: Arc<AtomicUsize>,
    closed: mpsc::Receiver<()>,
    shutdown: Option<tokio::sync::oneshot::Sender<()>>,
    worker: Option<std::thread::JoinHandle<()>>,
}

impl Server {
    pub fn start() -> Self {
        Self::with_window_size(server::Config::default().window_size)
    }

    pub fn with_window_size(window_size: u32) -> Self {
        Self::with_config(server::Config {
            window_size,
            ..Default::default()
        })
    }

    pub fn with_config(config: server::Config) -> Self {
        let key = PrivateKey::random(&mut rand::rng(), Algorithm::Ed25519).unwrap();
        let fingerprint = key.public_key().fingerprint(HashAlg::Sha256).to_string();
        let (port_tx, port_rx) = mpsc::sync_channel(1);
        let (finished, closed) = mpsc::channel();
        let (shutdown, stop) = tokio::sync::oneshot::channel();
        let attempts = Arc::new(AtomicUsize::default());
        let home = std::env::temp_dir().join(format!(
            "spaceport-ssh-test-{:032x}",
            rand::random::<u128>()
        ));
        std::fs::DirBuilder::new()
            .mode(0o700)
            .create(&home)
            .unwrap();
        let key_logins = Arc::new(AtomicUsize::default());
        let next_open: DeferredOpen = Default::default();
        let next_lookup = DeferredLookup::default();
        let (events, channel_events) = mpsc::channel();
        let fixture = Peer {
            next_lookup: next_lookup.clone(),
            events,
            next_open: next_open.clone(),
            home: home.clone(),
            key_logins: key_logins.clone(),
            attempts: attempts.clone(),
            closed: finished,
            echo: false,
            enrollment: None,
            input: Vec::new(),
        };
        let worker = std::thread::spawn(move || {
            runtime().unwrap().block_on(async {
                let listener = tokio::net::TcpListener::bind(("127.0.0.1", 0)).await.unwrap();
                port_tx.send(listener.local_addr().unwrap().port()).unwrap();
                let config = Arc::new(server::Config {
                    keys: vec![key],
                    auth_rejection_time: Duration::ZERO,
                    auth_rejection_time_initial: Some(Duration::ZERO),
                    ..config
                });
                let mut stop = stop;
                loop {
                    tokio::select! {
                        _ = &mut stop => return,
                        result = listener.accept() => {
                            let (socket, _) = result.unwrap();
                            let config = config.clone();
                            let handler = fixture.clone();
                            tokio::spawn(async move {
                                if let Ok(session) = server::run_stream(config, socket, handler).await {
                                    let _ = session.await;
                                }
                            });
                        }
                    }
                }
            });
        });
        Self {
            port: port_rx.recv_timeout(Duration::from_secs(5)).unwrap(),
            fingerprint,
            next_open,
            next_lookup,
            channel_events,
            attempts,
            closed,
            key_logins,
            home,
            shutdown: Some(shutdown),
            worker: Some(worker),
        }
    }

    pub fn defer_next_open(&self) -> mpsc::Receiver<OpenChannel> {
        let (tx, rx) = mpsc::channel();
        *self.next_open.lock().unwrap() = Some(tx);
        rx
    }

    pub fn defer_next_lookup(&self) -> mpsc::Receiver<Lookup> {
        let (tx, rx) = mpsc::channel();
        *self.next_lookup.lock().unwrap() = Some(tx);
        rx
    }

    pub fn host(&self) -> SshHost {
        SshHost {
            hostname: "127.0.0.1".into(),
            port: self.port,
            username: "operator".into(),
            fingerprint: Some(self.fingerprint.clone()),
        }
    }

    pub fn password(&self) -> SshAuth {
        SshAuth::Password {
            password: "fixture".into(),
        }
    }

    pub fn wait_for_disconnect(&self) {
        self.closed.recv_timeout(Duration::from_secs(5)).unwrap();
    }
}

impl Drop for Server {
    fn drop(&mut self) {
        if let Some(stop) = self.shutdown.take() {
            let _ = stop.send(());
        }
        if let Some(worker) = self.worker.take() {
            worker.join().unwrap();
        }
        std::fs::remove_dir_all(&self.home).unwrap();
    }
}

#[derive(Clone)]
struct Peer {
    next_lookup: DeferredLookup,
    events: mpsc::Sender<ChannelEvent>,
    next_open: DeferredOpen,
    home: std::path::PathBuf,
    key_logins: Arc<AtomicUsize>,
    enrollment: Option<String>,
    input: Vec<u8>,
    attempts: Arc<AtomicUsize>,
    closed: mpsc::Sender<()>,
    echo: bool,
}

impl Drop for Peer {
    fn drop(&mut self) {
        if self.echo {
            let _ = self.closed.send(());
        }
    }
}

impl server::Handler for Peer {
    type Error = russh::Error;

    async fn auth_password(
        &mut self,
        user: &str,
        password: &str,
    ) -> Result<server::Auth, Self::Error> {
        self.attempts.fetch_add(1, Ordering::Relaxed);
        Ok(if user == "operator" && password == "fixture" {
            server::Auth::Accept
        } else {
            server::Auth::reject()
        })
    }

    async fn auth_publickey(
        &mut self,
        user: &str,
        key: server::VerifiedPublicKey<'_>,
    ) -> Result<server::Auth, Self::Error> {
        let authorized =
            std::fs::read_to_string(self.home.join(".ssh/authorized_keys")).unwrap_or_default();
        let presented = key.public_key.to_openssh().unwrap();
        if user == "operator" && authorized.lines().any(|line| line == presented) {
            self.key_logins.fetch_add(1, Ordering::Relaxed);
            Ok(server::Auth::Accept)
        } else {
            Ok(server::Auth::reject())
        }
    }

    async fn channel_open_session(
        &mut self,
        channel: Channel<server::Msg>,
        reply: server::ChannelOpenHandle,
        _: &mut server::Session,
    ) -> Result<(), Self::Error> {
        let barrier = self.next_open.lock().unwrap().take();
        if let Some(barrier) = barrier {
            let _ = barrier.send((channel, reply));
        } else {
            reply.accept().await;
        }
        Ok(())
    }

    async fn channel_close(
        &mut self,
        channel: ChannelId,
        _: &mut server::Session,
    ) -> Result<(), Self::Error> {
        let _ = self.events.send(ChannelEvent::Closed(channel));
        Ok(())
    }

    async fn pty_request(
        &mut self,
        channel: ChannelId,
        term: &str,
        cols: u32,
        rows: u32,
        _: u32,
        _: u32,
        _: &[(russh::Pty, u32)],
        session: &mut server::Session,
    ) -> Result<(), Self::Error> {
        if (term, cols, rows) == ("xterm-256color", 80, 24) {
            let _ = self.events.send(ChannelEvent::AllocatedPty(channel));
            session.channel_success(channel)
        } else {
            session.channel_failure(channel)
        }
    }

    async fn window_change_request(
        &mut self,
        channel: ChannelId,
        cols: u32,
        rows: u32,
        _: u32,
        _: u32,
        session: &mut server::Session,
    ) -> Result<(), Self::Error> {
        if (cols, rows) == (83, 25) {
            session.data(channel, b"resized".to_vec())?;
        }
        Ok(())
    }

    async fn exec_request(
        &mut self,
        channel: ChannelId,
        command: &[u8],
        session: &mut server::Session,
    ) -> Result<(), Self::Error> {
        if command == b"exec '/fixture/herdr' --session 'studio' pane get 'w1:p1'" {
            session.channel_success(channel)?;
            if let Some(reply) = self.next_lookup.lock().unwrap().take() {
                let _ = reply.send((channel, session.handle()));
                return Ok(());
            }
            session.data(
                channel,
                br#"{"result":{"pane":{"pane_id":"w1:p1","terminal_id":"term_123"}}}"#.to_vec(),
            )?;
            session.exit_status_request(channel, 0)?;
            session.eof(channel)?;
            return session.close(channel);
        }
        if command == b"exec '/fixture/herdr' --session 'studio' terminal attach 'term_123'" {
            session.channel_success(channel)?;
            let _ = self.events.send(ChannelEvent::Attached);
            return session.data(channel, b"attached".to_vec());
        }
        if let Ok(script) = std::str::from_utf8(command)
            && script.starts_with("/bin/sh -c ")
        {
            self.enrollment = Some(script.into());
            return session.channel_success(channel);
        }
        if command != b"cat" {
            return session.channel_failure(channel);
        }
        self.echo = true;
        session.channel_success(channel)?;
        session.extended_data(channel, 1, b"fixture stderr".to_vec())
    }

    async fn channel_eof(
        &mut self,
        channel: ChannelId,
        session: &mut server::Session,
    ) -> Result<(), Self::Error> {
        if let Some(command) = self.enrollment.take() {
            use std::process::Stdio;
            let mut child = spaceport_client::new_command("/bin/sh")
                .arg("-c")
                .arg(command)
                .env_clear()
                .env("HOME", &self.home)
                .env("PATH", "/bin:/usr/bin")
                .current_dir(&self.home)
                .stdin(Stdio::piped())
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn()
                .unwrap();
            child.stdin.take().unwrap().write_all(&self.input).unwrap();
            let status = child.wait().unwrap();
            session.exit_status_request(channel, status.code().unwrap_or(1) as u32)?;
            session.eof(channel)?;
            session.close(channel)?;
        }
        Ok(())
    }
    async fn data(
        &mut self,
        channel: ChannelId,
        data: &[u8],
        session: &mut server::Session,
    ) -> Result<(), Self::Error> {
        if self.enrollment.is_some() {
            self.input.extend_from_slice(data);
        }
        if self.echo {
            session.data(channel, data.to_vec())?;
        }
        Ok(())
    }
}
