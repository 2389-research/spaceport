// ABOUTME: Shares controlled channels for sound service tests.
// ABOUTME: Builds isolated service fixtures and exact queued notification jobs.
use super::*;

/// A worker whose playback a test reads from a channel, with the default one
/// second delay.
fn service() -> (Service, mpsc::Receiver<Job>) {
    let (sender, receiver) = mpsc::sync_channel(8);
    let service = Service {
        sender: Some(sender),
        settings: Arc::new(Mutex::new(Some(Arc::new(Settings::default())))),
        reload: Default::default(),
        stop: Default::default(),
        volume: Default::default(),
    };
    (service, receiver)
}

/// Hands the worker `event` as if a poll emitted it, stamped `queued`.
fn send(service: &Service, boot: Option<&str>, event: SemanticNotification, queued: Instant) {
    service
        .sender
        .as_ref()
        .unwrap()
        .send(Job {
            request: PlaybackRequest::Notification {
                boot: boot.map(str::to_owned),
                event,
            },
            cancel: Arc::new(AtomicBool::new(false)),
            connection_cancel: Arc::new(AtomicBool::new(false)),
            queued,
        })
        .unwrap();
}

mod mutes;
mod play;
mod reload;
mod repeats;
mod volume;
mod worker_lifecycle;
