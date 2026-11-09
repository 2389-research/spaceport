//! The Windows taskbar and Start menu jump list lists saved devices as tasks.
//! Each entry starts this app with a `spaceport://device` link as its only
//! argument. Building the list takes COM calls, so a worker owns it.

mod shell;
#[cfg(test)]
mod tests;

use crate::{SheprdWindow, endpoint::LOCAL, link::Link};
use gpui::{App, Global};
use std::{
    sync::{Arc, Mutex, mpsc},
    time::{Duration, Instant},
};

const MAX_DEVICES: usize = 8;
/// Labels are shown in a narrow menu.
const MAX_LABEL: usize = 80;

/// One jump list row: what it is called, and the link it opens.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Item {
    pub title: String,
    pub link: Link,
}

/// Device tasks compared before the shell is called.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct Entries {
    pub devices: Vec<Item>,
}

/// Devices are listed Local first, then by name. Each window keeps its own
/// device order, so the list must not change with whichever window looked last.
pub(crate) fn entries(devices: &[(String, String)]) -> Entries {
    let label = |text: &str| crate::notifications::safe_text(text, MAX_LABEL);
    let mut devices: Vec<&(String, String)> = devices.iter().collect();
    devices.sort_by_cached_key(|(id, name)| (id != LOCAL, name.to_lowercase(), id.clone()));
    Entries {
        devices: devices
            .iter()
            .take(MAX_DEVICES)
            .map(|(id, name)| Item {
                title: label(&format!("Show {name}")),
                link: Link::Device { id: id.clone() },
            })
            .collect(),
    }
}

/// What the worker shares with the UI thread.
#[derive(Default)]
pub(crate) struct Mailbox {
    /// The latest entries, replacing any the worker has not taken.
    pub pending: Option<Entries>,
    /// Set when the shell refused the last entries, which are then sent
    /// again after [`RETRY`].
    pub failed: bool,
}

/// How long a refused jump list waits before it is sent again.
const RETRY: Duration = Duration::from_secs(30);

/// Whether `entries` go to the worker: when they differ from those last
/// sent, or once a retry after a refusal is due.
fn due(
    published: Option<&Entries>,
    entries: &Entries,
    retry: Option<Instant>,
    now: Instant,
) -> bool {
    published != Some(entries) || retry.is_some_and(|at| now >= at)
}

/// The app's jump list: what it shows and the worker that builds it.
#[derive(Default)]
pub(crate) struct JumpList {
    published: Option<Entries>,
    /// When to send the entries again after the shell refused them.
    retry: Option<Instant>,
    mailbox: Arc<Mutex<Mailbox>>,
    wake: Option<mpsc::SyncSender<()>>,
}

impl Global for JumpList {}

impl JumpList {
    pub(crate) fn install(mut self, cx: &mut App) {
        let mailbox = self.mailbox.clone();
        // Entries start the windowed launcher when it is installed, so a
        // click opens no console window; a `cargo run` build has none.
        let exe = cx
            .try_global::<crate::taskbar::Taskbar>()
            .and_then(|taskbar| taskbar.launcher().map(std::path::Path::to_owned))
            .or_else(|| std::env::current_exe().ok());
        let (wake, receiver) = mpsc::sync_channel::<()>(1);
        let started = std::thread::Builder::new()
            .name("jump-list".into())
            .spawn(move || shell::run(&receiver, &mailbox, exe.as_deref()));
        match started {
            Ok(_) => self.wake = Some(wake),
            Err(error) => tracing::warn!(%error, "The jump list is off"),
        }
        cx.set_global(self);
    }

    /// Notes what a window shows; called from its poll, so it does no work
    /// unless the devices or the shell's list changed.
    pub(crate) fn observe(view: &SheprdWindow, cx: &mut App) {
        if cx.try_global::<Self>().is_none() {
            return;
        }
        let devices: Vec<(String, String)> = view
            .endpoints
            .iter()
            .map(|endpoint| (endpoint.id.clone(), endpoint.label.clone()))
            .collect();
        let list = cx.global_mut::<Self>();
        let failed = list
            .mailbox
            .lock()
            .map(|mut mailbox| std::mem::take(&mut mailbox.failed))
            .unwrap_or_default();
        let now = Instant::now();
        if failed {
            list.retry = Some(now + RETRY);
        }
        let entries = entries(&devices);
        if !due(list.published.as_ref(), &entries, list.retry, now) {
            return;
        }
        list.retry = None;
        list.published = Some(entries.clone());
        if let Ok(mut mailbox) = list.mailbox.lock() {
            mailbox.pending = Some(entries);
        }
        if let Some(wake) = &list.wake {
            // Full means the worker is already due to look.
            let _ = wake.try_send(());
        }
    }
}
