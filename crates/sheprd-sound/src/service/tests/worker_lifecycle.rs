// ABOUTME: Verifies bounded sound queues and worker shutdown.
// ABOUTME: Preserves preview, cancellation, reload, and backend-error contracts.
use super::*;
use crate::tests::{Endpoint, event};
use sheprd_protocol::{
    AgentStatus, SemanticNotificationKind as Kind, SemanticNotificationSound as Sound,
};

#[test]
fn service_nonblocking_queue_bound_reload_and_move_once() {
    let (sender, receiver) = mpsc::sync_channel(1);
    let service = Service {
        reload: Default::default(),
        sender: Some(sender),
        settings: Arc::new(Mutex::new(Some(Arc::new(Settings::default())))),
        stop: Default::default(),
        volume: Default::default(),
    };
    let mut state = Endpoint::new(AgentStatus::Idle);
    let now = Instant::now();
    state.reload = true;
    for _ in 0..3 {
        state.events.push_back((now, event(Kind::Custom)));
    }
    let mut policy = Policy::default();
    service.poll(&mut policy, state.live(), false, now, |_| false);
    assert!(service.reload.load(Ordering::Acquire));
    assert!(!state.reload);
    assert!(receiver.try_recv().is_ok());
    assert!(receiver.try_recv().is_err());
    service.poll(&mut policy, state.live(), false, now, |_| false);
    assert!(receiver.try_recv().is_err());
}

#[test]
fn preview_uses_builtin_on_worker_despite_notification_mute() {
    let caller = std::thread::current().id();
    let (sender, played) = mpsc::sync_channel(8);
    let service = Service::start(
        move || {
            assert_ne!(std::thread::current().id(), caller);
            let mut settings = Settings::default();
            settings.sound = toml::from_str(
                "enabled = false\ndone_path = '/unused-custom.mp3'\n[agents]\nclaude = 'off'",
            )
            .unwrap();
            Ok(settings)
        },
        move |sound, path, _, cancel, stop| {
            assert_ne!(std::thread::current().id(), caller);
            assert!(path.is_none());
            assert!(cancel.iter().all(|flag| !flag.load(Ordering::Acquire)));
            assert!(!stop.load(Ordering::Acquire));
            sender.send(sound).unwrap();
            Ok(())
        },
        crate::MAX_VOLUME,
    );
    service
        .sender
        .as_ref()
        .unwrap()
        .send(Job {
            request: PlaybackRequest::Notification {
                boot: None,
                event: event(Kind::Custom),
            },
            cancel: Arc::new(AtomicBool::new(false)),
            connection_cancel: Arc::new(AtomicBool::new(false)),
            queued: Instant::now(),
        })
        .unwrap();
    service.preview();
    assert_eq!(
        played.recv_timeout(Duration::from_secs(3)).unwrap(),
        Sound::Done
    );
    drop(service);
    assert!(matches!(
        played.recv_timeout(Duration::from_secs(3)),
        Err(mpsc::RecvTimeoutError::Disconnected)
    ));
}

#[test]
fn preview_queue_is_bounded_and_uses_service_cancellation() {
    let (sender, receiver) = mpsc::sync_channel(1);
    let service = Service {
        reload: Default::default(),
        sender: Some(sender),
        settings: Default::default(),
        stop: Default::default(),
        volume: Default::default(),
    };
    service.preview();
    service.preview();
    let job = receiver.try_recv().unwrap();
    assert!(matches!(job.request, PlaybackRequest::Preview));
    assert!(Arc::ptr_eq(&job.cancel, &service.stop));
    assert!(receiver.try_recv().is_err());
    drop(service);
    assert!(job.cancel.load(Ordering::Acquire));
    // Normal constructors in unit/headless tests must never open a real device.
    let suppressed = Service::new("/unused-sound-config.toml".into(), crate::MAX_VOLUME);
    assert!(suppressed.sender.is_none());
    suppressed.preview();
}

#[test]
fn worker_continues_after_backend_error_without_replaying() {
    let (sender, played) = mpsc::sync_channel(8);
    let mut first = true;
    let service = Service::start(
        || Ok(Settings::default()),
        move |sound, _, _, _, _| {
            sender.send(sound).unwrap();
            if std::mem::take(&mut first) {
                return Err(std::io::Error::from(std::io::ErrorKind::BrokenPipe).into());
            }
            Ok(())
        },
        crate::MAX_VOLUME,
    );
    for sound in [Sound::Done, Sound::Request] {
        let mut notification = event(Kind::Custom);
        notification.sound = Some(sound);
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
        assert_eq!(played.recv_timeout(Duration::from_secs(3)).unwrap(), sound);
    }
    drop(service);
    assert!(matches!(
        played.recv_timeout(Duration::from_secs(3)),
        Err(mpsc::RecvTimeoutError::Disconnected)
    ));
}

#[test]
fn worker_drops_cancelled_and_expired_jobs_without_playing_them() {
    let (service, played) = Service::recording();
    for (cancelled, age) in [(true, 0), (false, 2), (false, 0)] {
        service
            .sender
            .as_ref()
            .unwrap()
            .send(Job {
                request: PlaybackRequest::Notification {
                    boot: None,
                    event: event(Kind::Custom),
                },
                cancel: Arc::new(AtomicBool::new(cancelled)),
                connection_cancel: Arc::new(AtomicBool::new(false)),
                queued: Instant::now() - Duration::from_secs(age),
            })
            .unwrap();
    }
    assert_eq!(
        played.recv_timeout(Duration::from_secs(3)).unwrap(),
        Sound::Done
    );
    drop(service);
    assert!(matches!(
        played.recv_timeout(Duration::from_secs(3)),
        Err(mpsc::RecvTimeoutError::Disconnected)
    ));
}
