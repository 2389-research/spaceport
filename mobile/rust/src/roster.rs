// ABOUTME: Observes a saved Herdr session over authenticated SSH with an inactive terminal surface.
// ABOUTME: Publishes the latest agent roster and its observation time to the phone's Home screen.
use crate::{
    MobileError, MobileView, SshAuth, SshHost,
    controller::{connection_error, lock},
    projection::Projection,
    ssh::open_bridge,
};
use spaceport_client::{ClientEvent, ClientHandle, ConnectOptions};
use std::{
    sync::{Arc, Mutex},
    thread,
};

#[derive(uniffi::Object)]
pub struct MobileRoster {
    handle: ClientHandle,
    state: Arc<Mutex<Projection>>,
}

impl Drop for MobileRoster {
    fn drop(&mut self) {
        self.handle.disconnect();
    }
}

#[uniffi::export]
impl MobileRoster {
    /// Receive snapshots on the saved SSH account. The observer never claims terminal focus or size.
    #[uniffi::constructor]
    pub fn open(host: SshHost, auth: SshAuth, session: String) -> Result<Arc<Self>, MobileError> {
        let options = ConnectOptions::default();
        let state = Arc::new(Mutex::new(Projection::new(options)));
        let connecting = state.clone();
        let client = spaceport_client::observe_embedded_bridge(&session, move |command, stop| {
            open_bridge(host, auth, command.to_owned(), stop, None, None)
                .map_err(|error| connection_error(&connecting, error))
        })?;
        let roster = Arc::new(Self {
            handle: client.handle.clone(),
            state: state.clone(),
        });
        thread::Builder::new()
            .name("spaceport-mobile-roster".into())
            .spawn(move || collect(client.events, &state))
            .map_err(|error| MobileError::Client {
                message: error.to_string(),
            })?;
        Ok(roster)
    }

    pub fn poll(&self, after: u64) -> Option<MobileView> {
        let state = {
            let state = lock(&self.state);
            if state.version == after {
                return None;
            }
            state.clone()
        };
        Some(state.view())
    }

    pub fn disconnect(&self) {
        self.handle.disconnect();
        let mut state = lock(&self.state);
        if !state.disconnected {
            state.disconnected = true;
            state.version += 1;
        }
    }
}

fn collect(events: impl IntoIterator<Item = ClientEvent>, state: &Mutex<Projection>) {
    for event in events {
        let mut state = lock(state);
        if state.disconnected {
            return;
        }
        match event {
            ClientEvent::Snapshot(snapshot) => state.snapshot(snapshot),
            ClientEvent::Disconnected { reason, .. } => state.end(reason),
            _ => continue,
        }
        state.version += 1;
        if state.disconnected {
            return;
        }
    }
}

#[cfg(test)]
mod tests;
