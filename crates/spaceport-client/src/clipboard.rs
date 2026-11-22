//! FIFO reservations with an optional image lease held through transport.

mod error;
pub use error::Error;

use crate::{Result, frame::ImageWriter, limits::COMMAND_TIMEOUT, protocol::*, queue::Waker};
use crossbeam_channel::{Receiver, Sender};
#[cfg(any(test, feature = "test-support"))]
use std::sync::{Condvar, Mutex, PoisonError};
use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::Instant,
};

/// A FIFO slot reserved before starting asynchronous clipboard/file work.
/// Drop to skip the slot. Call `complete` on a background executor, never the UI.
#[must_use]
pub struct ClipboardImageUpload {
    pub(crate) target: ClientClipboardImageTarget,
    pub(crate) sender: Sender<Vec<u8>>,
    /// Fields drop in declaration order: this wakes the worker only once
    /// `sender` is gone, so it sees the published bytes or the closed slot.
    pub(crate) _wake: WakeOnDrop,
    pub(crate) lease: Arc<ImageLease>,
    pub(crate) stop: Arc<AtomicBool>,
}

/// Wakes the connection worker when dropped.
pub(crate) struct WakeOnDrop(pub(crate) Waker);

impl Drop for WakeOnDrop {
    fn drop(&mut self) {
        self.0.wake();
    }
}

/// A latch a lifecycle test blocks on until the worker's image slot is gone.
/// Test-only: the app polls [`ClipboardImageCancellation::is_finished`] from the
/// UI thread and must never wait on a slot.
#[cfg(any(test, feature = "test-support"))]
#[derive(Default)]
pub(crate) struct Finish {
    done: Mutex<bool>,
    woke: Condvar,
}

#[cfg(any(test, feature = "test-support"))]
impl Finish {
    /// Marks the slot gone and wakes every waiter.
    fn finish(&self) {
        *self.done.lock().unwrap_or_else(PoisonError::into_inner) = true;
        self.woke.notify_all();
    }

    fn wait(&self) {
        let mut done = self.done.lock().unwrap_or_else(PoisonError::into_inner);
        while !*done {
            done = self.woke.wait(done).unwrap_or_else(PoisonError::into_inner);
        }
    }
}

/// Cancels only this upload. Keeping this handle does not retain the image lease.
#[derive(Clone)]
pub struct ClipboardImageCancellation {
    pub(crate) cancelled: Arc<AtomicBool>,
    pub(crate) finished: Arc<AtomicBool>,
    #[cfg(any(test, feature = "test-support"))]
    pub(crate) finish: Arc<Finish>,
    pub(crate) waker: Waker,
}

impl ClipboardImageCancellation {
    /// Nonblocking. If a frame has begun writing, cancellation closes the
    /// connection rather than leaving a truncated frame on a reusable stream.
    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::Release);
        self.waker.wake();
    }

    /// True only once the worker's slot is gone (sent, rejected, or cancelled).
    /// Publication and requesting cancellation do not finish the slot. This is
    /// not daemon acknowledgement; retain this handle until it returns true.
    pub fn is_finished(&self) -> bool {
        self.finished.load(Ordering::Acquire)
    }

    /// Blocks until the worker's slot is gone. Lifecycle tests only: the app
    /// polls [`Self::is_finished`] from the UI thread and must never wait here.
    #[cfg(any(test, feature = "test-support"))]
    pub fn wait_finished(&self) {
        self.finish.wait();
    }
}

impl ClipboardImageUpload {
    pub fn cancellation_handle(&self) -> ClipboardImageCancellation {
        ClipboardImageCancellation {
            cancelled: self.lease.cancelled.clone(),
            finished: self.lease.finished.clone(),
            #[cfg(any(test, feature = "test-support"))]
            finish: self.lease.finish.clone(),
            waker: self.lease.waker.clone(),
        }
    }

    /// Useful between bounded background reads. Also true after worker rejection.
    pub fn is_cancelled(&self) -> bool {
        self.stop.load(Ordering::Acquire)
            || self.lease.cancelled.load(Ordering::Acquire)
            || self.lease.reserved_at.elapsed() >= COMMAND_TIMEOUT
    }

    /// Validate and encode at most 16 MiB, then publish without waiting for the
    /// worker. Success means queued, not daemon acknowledgement. The GUI must
    /// validate its captured connection epoch, boot, and target before calling.
    pub fn complete(self, extension: &str, data: Vec<u8>) -> Result<()> {
        self.publish(|upload| {
            upload.lease.claim_image()?;
            Ok(encode_clipboard_image(
                upload.target.clone(),
                extension,
                data,
            )?)
        })
    }

    /// Publish one original semantic event in this FIFO slot when no image is
    /// available. Encode in the background after the same GUI validation as
    /// `complete`. No terminal key or paste bytes are synthesized.
    pub fn complete_input(self, event: ClientPaneInputEvent) -> Result<()> {
        self.publish(|upload| {
            let events = vec![event];
            let message = match &upload.target {
                ClientClipboardImageTarget::Pane(pane_id) => ClientMessage::ClientShellPaneInput {
                    pane_id: pane_id.clone(),
                    events,
                },
                ClientClipboardImageTarget::Popup(terminal_id) => {
                    ClientMessage::ClientShellPopupInput {
                        terminal_id: terminal_id.clone(),
                        events,
                    }
                }
                ClientClipboardImageTarget::DirectTerminal => {
                    return Err(Error::InputTarget.into());
                }
            };
            Ok(encode_message(&message, MAX_FRAME_SIZE)?)
        })
    }

    fn publish(self, encode: impl FnOnce(&Self) -> Result<Vec<u8>>) -> Result<()> {
        self.check_ready()?;
        let bytes = encode(&self)?;
        self.check_ready()?;
        self.sender
            .try_send(bytes)
            .map_err(|_| crate::Error::Disconnected)
    }

    fn check_ready(&self) -> Result<()> {
        if self.stop.load(Ordering::Acquire) {
            return Err(crate::Error::Disconnected);
        }
        if self.is_cancelled() {
            return Err(Error::Cancelled.into());
        }
        Ok(())
    }
}

pub(crate) struct ImageLease {
    pub(crate) busy: Arc<AtomicBool>,
    pub(crate) claimed: AtomicBool,
    pub(crate) cancelled: Arc<AtomicBool>,
    pub(crate) finished: Arc<AtomicBool>,
    /// Set when the worker's slot goes; see [`Finish`].
    #[cfg(any(test, feature = "test-support"))]
    pub(crate) finish: Arc<Finish>,
    pub(crate) reserved_at: Instant,
    /// Publication and cancellation reach the worker outside its queue.
    pub(crate) waker: Waker,
}

impl ImageLease {
    // Only the reservation's producer claims; the worker only holds/drops it.
    pub(crate) fn claim_image(&self) -> Result<()> {
        if !self.claimed.load(Ordering::Acquire) {
            self.busy
                .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
                .map_err(|_| Error::Busy)?;
            self.claimed.store(true, Ordering::Release);
        }
        Ok(())
    }
}

impl Drop for ImageLease {
    fn drop(&mut self) {
        if self.claimed.load(Ordering::Acquire) {
            self.busy.store(false, Ordering::Release);
        }
        // A connection torn down after its slot went must not leave a waiting
        // test parked on a latch nothing else would set.
        #[cfg(any(test, feature = "test-support"))]
        self.finish.finish();
    }
}

pub(crate) struct ImageSlot {
    pub(crate) receiver: Receiver<Vec<u8>>,
    pub(crate) lease: Arc<ImageLease>,
    pub(crate) writer: ImageWriter,
}

impl ImageSlot {
    /// When the worker must act on this slot unwoken: preparation expires, or
    /// a started write times out. `published` is whether its bytes arrived.
    pub(crate) fn deadline(&self, published: bool) -> Option<Instant> {
        if published {
            self.writer.deadline()
        } else {
            Some(self.lease.reserved_at + COMMAND_TIMEOUT)
        }
    }
}

impl Drop for ImageSlot {
    fn drop(&mut self) {
        self.lease.cancelled.store(true, Ordering::Release);
        self.lease.finished.store(true, Ordering::Release);
        #[cfg(any(test, feature = "test-support"))]
        self.lease.finish.finish();
    }
}
