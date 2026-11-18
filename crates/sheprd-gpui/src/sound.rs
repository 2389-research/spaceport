//! The app's one sound worker: it plays the notification sounds Herdr's
//! settings name, on this window's endpoint's terms.
//! Playback adapted from Herdr; see SOUND-NOTICE.md.
// i18n: migrated
mod error;
mod playback;
#[cfg(test)]
mod tests;

pub(crate) use error::Error;

use sheprd_client::protocol::SemanticNotification;
use sheprd_sound::{Live, MAX_PENDING, Policy, Settings};
use std::{
    collections::VecDeque,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, AtomicU8, Ordering},
        mpsc,
    },
    time::{Duration, Instant},
};

/// How close together two identical notifications from one daemon boot must be
/// to play once. Herdr sends each event to every attached client, so windows
/// that share a daemon all hand the same event to the one worker.
const REPEAT_WINDOW: Duration = Duration::from_secs(2);

/// Whether this process plays no sound at all, as `HERDR_DISABLE_SOUND` asks.
pub(crate) fn muted() -> bool {
    playback::muted()
}

enum PlaybackRequest {
    Notification {
        boot: Option<String>,
        event: SemanticNotification,
    },
    Preview,
    /// One sound now, from the file Herdr's settings name for it.
    Play(sheprd_client::protocol::SemanticNotificationSound),
}

/// Notifications the worker recently accepted, compared as complete values.
#[derive(Default)]
struct Recent(VecDeque<(Instant, Option<String>, SemanticNotification)>);

impl Recent {
    /// Whether this is the first time `event` from `boot` arrives within
    /// [`REPEAT_WINDOW`]. Windows poll in any order, so `at` may precede an
    /// entry recorded earlier.
    fn first(&mut self, boot: Option<&str>, event: &SemanticNotification, at: Instant) -> bool {
        let near = |then: Instant| at.max(then).duration_since(at.min(then)) < REPEAT_WINDOW;
        self.0.retain(|(then, ..)| near(*then));
        if self
            .0
            .iter()
            .any(|(_, seen, heard)| seen.as_deref() == boot && heard == event)
        {
            return false;
        }
        if self.0.len() == MAX_PENDING {
            self.0.pop_front();
        }
        self.0
            .push_back((at, boot.map(str::to_owned), event.clone()));
        true
    }
}

struct Job {
    request: PlaybackRequest,
    cancel: Arc<AtomicBool>,
    connection_cancel: Arc<AtomicBool>,
    queued: Instant,
}

#[derive(Default)]
pub(crate) struct Service {
    sender: Option<mpsc::SyncSender<Job>>,
    settings: Arc<Mutex<Option<Arc<Settings>>>>,
    reload: Arc<AtomicBool>,
    stop: Arc<AtomicBool>,
    /// Percent of each sound's own level, read by the worker per sound.
    volume: Arc<AtomicU8>,
}

impl Service {
    pub(crate) fn reload(&self) {
        self.reload.store(true, Ordering::Release);
    }

    /// Plays every later sound, previews included, at `percent` of its own
    /// level; 0 plays nothing. Values above 100 play at full level.
    pub(crate) fn set_volume(&self, percent: u8) {
        self.volume.store(percent, Ordering::Relaxed);
    }

    pub(crate) fn new() -> Self {
        // Test/native fixtures must never touch personal configuration or audio.
        if cfg!(test) || playback::muted() {
            return Self::default();
        }
        // Resolved once, at construction: a window must not change what the
        // worker reads when its process environment changes later.
        let path = crate::config::view_settings_path(sheprd_client::local_var);
        Self::start(move || Settings::load(&path), playback::play)
    }

    /// Starts the worker with an injectable playback backend, called serially and
    /// only on that worker with the volume percent current when the sound
    /// starts. `play` blocks until completion or error, observing
    /// boot/connection cancellation flags and service shutdown.
    /// It must release per-job audio/device resources before returning; no player
    /// or device objects cross this boundary. Return typed errors with their
    /// sources intact: the worker logs failures without replaying or stopping.
    fn start(
        mut load: impl FnMut() -> sheprd_sound::Result<Settings> + Send + 'static,
        mut play: impl FnMut(
            sheprd_client::protocol::SemanticNotificationSound,
            Option<&std::path::Path>,
            u8,
            &[&AtomicBool],
            &AtomicBool,
        ) -> crate::Result<()>
        + Send
        + 'static,
    ) -> Self {
        let mut service = Self::default();
        service.set_volume(crate::config::SoundConfig::default().volume);
        let (sender, receiver) = mpsc::sync_channel::<Job>(8);
        let settings = service.settings.clone();
        let volume = service.volume.clone();
        let reload = service.reload.clone();
        let stop = service.stop.clone();
        let result = std::thread::Builder::new()
            .name("herdr-sound".into())
            .spawn(move || {
                let mut current = Arc::new(Settings::default());
                let mut recent = Recent::default();
                reload.store(true, Ordering::Release);
                while !stop.load(Ordering::Acquire) {
                    let job = match receiver.recv_timeout(Duration::from_millis(50)) {
                        Ok(job) => Some(job),
                        Err(mpsc::RecvTimeoutError::Timeout) => None,
                        Err(mpsc::RecvTimeoutError::Disconnected) => break,
                    };
                    if reload.swap(false, Ordering::AcqRel) {
                        match load() {
                            Ok(value) => current = Arc::new(value),
                            Err(error) => {
                                tracing::warn!(%error, "Could not load local TUI sound settings; retaining settings");
                            }
                        }
                        if let Ok(mut value) = settings.lock() {
                            *value = Some(current.clone());
                        }
                    }
                    let Some(job) = job else {
                        continue;
                    };
                    if job.cancel.load(Ordering::Acquire)
                        || job.connection_cancel.load(Ordering::Acquire)
                        || job.queued.elapsed() >= Duration::from_secs(1)
                    {
                        continue;
                    }
                    let (sound, path) = match job.request {
                        PlaybackRequest::Notification { boot, event } => {
                            if !recent.first(boot.as_deref(), &event, job.queued)
                                || !current.sound.allows(event.agent.as_deref())
                            {
                                continue;
                            }
                            let Some(sound) = event.sound else {
                                continue;
                            };
                            (sound, current.path_for(sound))
                        }
                        PlaybackRequest::Preview => {
                            (sheprd_client::protocol::SemanticNotificationSound::Done, None)
                        }
                        PlaybackRequest::Play(sound) => (sound, current.path_for(sound)),
                    };
                    if let Err(error) = play(
                        sound,
                        path.as_deref(),
                        volume.load(Ordering::Relaxed),
                        &[&job.cancel, &job.connection_cancel],
                        &stop,
                    ) {
                        tracing::debug!(%error, "Sound not played");
                    }
                }
            });
        match result {
            Ok(_) => service.sender = Some(sender),
            Err(error) => tracing::warn!(%error, "Could not start sound worker"),
        }
        service
    }

    /// Explicit local test, independent of endpoint state and notification settings.
    pub(crate) fn preview(&self) {
        self.request(PlaybackRequest::Preview);
    }

    /// Plays `sound` now, from the file the local sound settings name for it
    /// or the built-in one, whatever the notification settings say. Muted
    /// like every sound when `HERDR_DISABLE_SOUND` is set. False when the
    /// worker is not running or already has sounds queued.
    pub(crate) fn play(&self, sound: sheprd_client::protocol::SemanticNotificationSound) -> bool {
        self.request(PlaybackRequest::Play(sound))
    }

    fn request(&self, request: PlaybackRequest) -> bool {
        self.sender.as_ref().is_some_and(|sender| {
            sender
                .try_send(Job {
                    request,
                    cancel: self.stop.clone(),
                    connection_cancel: self.stop.clone(),
                    queued: Instant::now(),
                })
                .is_ok()
        })
    }

    #[cfg(test)]
    pub(crate) fn recording() -> (
        Self,
        mpsc::Receiver<sheprd_client::protocol::SemanticNotificationSound>,
    ) {
        let (sender, receiver) = mpsc::sync_channel(8);
        let service = Self::start(
            || Ok(Settings::default()),
            move |sound, _, _, _, _| {
                let _ = sender.try_send(sound);
                Ok(())
            },
        );
        service.publish_settings();
        (service, receiver)
    }

    /// The settings the worker loads on its first tick, so a test does not wait
    /// for the worker's own timer before it can hand it a notification.
    #[cfg(test)]
    fn publish_settings(&self) {
        if let Ok(mut value) = self.settings.lock() {
            *value = Some(Arc::new(Settings::default()));
        }
    }

    /// A worker that reports the volume each sound would play at.
    #[cfg(test)]
    pub(crate) fn recording_volume() -> (Self, mpsc::Receiver<u8>) {
        let (sender, receiver) = mpsc::sync_channel(8);
        let service = Self::start(
            || Ok(Settings::default()),
            move |_, _, volume, _, _| {
                let _ = sender.try_send(volume);
                Ok(())
            },
        );
        service.publish_settings();
        (service, receiver)
    }

    /// Plays what `live`'s notifications ask for, as the policy delays and
    /// validates them. Notifications `silenced` says the user muted, on
    /// arrival or while they wait, never play.
    pub(crate) fn poll(
        &self,
        policy: &mut Policy,
        live: Live<'_>,
        focused: bool,
        now: Instant,
        silenced: impl Fn(&SemanticNotification) -> bool,
    ) {
        if std::mem::take(live.reload) {
            self.reload();
        }
        live.events.retain(|(_, event)| !silenced(event));
        policy.forget(&silenced);
        let settings = self
            .settings
            .try_lock()
            .ok()
            .and_then(|value| value.clone());
        let cancel = Arc::clone(live.cancel);
        let connection_cancel = Arc::clone(live.connection_cancel);
        let boot = live.snapshot.map(|snapshot| snapshot.boot_id.clone());
        policy.poll(
            live,
            settings.map(|s| s.ui_delay()),
            focused,
            now,
            |event| {
                if let Some(sender) = &self.sender {
                    let _ = sender.try_send(Job {
                        request: PlaybackRequest::Notification {
                            boot: boot.clone(),
                            event,
                        },
                        cancel: Arc::clone(&cancel),
                        connection_cancel: Arc::clone(&connection_cancel),
                        queued: now,
                    });
                }
            },
        );
    }
}

impl Drop for Service {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Release);
    }
}
