//! Spawning a connection worker. Setup, handshake, and teardown all happen off
//! the caller's thread; failures arrive as events rather than as a return value.

use crate::{
    ConnectTarget, Error, Result, catalog,
    discovery::session_socket,
    event::{ClientEvent, deliver},
    handle::{Client, ClientHandle, HandleInner},
    limits::{COMMAND_CAPACITY, EVENT_CAPACITY},
    options::ConnectOptions,
    queue,
    session::{Pace, Signals, run_connection},
    ssh,
    transport::Stream,
    wsl,
};
use crossbeam_channel::{Sender, bounded};
use std::{
    io,
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicU64, Ordering},
    },
    thread,
};

/// Owns terminal delivery even when setup or the session unwinds.
struct Disconnect {
    events: Sender<ClientEvent>,
    stop: Arc<AtomicBool>,
    event: Option<ClientEvent>,
}

impl Drop for Disconnect {
    fn drop(&mut self) {
        // Delivery remains ordered, bounded, and cancellable on this worker.
        if let Some(event) = self.event.take() {
            let _ = deliver(&self.events, event, &self.stop);
        }
        self.stop.store(true, Ordering::Release);
    }
}

/// Returns immediately after spawning. Connection/handshake errors arrive as events.
pub fn connect(target: ConnectTarget, options: ConnectOptions) -> Result<Client> {
    connect_with_surface_active(target, options, true)
}

/// Connect without changing `ConnectOptions` literals. Inactive connections require
/// negotiated surface interest and presentation-effect fencing. SSH additionally
/// requires endpoint health checks. All transport work runs off the caller thread.
pub fn connect_with_surface_active(
    target: ConnectTarget,
    options: ConnectOptions,
    surface_active: bool,
) -> Result<Client> {
    connect_with_connector(target, options, surface_active, |target, _| {
        let path = target
            .socket_path()
            .map_err(|error| io::Error::new(error.kind(), error))?;
        Stream::connect(path)
    })
}

/// Connect using application-specific setup on the I/O worker. SSH and WSL
/// targets always use the built-in remote bridge; local targets use
/// the connector. The connector should observe `stop` during waits so detach
/// cancels setup.
pub fn connect_with_connector(
    target: ConnectTarget,
    options: ConnectOptions,
    surface_active: bool,
    connector: impl FnOnce(&ConnectTarget, &AtomicBool) -> io::Result<Stream> + Send + 'static,
) -> Result<Client> {
    options.validate()?;
    match &target {
        ConnectTarget::Ssh { target, session } => {
            catalog::validate_target(target)?;
            session_socket(std::path::Path::new(""), session)?;
        }
        ConnectTarget::Wsl { distro, session } => wsl::validate(distro, session)?,
        _ => {}
    }
    let transport = match &target {
        ConnectTarget::Ssh { .. } => "ssh",
        ConnectTarget::Wsl { .. } => "wsl",
        _ => "local",
    };
    spawn_connection(
        options,
        surface_active,
        transport,
        target.is_remote(),
        move |stop| {
            Ok(match &target {
                ConnectTarget::Ssh { target, session } => {
                    let connected = ssh::connect(target, session, stop)?;
                    if connected.host_keys == ssh::HostKeys::Skip {
                        // The connection is up, but anyone intercepting it can
                        // impersonate the host the user exempted.
                        tracing::warn!("connected without verifying the host key");
                    }
                    (
                        connected.stream,
                        Some(connected.child),
                        Some(connected.host_keys),
                    )
                }
                ConnectTarget::Wsl { distro, session } => {
                    let (stream, child) = wsl::connect(distro, session, stop)?;
                    (stream, Some(child), None)
                }
                _ => (connector(&target, stop)?, None, None),
            })
        },
    )
}

/// Attach over an authenticated SSH exec channel supplied by an embedded client.
/// The connector runs on the worker and returns the socket carrying the command's
/// standard streams and a guard whose drop cancels the SSH transport.
/// Discovery, bridge negotiation, health checks and protocol framing match desktop.
#[cfg(unix)]
pub fn connect_embedded_bridge<G>(
    session: &str,
    options: ConnectOptions,
    connector: impl FnOnce(&str, &AtomicBool) -> Result<(Stream, G)> + Send + 'static,
) -> Result<Client> {
    connect_embedded_bridge_with_path(session, options, connector, |_| {})
}

/// Reports the accepted POSIX executable for follow-up channels on the same connection.
#[cfg(unix)]
pub fn connect_embedded_bridge_with_path<G>(
    session: &str,
    options: ConnectOptions,
    connector: impl FnOnce(&str, &AtomicBool) -> Result<(Stream, G)> + Send + 'static,
    accepted_path: impl FnOnce(String) + Send + 'static,
) -> Result<Client> {
    let command = ssh::bridge_command(session);
    embedded(
        session,
        options,
        true,
        connector,
        command,
        move |stream, stop| {
            if let Some(path) = ssh::handshake_with_path(stream, stop)? {
                accepted_path(path);
            }
            Ok(())
        },
    )
}

/// Read snapshots from a running named session over a surface-inactive endpoint.
/// The socket-only bridge fails if the session stops during discovery.
#[cfg(unix)]
pub fn observe_embedded_bridge<G>(
    session: &str,
    connector: impl FnOnce(&str, &AtomicBool) -> Result<(Stream, G)> + Send + 'static,
) -> Result<Client> {
    let command = ssh::observe::command(session);
    let name = session.to_owned();
    embedded(
        session,
        ConnectOptions::default(),
        false,
        connector,
        command,
        move |stream, stop| ssh::observe::handshake(stream, stop, &name),
    )
}

#[cfg(unix)]
fn embedded<G>(
    session: &str,
    options: ConnectOptions,
    active: bool,
    connector: impl FnOnce(&str, &AtomicBool) -> Result<(Stream, G)> + Send + 'static,
    command: String,
    handshake: impl FnOnce(&mut Stream, &AtomicBool) -> Result<()> + Send + 'static,
) -> Result<Client> {
    options.validate()?;
    session_socket(std::path::Path::new(""), session)?;
    spawn_connection(options, active, "ssh", true, move |stop| {
        let (mut stream, guard) = connector(&command, stop)?;
        stream.set_read_timeout(Some(crate::limits::POLL))?;
        stream.set_write_timeout(Some(std::time::Duration::from_secs(1)))?;
        handshake(&mut stream, stop)?;
        Ok((stream, guard, None))
    })
}

/// A transport the worker opened, and the host-key choice its bridge was
/// started with when that choice was this crate's to make. `None` is a
/// transport that checks no host key here: a local socket, WSL, or a bridge an
/// embedded client supplied.
type Opened<G> = (Stream, G, Option<ssh::HostKeys>);

/// Told ahead of the handshake it belongs to, so a window can show the
/// connection as unverified only once that handshake is accepted.
fn report_host_keys(
    host_keys: Option<ssh::HostKeys>,
    tx: &Sender<ClientEvent>,
    stop: &AtomicBool,
) -> Result<()> {
    if host_keys != Some(ssh::HostKeys::Skip) {
        return Ok(());
    }
    deliver(tx, ClientEvent::HostKeysUnverified, stop)
}

fn spawn_connection<G>(
    options: ConnectOptions,
    surface_active: bool,
    transport: &'static str,
    remote: bool,
    open: impl FnOnce(&AtomicBool) -> Result<Opened<G>> + Send + 'static,
) -> Result<Client> {
    let (commands, rx) = queue::channel(COMMAND_CAPACITY)?;
    let (tx, events) = bounded(EVENT_CAPACITY);
    let stop = Arc::new(AtomicBool::new(false));
    let worker_stop = stop.clone();
    let liveness = Arc::new(AtomicBool::new(false));
    let worker_liveness = liveness.clone();
    let transport_lost = Arc::new(AtomicBool::new(false));
    let worker_transport_lost = transport_lost.clone();
    #[cfg_attr(
        target_os = "linux",
        allow(
            clippy::cognitive_complexity,
            reason = "2026-11-27 Linux tracing baseline: complexity 38"
        )
    )]
    thread::Builder::new()
        .name("herdr-client-io".into())
        .spawn(move || {
            let mut disconnect = Disconnect {
                events: tx,
                stop: worker_stop,
                event: Some(ClientEvent::Disconnected {
                    reason: "connection worker panicked".into(),
                    ssh: None,
                }),
            };
            let worker_stop = &disconnect.stop;
            let span = tracing::info_span!("connection", transport);
            let _entered = span.enter();
            tracing::info!(transport, "connection starting");
            let result = (|| {
                let (stream, _guard, host_keys) = open(worker_stop)?;
                report_host_keys(host_keys, &disconnect.events, worker_stop)?;
                run_connection(
                    stream,
                    options,
                    surface_active,
                    remote,
                    rx,
                    &disconnect.events,
                    Signals {
                        stop: worker_stop,
                        liveness: &worker_liveness,
                        pace: Pace::default(),
                    },
                )
                .inspect_err(|error| {
                    if error.is_transport_loss() && !worker_stop.load(Ordering::Acquire) {
                        worker_transport_lost.store(true, Ordering::Release);
                    }
                })
                // The child guard is dropped before delivering a disconnect event.
            })();
            if worker_stop.load(Ordering::Acquire) {
                tracing::debug!("connection cancelled");
            } else if let Err(error) = &result {
                tracing::warn!(kind = ?error.kind(), "connection ended with transport or protocol failure");
            } else {
                tracing::info!("connection ended");
            }
            if !worker_stop.load(Ordering::Acquire) {
                if let Some(mismatch) = result.as_ref().err().and_then(Error::version_mismatch) {
                    let _ = deliver(
                        &disconnect.events,
                        ClientEvent::VersionMismatch(mismatch),
                        worker_stop,
                    );
                }
                let ssh = match &result {
                    Err(Error::Ssh(ssh::Error::Refused(failure))) => Some(*failure),
                    _ => None,
                };
                let reason = result
                    .err()
                    .map(|e| {
                        e.to_string()
                            .chars()
                            .filter(|c| !c.is_control())
                            .take(1024)
                            .collect()
                    })
                    .unwrap_or_else(|| "server disconnected".into());
                disconnect.event = Some(ClientEvent::Disconnected { reason, ssh });
            }
        })?;
    Ok(Client {
        handle: ClientHandle {
            inner: Arc::new(HandleInner {
                commands,
                stop,
                next_request: AtomicU64::new(1),
                image_busy: Arc::new(AtomicBool::new(false)),
                last_queued_theme: Default::default(),
                liveness,
                transport_lost,
            }),
        },
        events,
    })
}

#[cfg(test)]
#[allow(clippy::unwrap_used)] // Test fixtures only.
mod tests;
