use super::*;
use crate::{Error, MAX_VOLUME, tests::event};
use spaceport_protocol::SemanticNotificationKind as Kind;
use std::sync::atomic::AtomicUsize;

#[test]
fn worker_reloads_asynchronously_and_keeps_last_valid_config() {
    let (loads, loaded) = mpsc::sync_channel(4);
    let (played, sounds) = mpsc::sync_channel(4);
    let count = Arc::new(AtomicUsize::new(0));
    let worker_count = count.clone();
    loads.send(Ok(Settings::default())).unwrap();
    let service = Service::start(
        move || {
            let result = loaded.recv_timeout(Duration::from_secs(3)).unwrap();
            worker_count.fetch_add(1, Ordering::Release);
            result
        },
        move |_, path, _, _, _| {
            played.send(path.map(Path::to_owned)).unwrap();
            Ok(())
        },
        MAX_VOLUME,
    );
    let wait_loaded = |expected| {
        let deadline = Instant::now() + Duration::from_secs(3);
        while count.load(Ordering::Acquire) < expected {
            assert!(Instant::now() < deadline);
            std::thread::sleep(Duration::from_millis(1));
        }
    };
    wait_loaded(1);
    // Distinct titles, since the worker plays one copy of a repeated event.
    send_event(&service, "initial");
    assert_eq!(sounds.recv_timeout(Duration::from_secs(3)).unwrap(), None);
    let dir = tempfile::tempdir().unwrap();
    let config = dir.path().join("config.toml");
    std::fs::write(&config, "[ui.sound]\npath = 'new.mp3'").unwrap();
    loads.send(Settings::load(&config)).unwrap();
    service.reload.store(true, Ordering::Release);
    wait_loaded(2);
    send_event(&service, "reloaded");
    assert_eq!(
        sounds.recv_timeout(Duration::from_secs(3)).unwrap(),
        Some(dir.path().join("new.mp3"))
    );
    // A failed load keeps what the worker already had.
    loads.send(Err(Error::ConfigSize)).unwrap();
    service.reload.store(true, Ordering::Release);
    wait_loaded(3);
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
