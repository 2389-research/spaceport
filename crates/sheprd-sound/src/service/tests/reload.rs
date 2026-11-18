// ABOUTME: Verifies asynchronous sound reloads retain the last valid settings.
// ABOUTME: Coordinates through the controlled loader and worker playback channels.
use super::*;
use crate::{Error, MAX_VOLUME, tests::event};
use sheprd_protocol::SemanticNotificationKind as Kind;

#[test]
fn worker_reloads_asynchronously_and_keeps_last_valid_config() {
    let (loads, loaded) = mpsc::sync_channel(4);
    let (played, sounds) = mpsc::sync_channel(4);
    let (applied, completed) = mpsc::sync_channel(4);
    loads.send(Ok(Settings::default())).unwrap();
    let service = Service::start(
        move || {
            let result = loaded.recv_timeout(Duration::from_secs(3)).unwrap();
            applied.send(()).unwrap();
            result
        },
        move |_, path, _, _, _| {
            played.send(path.map(Path::to_owned)).unwrap();
            Ok(())
        },
        MAX_VOLUME,
    );
    completed.recv_timeout(Duration::from_secs(3)).unwrap();
    // Distinct titles, since the worker plays one copy of a repeated event.
    send_event(&service, "initial");
    assert_eq!(sounds.recv_timeout(Duration::from_secs(3)).unwrap(), None);
    let dir = tempfile::tempdir().unwrap();
    let config = dir.path().join("config.toml");
    std::fs::write(&config, "[ui.sound]\npath = 'new.mp3'").unwrap();
    loads.send(Settings::load(&config)).unwrap();
    service.reload.store(true, Ordering::Release);
    completed.recv_timeout(Duration::from_secs(3)).unwrap();
    send_event(&service, "reloaded");
    assert_eq!(
        sounds.recv_timeout(Duration::from_secs(3)).unwrap(),
        Some(dir.path().join("new.mp3"))
    );
    // A failed load keeps what the worker already had.
    loads.send(Err(Error::ConfigSize)).unwrap();
    service.reload.store(true, Ordering::Release);
    completed.recv_timeout(Duration::from_secs(3)).unwrap();
    send_event(&service, "retained");
    assert_eq!(
        sounds.recv_timeout(Duration::from_secs(3)).unwrap(),
        Some(dir.path().join("new.mp3"))
    );
    drop(service);
    assert!(matches!(
        sounds.recv_timeout(Duration::from_secs(3)),
        Err(mpsc::RecvTimeoutError::Disconnected)
    ));
}

/// A notification titled `title`, which the worker plays once.
fn send_event(service: &Service, title: &str) {
    let mut notification = event(Kind::Custom);
    notification.title = title.into();
    notification.agent = None;
    notification.workspace_id = None;
    notification.tab_id = None;
    notification.pane_id = None;
    service
        .sender
        .as_ref()
        .unwrap()
        .send(Job {
            request: PlaybackRequest::Notification {
                boot: None,
                event: notification,
            },
            cancel: Arc::new(AtomicBool::new(false)),
            connection_cancel: Arc::new(AtomicBool::new(false)),
            queued: Instant::now(),
        })
        .unwrap();
}
