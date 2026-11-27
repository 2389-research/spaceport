// ABOUTME: Reads one boot-fenced roster from an already-running session over pinned SSH.
// ABOUTME: Disconnects the inactive observer on completion, cancellation, and timeout.

use crate::{MobileError, MobileView, SshAuth, SshHost, projection::Projection, ssh::open_bridge};
use spaceport_client::{ClientEvent, ClientHandle, ConnectOptions};
use std::{
    io,
    sync::{Arc, Mutex, PoisonError},
    time::{Duration, Instant},
};

enum State {
    Pending,
    Reading(ClientHandle),
    Finished,
}

#[derive(uniffi::Object)]
pub struct MobileRosterRequest {
    state: Mutex<State>,
}

#[uniffi::export]
impl MobileRosterRequest {
    #[uniffi::constructor]
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            state: Mutex::new(State::Pending),
        })
    }

    pub fn cancel(&self) {
        let mut state = self.state.lock().unwrap_or_else(PoisonError::into_inner);
        if let State::Reading(handle) = std::mem::replace(&mut *state, State::Finished) {
            handle.disconnect();
        }
    }

    pub fn read(
        &self,
        host: SshHost,
        auth: SshAuth,
        session: String,
    ) -> Result<MobileView, MobileError> {
        let deadline = Instant::now() + Duration::from_secs(15);
        let client = {
            let mut state = self.state.lock().unwrap_or_else(PoisonError::into_inner);
            if !matches!(*state, State::Pending) {
                return Err(MobileError::Disconnected);
            }
            if host.fingerprint.as_ref().is_none_or(String::is_empty) {
                return Err(MobileError::Disconnected);
            }
            let client =
                spaceport_client::connect_embedded_observer(&session, move |command, stop| {
                    open_bridge(host, auth, command.to_owned(), stop, None, None)
                        .map_err(|error| spaceport_client::Error::from(io::Error::other(error)))
                })?;
            *state = State::Reading(client.handle.clone());
            client
        };
        let result = loop {
            match client.events.recv_deadline(deadline) {
                Ok(ClientEvent::Snapshot(snapshot)) if Instant::now() < deadline => {
                    let mut projection = Projection::new(ConnectOptions::default());
                    projection.snapshot(snapshot);
                    break Ok(projection.view());
                }
                Ok(ClientEvent::Disconnected { reason, .. }) => {
                    break Err(MobileError::Client { message: reason });
                }
                Ok(_) if Instant::now() < deadline => continue,
                Err(error) if error.is_disconnected() => break Err(MobileError::Disconnected),
                _ => break Err(MobileError::RosterTimeout),
            }
        };
        let mut state = self.state.lock().unwrap_or_else(PoisonError::into_inner);
        match std::mem::replace(&mut *state, State::Finished) {
            State::Reading(handle) => {
                handle.disconnect();
                result
            }
            _ => Err(MobileError::Disconnected),
        }
    }
}
