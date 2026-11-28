// ABOUTME: Opens one user-selected loopback service on an authenticated SSH transport.
// ABOUTME: Bounds concurrent browser sockets and closes them all when the preview stops.
use super::transport::ConnectedSsh;
use crate::MobileError;
use std::{
    net::{Ipv4Addr, TcpListener},
    sync::{Arc, Condvar, Mutex},
    time::Duration,
};
use tokio::{
    io::{AsyncWriteExt, copy_bidirectional},
    net,
    sync::watch,
    task::JoinSet,
    time::timeout,
};

#[derive(Default)]
struct Closed {
    done: Mutex<bool>,
    signal: Condvar,
}

struct Listening {
    stop: watch::Sender<bool>,
    closed: Arc<Closed>,
}

#[derive(Default)]
pub(crate) struct ServicePreview(Mutex<Option<Arc<Listening>>>);

impl ServicePreview {
    pub(crate) fn start(&self, transport: &ConnectedSsh, remote: u16) -> Result<u16, MobileError> {
        let mut active = self.0.lock().unwrap_or_else(|e| e.into_inner());
        if remote == 0 || active.is_some() {
            return Err(MobileError::ServicePreview);
        }
        let (runtime, ssh) = transport.get()?;
        let local =
            TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).map_err(|_| MobileError::ServicePreview)?;
        let port = local
            .local_addr()
            .map_err(|_| MobileError::ServicePreview)?
            .port();
        local
            .set_nonblocking(true)
            .map_err(|_| MobileError::ServicePreview)?;
        let listener = {
            let _entered = runtime.enter();
            net::TcpListener::from_std(local).map_err(|_| MobileError::ServicePreview)?
        };
        let (stop, mut stopping) = watch::channel(false);
        let closed = Arc::new(Closed::default());
        *active = Some(Arc::new(Listening {
            stop,
            closed: Arc::clone(&closed),
        }));
        runtime.spawn(async move {
            let mut clients = JoinSet::new();
            loop {
                tokio::select! {
                    biased;
                    _ = stopping.changed() => break,
                    accepted = listener.accept(), if clients.len() < 8 => {
                        let Ok((mut local, peer)) = accepted else { break };
                        let ssh = Arc::clone(&ssh);
                        clients.spawn(async move {
                            let open = ssh.channel_open_direct_tcpip("127.0.0.1", u32::from(remote), "127.0.0.1", u32::from(peer.port()));
                            if let Ok(Ok(channel)) = timeout(Duration::from_secs(5), open).await {
                                let _ = copy_bidirectional(&mut local, &mut channel.into_stream()).await;
                            } else {
                                let _ = local.write_all(b"HTTP/1.0 502 Bad Gateway\r\nContent-Type: text/plain\r\nConnection: close\r\n\r\nThe SSH host could not open this loopback service.").await;
                            }
                        });
                    }
                    _ = clients.join_next(), if !clients.is_empty() => {}
                }
            }
            drop(listener);
            clients.shutdown().await;
            *closed.done.lock().unwrap_or_else(|e| e.into_inner()) = true;
            closed.signal.notify_all();
        });
        Ok(port)
    }

    pub(crate) fn stop(&self) -> Result<(), MobileError> {
        let current = self.0.lock().unwrap_or_else(|e| e.into_inner()).clone();
        let Some(current) = current else {
            return Ok(());
        };
        current.stop.send_replace(true);
        let closed = &current.closed;
        let done = closed.done.lock().unwrap_or_else(|e| e.into_inner());
        let (done, _) = closed
            .signal
            .wait_timeout_while(done, Duration::from_secs(2), |done| !*done)
            .unwrap_or_else(|e| e.into_inner());
        if !*done {
            return Err(MobileError::ServicePreviewStopping);
        }
        let mut active = self.0.lock().unwrap_or_else(|e| e.into_inner());
        if active
            .as_ref()
            .is_some_and(|item| Arc::ptr_eq(item, &current))
        {
            *active = None;
        }
        Ok(())
    }

    pub(crate) fn cancel(&self) {
        if let Some(active) = self.0.lock().unwrap_or_else(|e| e.into_inner()).take() {
            active.stop.send_replace(true);
        }
    }
}
