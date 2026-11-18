//! Rodio playback of one sound, on the worker's thread.
//!
//! Playback adapted from Herdr; see SOUND-NOTICE.md.

use crate::{Error, Result};
use rodio::{Decoder, DeviceSinkBuilder, Player, Source};
use sheprd_protocol::SemanticNotificationSound as Sound;
#[cfg(unix)]
use std::os::unix::fs::OpenOptionsExt;
use std::{
    borrow::Cow,
    io::{Cursor, Read},
    path::Path,
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc,
    },
    time::{Duration, Instant},
};

/// The loudest volume; a percentage above it plays at full level.
pub const MAX_VOLUME: u8 = 100;

const MAX_DURATION: Duration = Duration::from_secs(15);
const MAX_FILE_BYTES: u64 = 16 * 1024 * 1024;
type SoundDecoder = Decoder<Cursor<Cow<'static, [u8]>>>;

/// Whether this process plays no sound at all, as `HERDR_DISABLE_SOUND` asks.
pub fn muted() -> bool {
    std::env::var_os("HERDR_DISABLE_SOUND").is_some() || std::env::var_os("NEXTEST").is_some()
}

/// Rodio implementation of [`crate::Service`]'s worker-only, serial backend
/// contract. Blocks until completion or error, checking endpoint `cancel` and
/// service `stop`. Player/device resources remain local and are released before
/// every return; typed errors retain their sources, and the service continues
/// without replay.
pub(crate) fn play(
    sound: Sound,
    custom: Option<&Path>,
    volume: u8,
    cancel: &[&AtomicBool],
    stop: &AtomicBool,
) -> Result<()> {
    if muted() || volume == 0 {
        return Ok(());
    }
    let deadline = Instant::now() + MAX_DURATION;
    check(deadline, cancel, stop, Instant::now())?;
    let source = decode(sound, custom)?;
    check(deadline, cancel, stop, Instant::now())?;

    // Worker-owned and scoped to this job: the next notification reselects the
    // default device, including after unplug/sleep. Failed jobs are never replayed.
    let (errors, receiver) = mpsc::sync_channel(1);
    let mut output = DeviceSinkBuilder::from_default_device()?
        .with_error_callback(move |error| {
            let _ = errors.try_send(error);
        })
        .open_sink_or_fallback()?;
    output.log_on_drop(false);
    check(deadline, cancel, stop, Instant::now())?;
    let player = Player::connect_new(output.mixer());
    player.set_volume(gain(volume));
    player.append(source.take_duration(MAX_DURATION));
    wait(
        &player,
        &receiver,
        deadline,
        cancel,
        stop,
        Instant::now,
        || {
            std::thread::sleep(Duration::from_millis(25));
        },
    )
}

/// The amplitude factor for a volume percent. Loudness follows amplitude
/// roughly as its square, so a linear factor would leave the lower half of
/// the range barely quieter than full: 50% plays at a quarter, about -12 dB.
fn gain(percent: u8) -> f32 {
    let level = f32::from(percent.min(MAX_VOLUME)) / 100.;
    level * level
}

fn decode(sound: Sound, custom: Option<&Path>) -> Result<SoundDecoder> {
    if let Some(path) = custom {
        match decode_file(path) {
            Ok(source) => return Ok(source),
            Err(error) => tracing::debug!(%error, "Custom sound failed; using built-in sound"),
        }
    }
    let bytes: &'static [u8] = match sound {
        Sound::Done => include_bytes!("../../../assets/sounds/done.mp3"),
        Sound::Request => include_bytes!("../../../assets/sounds/request.mp3"),
    };
    Ok(Decoder::try_from(Cursor::new(Cow::Borrowed(bytes)))?)
}

fn decode_file(path: &Path) -> Result<SoundDecoder> {
    let read = || -> Result<Vec<u8>> {
        let mut options = std::fs::OpenOptions::new();
        options.read(true);
        // Do not let a configured FIFO block the sole sound worker on open.
        #[cfg(unix)]
        options.custom_flags(rustix::fs::OFlags::NONBLOCK.bits() as i32);
        let file = options.open(path)?;
        let metadata = file.metadata()?;
        if !metadata.is_file() || metadata.len() > MAX_FILE_BYTES {
            return Err(Error::FileSize);
        }
        let mut bytes = Vec::new();
        file.take(MAX_FILE_BYTES + 1).read_to_end(&mut bytes)?;
        if bytes.len() as u64 > MAX_FILE_BYTES {
            return Err(Error::FileSize);
        }
        Ok(bytes)
    };
    let bytes = read().map_err(|error| match error {
        Error::Io(source) => Error::File {
            path: path.into(),
            source,
        },
        error => error,
    })?;
    Ok(Decoder::try_from(Cursor::new(Cow::Owned(bytes)))?)
}

fn check(deadline: Instant, cancel: &[&AtomicBool], stop: &AtomicBool, now: Instant) -> Result<()> {
    if cancel.iter().any(|flag| flag.load(Ordering::Acquire)) || stop.load(Ordering::Acquire) {
        return Err(Error::Cancelled);
    }
    if now >= deadline {
        return Err(Error::Timeout);
    }
    Ok(())
}

fn wait(
    player: &Player,
    errors: &mpsc::Receiver<rodio::cpal::StreamError>,
    deadline: Instant,
    cancel: &[&AtomicBool],
    stop: &AtomicBool,
    mut now: impl FnMut() -> Instant,
    mut sleep: impl FnMut(),
) -> Result<()> {
    let result = loop {
        if let Err(error) = check(deadline, cancel, stop, now()) {
            break Err(error);
        }
        if let Ok(error) = errors.try_recv() {
            break Err(Error::Stream(error));
        }
        if player.empty() {
            break Ok(());
        }
        sleep();
    };
    // Stop also on errors; dropping the enclosing output discards buffered audio.
    player.stop();
    result
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests;
