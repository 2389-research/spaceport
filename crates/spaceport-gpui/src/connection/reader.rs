// ABOUTME: Drains ordered connection events into their owning mailboxes.
// ABOUTME: Publishes terminal reader state before allowing connection retirement.
use super::*;
use crate::i18n::fl;

fn disconnected() -> ClientEvent {
    ClientEvent::Disconnected {
        reason: fl!("endpoint-connection-lost"),
        ssh: None,
    }
}

/// Poisoned state may contain a partially applied event. Retire it completely.
pub(super) fn recover(inbox: &Mutex<LiveState>, state: &mut LiveState) {
    state.cancel_sounds();
    state.sound_connection_cancel.store(true, Ordering::Release);
    // A panic must not turn an explicit shutdown into permission to restart.
    let mut failed = LiveState {
        server_stopped: state.server_stopped,
        sound_connection_cancel: state.sound_connection_cancel.clone(),
        ..LiveState::default()
    };
    failed.apply(disconnected(), &crate::state::labels());
    *state = failed;
    inbox.clear_poison();
}

pub(super) struct Reader {
    pub(super) drained: Arc<AtomicBool>,
    pub(super) handle: ClientHandle,
    pub(super) inbox: Arc<Mutex<LiveState>>,
    pub(super) integrations: Arc<Mutex<IntegrationInbox>>,
    pub(super) links: Arc<Mutex<crate::links::LinkInbox>>,
    pub(super) scrollback: Arc<Mutex<crate::scrollback::Inbox>>,
    #[cfg(all(test, unix))]
    pub(super) wake: applied::Wake,
}

impl Reader {
    pub(super) fn read(self, events: impl IntoIterator<Item = ClientEvent>) {
        for event in events {
            if !self.receive(event) {
                break;
            }
            // The inbox guard is gone before a waiting test observes the wake.
            #[cfg(all(test, unix))]
            self.wake.notify();
        }
    }

    fn receive(&self, event: ClientEvent) -> bool {
        let event = self.filter(event);
        // The words a notice the reducer raises is written in, in the language
        // this process shows; `fl!` reads it from any thread.
        let labels = crate::state::labels();
        // Only this background reader waits on the inbox.
        let Ok(mut state) = self.inbox.lock() else {
            return false;
        };
        // Terminal state also fences queued events after poison recovery.
        if matches!(
            state.status,
            ConnectionStatus::Disconnected | ConnectionStatus::Detached
        ) {
            return false;
        }
        match event {
            Some(event) => state.apply(event, &labels),
            // Answer mailboxes still need to wake the window.
            None => state.dirty = true,
        }
        true
    }

    fn filter(&self, event: ClientEvent) -> Option<ClientEvent> {
        match &event {
            ClientEvent::Connected(_) => tracing::debug!("Connection bridge connected"),
            ClientEvent::Disconnected { .. } => tracing::debug!(
                category = "transport_disconnected",
                "Connection bridge disconnected"
            ),
            _ => {}
        }
        let event = match self.integrations.lock() {
            Ok(mut integrations) => integrations.apply(event),
            Err(_) => Some(event),
        };
        let event = event.and_then(|event| match self.scrollback.lock() {
            Ok(mut scrollback) => scrollback.apply(event),
            Err(_) => Some(event),
        });
        event.and_then(|event| match self.links.lock() {
            Ok(mut links) => links.apply(event),
            Err(_) => Some(event),
        })
    }
}

impl Drop for Reader {
    fn drop(&mut self) {
        self.handle.disconnect();
        // Revoke pending answers even if the UI already recovered this inbox.
        let event = self.filter(disconnected());
        let mut state = self.inbox.lock().unwrap_or_else(|error| error.into_inner());
        if self.inbox.is_poisoned() {
            recover(&self.inbox, &mut state);
        } else if !matches!(
            state.status,
            ConnectionStatus::Disconnected | ConnectionStatus::Detached
        ) && let Some(event) = event
        {
            state.apply(event, &crate::state::labels());
        }
        drop(state);
        // Retry may replace this inbox only after terminal state is readable.
        self.drained.store(true, Ordering::Release);
        #[cfg(all(test, unix))]
        self.wake.notify();
        tracing::debug!("Connection bridge event reader drained");
    }
}
