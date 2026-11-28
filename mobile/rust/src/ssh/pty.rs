// ABOUTME: Binds a direct SSH terminal channel to one confirmed mobile input target.
// ABOUTME: Serializes attachments and drops buffered input when its owner closes.
mod channel;
mod open;

use super::transport::HostKey;
use crate::{InputTarget, MobileError, projection::Projection};
use russh::client;
use std::sync::{
    Arc, Condvar, Mutex, MutexGuard, OnceLock, PoisonError,
    atomic::{AtomicBool, Ordering},
};
use tokio::sync::{Notify, mpsc, watch};

const MAX_WRITE: usize = 64 * 1024;
const PACKET: usize = 16 * 1024;

fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(PoisonError::into_inner)
}

fn check_size(cols: u16, rows: u16) -> Result<(), MobileError> {
    crate::controller::geometry(cols, rows, 1, 1).map(|_| ())
}

struct Binding {
    target: InputTarget,
    projection: Arc<Mutex<Projection>>,
    ready: AtomicBool,
    signal: (Mutex<()>, Condvar),
    error: Mutex<Option<MobileError>>,
}

impl Binding {
    fn wake_reader(&self) {
        let _guard = lock(&self.signal.0);
        self.signal.1.notify_one();
    }

    fn current(&self) -> Result<(), MobileError> {
        if lock(&self.projection).direct_target().as_ref() == Some(&self.target) {
            Ok(())
        } else {
            Err(MobileError::StaleView)
        }
    }
}

enum Command {
    Write(Vec<u8>),
    Resize(u16, u16),
}

#[derive(uniffi::Object)]
pub struct MobilePty {
    binding: Arc<Binding>,
    stop: watch::Sender<bool>,
    input: mpsc::Sender<Command>,
    output: Mutex<mpsc::Receiver<Vec<u8>>>,
}

#[uniffi::export]
impl MobilePty {
    /// Drains ordered output without ever waiting on the network or renderer.
    pub fn read(&self) -> Result<Vec<u8>, MobileError> {
        if !lock(&self.binding.projection).can_read_direct(&self.binding.target)? {
            return Ok(Vec::new());
        }
        if *self.stop.borrow() {
            return Err(MobileError::TerminalEnded);
        }
        let mut output = lock(&self.output);
        let mut data = Vec::new();
        while data.len() <= MAX_WRITE - PACKET {
            match output.try_recv() {
                Ok(chunk) => data.extend(chunk),
                Err(mpsc::error::TryRecvError::Disconnected) if data.is_empty() => {
                    return Err(lock(&self.binding.error)
                        .clone()
                        .unwrap_or(MobileError::TerminalEnded));
                }
                Err(_) => break,
            }
        }
        Ok(data)
    }

    /// Waits for bytes or closure on a background reader; periodically rechecks target ownership.
    pub fn read_waiting(&self) -> Result<Vec<u8>, MobileError> {
        let (gate, wake) = &self.binding.signal;
        let guard = lock(gate);
        let data = self.read()?;
        if !data.is_empty() {
            return Ok(data);
        }
        let waited = wake
            .wait_timeout(guard, std::time::Duration::from_millis(150))
            .unwrap_or_else(PoisonError::into_inner);
        drop(waited);
        self.read()
    }

    pub fn write(&self, bytes: Vec<u8>) -> Result<(), MobileError> {
        self.binding.current()?;
        if bytes.len() > MAX_WRITE {
            return Err(MobileError::TerminalQueueFull);
        }
        if !self.binding.ready.load(Ordering::Acquire) {
            return Err(MobileError::TerminalStarting);
        }
        self.submit(Command::Write(bytes))
    }

    pub fn resize(&self, cols: u16, rows: u16) -> Result<(), MobileError> {
        self.binding.current()?;
        check_size(cols, rows)?;
        self.submit(Command::Resize(cols, rows))
    }

    pub fn close(&self) {
        self.stop.send_replace(true);
        self.binding.wake_reader();
    }
}

impl MobilePty {
    fn submit(&self, command: Command) -> Result<(), MobileError> {
        if *self.stop.borrow() {
            return Err(MobileError::TerminalEnded);
        }
        self.input
            .try_send(command)
            .map_err(|failure| match failure {
                mpsc::error::TrySendError::Full(_) => MobileError::TerminalQueueFull,
                mpsc::error::TrySendError::Closed(_) => MobileError::TerminalEnded,
            })
    }
}

impl Drop for MobilePty {
    fn drop(&mut self) {
        self.close();
    }
}

struct Request {
    binding: Arc<Binding>,
    size: (u16, u16),
    stop: watch::Receiver<bool>,
    input: mpsc::Receiver<Command>,
    output: mpsc::Sender<Vec<u8>>,
}

#[derive(Default)]
struct Selection {
    pending: Option<Request>,
    active: Option<(InputTarget, watch::Sender<bool>)>,
}

impl Selection {
    fn cancel(&mut self) {
        if let Some((_, stop)) = self.active.take() {
            stop.send_replace(true);
        }
        self.pending = None;
    }
}

#[derive(Default)]
pub(crate) struct PtyRoutes {
    pub(crate) transport: super::transport::ConnectedSsh,
    pub(crate) preview: super::service_preview::ServicePreview,
    pub(crate) images: super::images::ImageStager,
    path: OnceLock<String>,
    selection: Mutex<Selection>,
    wake: Notify,
}

impl PtyRoutes {
    pub(crate) fn accept_path(&self, path: String) {
        if path.starts_with('/') && path.len() <= 4096 && !path.contains(char::is_control) {
            let _ = self.path.set(path);
        }
    }

    pub(crate) fn cancel(&self) {
        lock(&self.selection).cancel();
    }

    /// Revoke ownership even while Swift is suspended or a write is blocked on SSH credit.
    pub(crate) fn invalidate(&self, state: &Projection) {
        let mut selection = lock(&self.selection);
        if selection
            .active
            .as_ref()
            .is_some_and(|(target, _)| state.can_read_direct(target).is_err())
        {
            selection.cancel();
        }
    }

    pub(crate) fn attach(
        &self,
        projection: Arc<Mutex<Projection>>,
        target: InputTarget,
        cols: u16,
        rows: u16,
    ) -> Result<Arc<MobilePty>, MobileError> {
        check_size(cols, rows)?;
        if self.path.get().is_none() {
            return Err(MobileError::TerminalUnavailable);
        }
        if target.popup_id.is_some() || !channel::valid_id(&target.pane_id) {
            return Err(MobileError::StaleView);
        }
        let binding = Arc::new(Binding {
            target,
            projection,
            ready: AtomicBool::new(false),
            signal: Default::default(),
            error: Mutex::new(None),
        });
        let state = lock(&binding.projection);
        if state.direct_target().as_ref() != Some(&binding.target) {
            return Err(MobileError::StaleView);
        }
        let mut selection = lock(&self.selection);
        let (stop, stopped) = watch::channel(false);
        let (input, commands) = mpsc::channel(16);
        let (output, data) = mpsc::channel(16);
        selection.cancel();
        selection.active = Some((binding.target.clone(), stop.clone()));
        selection.pending = Some(Request {
            binding: binding.clone(),
            size: (cols, rows),
            stop: stopped,
            input: commands,
            output,
        });
        drop(selection);
        drop(state);
        self.wake.notify_one();
        Ok(Arc::new(MobilePty {
            binding,
            stop,
            input,
            output: Mutex::new(data),
        }))
    }

    pub(super) async fn serve(&self, handle: Arc<client::Handle<HostKey>>, session: &str) {
        self.transport.connect(&handle);
        let opener = open::Opener::new(handle);
        loop {
            self.wake.notified().await;
            let request = lock(&self.selection).pending.take();
            if let (Some(mut request), Some(path)) = (request, self.path.get()) {
                let result = channel::run(&mut request, &opener, path, session).await;
                if let Err(error) = result {
                    *lock(&request.binding.error) = Some(error);
                }
                let binding = request.binding.clone();
                drop(request);
                binding.wake_reader();
            }
        }
    }
}
