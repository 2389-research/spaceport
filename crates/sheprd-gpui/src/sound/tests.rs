#![allow(clippy::unwrap_used)]
use super::*;
use crate::{connection::ConnectionBridge, state::LiveState};
use sheprd_client::{
    ClientEvent, ConnectTarget,
    protocol::{
        AgentStatus, ClientShellSnapshot, SemanticNotificationKind as Kind,
        SemanticNotificationSound as Sound, ServerMessage,
    },
};

fn live(status: AgentStatus) -> LiveState {
    let mut snapshot: ClientShellSnapshot = serde_json::from_str(include_str!(
        "../../../sheprd-protocol/tests/fixtures/endpoint-snapshot-v1.json"
    ))
    .unwrap();
    snapshot.agents.truncate(1);
    snapshot.agents[0].pane_id = "pane".into();
    snapshot.agents[0].agent_status = status;
    snapshot.focused_tab_id = Some("tab".into());
    snapshot.focused_workspace_id = Some("workspace".into());
    let mut live = LiveState::default();
    live.apply(ClientEvent::Snapshot(Arc::new(snapshot)));
    live
}

fn event(kind: Kind) -> SemanticNotification {
    SemanticNotification {
        kind,
        title: "test".into(),
        body: None,
        sound: Some(Sound::Done),
        agent: Some("claude".into()),
        workspace_id: Some("workspace".into()),
        tab_id: Some("tab".into()),
        pane_id: Some("pane".into()),
        position: None,
    }
}

/// One poll of what `state` queued, as the window hands it to the worker.
fn poll(
    service: &Service,
    policy: &mut Policy,
    state: &mut LiveState,
    focused: bool,
    now: Instant,
    silenced: impl Fn(&SemanticNotification) -> bool,
) {
    let snapshot = state.latest_snapshot().cloned();
    service.poll(
        policy,
        Live {
            cancel: &state.sound_cancel,
            connection_cancel: &state.sound_connection_cancel,
            connected: state.status.is_connected(),
            reload: &mut state.reload_sound,
            events: &mut state.sound_events,
            snapshot: snapshot.as_deref(),
        },
        focused,
        now,
        silenced,
    );
}

/// One policy poll of what `state` queued, as the worker's own poll runs it,
/// without the service's muting.
fn poll_policy(
    policy: &mut Policy,
    state: &mut LiveState,
    delay: Option<u64>,
    focused: bool,
    now: Instant,
    emit: impl FnMut(SemanticNotification),
) {
    let snapshot = state.latest_snapshot().cloned();
    policy.poll(
        Live {
            cancel: &state.sound_cancel,
            connection_cancel: &state.sound_connection_cancel,
            connected: state.status.is_connected(),
            reload: &mut state.reload_sound,
            events: &mut state.sound_events,
            snapshot: snapshot.as_deref(),
        },
        delay,
        focused,
        now,
        emit,
    );
}

#[test]
fn bridge_moves_bounded_notifications_and_reload_once() {
    let bridge = ConnectionBridge::new(ConnectTarget::Socket("/unused-sound.sock".into()));
    {
        let mut state = bridge.inbox.lock().unwrap();
        state.status = crate::state::ConnectionStatus::Connected;
        for i in 0..MAX_PENDING + 5 {
            let mut event = event(Kind::Custom);
            event.title = i.to_string();
            state.apply(ClientEvent::Message(ServerMessage::SemanticNotification(
                event,
            )));
        }
        state.apply(ClientEvent::Message(ServerMessage::ReloadSoundConfig));
    }
    let first = bridge.take_update().unwrap();
    assert_eq!(first.sound_events.len(), MAX_PENDING);
    assert_eq!(first.sound_events[0].1.title, "5");
    // Toast coalescing must not consume the independent sound delivery queue.
    assert_eq!(first.notifications.len(), 1);
    assert_eq!(first.notifications[0].title, (MAX_PENDING + 4).to_string());
    assert!(first.reload_sound);
    assert!(bridge.take_update().is_none());
    bridge.inbox.lock().unwrap().set_outer_focus(true);
    let next = bridge.take_update().unwrap();
    assert!(next.sound_events.is_empty());
    assert!(next.notifications.is_empty());
    assert!(!next.reload_sound);
}

#[test]
fn disconnect_boot_and_detach_cancel_pending_and_queued_generations() {
    let now = Instant::now();
    for boot_change in [false, true] {
        let mut state = live(AgentStatus::Done);
        let token = state.sound_cancel.clone();
        state.sound_events.push_back((now, event(Kind::Finished)));
        let mut policy = Policy::default();
        poll_policy(&mut policy, &mut state, Some(1), false, now, |_| panic!());
        if boot_change {
            let mut snapshot = state.snapshot.clone().unwrap();
            Arc::make_mut(&mut snapshot).boot_id = "new-boot".into();
            state.apply(ClientEvent::Snapshot(snapshot));
        } else {
            state.apply(ClientEvent::Disconnected {
                reason: "test".into(),
                ssh: None,
            });
        }
        assert!(token.load(Ordering::Acquire));
        poll_policy(
            &mut policy,
            &mut state,
            Some(1),
            false,
            now + Duration::from_secs(1),
            |_| panic!("retired generation"),
        );
        assert_eq!(policy.waiting(), 0, "the retired generation left nothing");
    }
    let mut bridge = ConnectionBridge::new(ConnectTarget::Socket("/unused-sound.sock".into()));
    let old = bridge.inbox.clone();
    let token = old.lock().unwrap().sound_cancel.clone();
    bridge.detach(false);
    assert!(token.load(Ordering::Acquire));
    old.lock()
        .unwrap()
        .apply(ClientEvent::Message(ServerMessage::SemanticNotification(
            event(Kind::Custom),
        )));
    assert!(bridge.take_update().unwrap().sound_events.is_empty());
    let token = bridge.inbox.lock().unwrap().sound_cancel.clone();
    drop(bridge);
    assert!(token.load(Ordering::Acquire));
}

#[test]
fn service_nonblocking_queue_bound_reload_and_move_once() {
    let (sender, receiver) = mpsc::sync_channel(1);
    let service = Service {
        sender: Some(sender),
        settings: Arc::new(Mutex::new(Some(Arc::new(Settings::default())))),
        reload: Default::default(),
        stop: Default::default(),
        volume: Default::default(),
    };
    let mut state = live(AgentStatus::Idle);
    let now = Instant::now();
    state.reload_sound = true;
    for _ in 0..3 {
        state.sound_events.push_back((now, event(Kind::Custom)));
    }
    let mut policy = Policy::default();
    poll(&service, &mut policy, &mut state, false, now, |_| false);
    assert!(service.reload.load(Ordering::Acquire));
    assert!(!state.reload_sound);
    assert!(receiver.try_recv().is_ok());
    assert!(receiver.try_recv().is_err());
    poll(&service, &mut policy, &mut state, false, now, |_| false);
    assert!(receiver.try_recv().is_err());
}

#[test]
fn preview_uses_builtin_on_worker_despite_notification_mute() {
    let caller = std::thread::current().id();
    let (sender, played) = mpsc::sync_channel(8);
    let service = Service::start(
        || Ok(Settings::default()),
        move |sound, path, _, cancel, stop| {
            assert_ne!(std::thread::current().id(), caller);
            assert!(path.is_none());
            assert!(cancel.iter().all(|flag| !flag.load(Ordering::Acquire)));
            assert!(!stop.load(Ordering::Acquire));
            sender.send(sound).unwrap();
            Ok(())
        },
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
        sender: Some(sender),
        settings: Default::default(),
        reload: Default::default(),
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
    let suppressed = Service::new();
    suppressed.preview();
    assert!(suppressed.sender.is_none());
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

#[test]
fn worker_reloads_asynchronously_and_keeps_last_valid_config() {
    use std::sync::atomic::AtomicUsize;
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
            played.send(path.map(std::path::Path::to_owned)).unwrap();
            Ok(())
        },
    );
    let wait_loaded = |expected| {
        let deadline = Instant::now() + Duration::from_secs(3);
        while count.load(Ordering::Acquire) < expected {
            assert!(Instant::now() < deadline);
            std::thread::sleep(Duration::from_millis(1));
        }
    };
    // Distinct titles, since the worker plays one copy of a repeated event.
    let send = |title: &str| {
        service
            .sender
            .as_ref()
            .unwrap()
            .send(Job {
                request: PlaybackRequest::Notification {
                    boot: None,
                    event: SemanticNotification {
                        kind: Kind::Custom,
                        title: title.into(),
                        body: None,
                        sound: Some(Sound::Done),
                        agent: None,
                        workspace_id: None,
                        tab_id: None,
                        pane_id: None,
                        position: None,
                    },
                },
                cancel: Arc::new(AtomicBool::new(false)),
                connection_cancel: Arc::new(AtomicBool::new(false)),
                queued: Instant::now(),
            })
            .unwrap();
    };
    wait_loaded(1);
    send("initial");
    assert_eq!(sounds.recv_timeout(Duration::from_secs(3)).unwrap(), None);
    let dir = tempfile::tempdir().unwrap();
    let config = dir.path().join("config.toml");
    std::fs::write(&config, "[ui.sound]\npath = 'new.mp3'").unwrap();
    loads.send(Settings::load(&config)).unwrap();
    service.reload.store(true, Ordering::Release);
    wait_loaded(2);
    send("reloaded");
    assert_eq!(
        sounds.recv_timeout(Duration::from_secs(3)).unwrap(),
        Some(dir.path().join("new.mp3"))
    );
    loads.send(Err(sheprd_sound::Error::ConfigSize)).unwrap();
    service.reload.store(true, Ordering::Release);
    wait_loaded(3);
    send("retained");
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

mod mutes;
mod play;
mod repeats;
mod staged_metadata;
mod volume;
