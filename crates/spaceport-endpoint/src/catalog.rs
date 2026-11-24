//! The saved-host catalog: background loads of the saved devices, the
//! serialized writes of this client's host selection, and the device list those
//! loads describe. SSH devices come from upstream's endpoint catalog; WSL
//! distributions from this client's own list beside it, which upstream never
//! reads.
//!
//! Nothing here holds a window: a load runs on its own thread, a choice is
//! written by one write at a time so rapid choices cannot finish backwards, and
//! the window rebuilds its endpoints from the list this module returns.

use crate::{
    Error, Result, StorageFailure,
    ids::{LOCAL, SAVED_PREFIX, WSL_PREFIX},
};
use spaceport_client::{ConnectTarget, SavedHost, WslHost};
use std::{
    sync::mpsc,
    time::{Duration, Instant},
};

/// The saved devices a window lists, loaded on their own threads: what the
/// catalog this window's target keeps last answered, the endpoint ID to restore,
/// and the choice still to be written back.
pub struct Catalog {
    development: Option<bool>,
    pending: Option<mpsc::Receiver<Result<CatalogUpdate>>>,
    next_poll: Instant,
    /// The endpoint ID to restore, or `None` for Local.
    pub desired: Option<String>,
    pub initialized: bool,
    pub restore_pending: bool,
    pub queued_write: Option<Option<String>>,
    writing: Option<mpsc::Receiver<Result<()>>>,
    /// The storage failure last warned about. Loads retry every two seconds,
    /// so each distinct failure warns once until a load succeeds.
    last_failure: Option<StorageFailure>,
}

/// One load's answer: the saved SSH hosts and WSL distributions, and the
/// endpoint ID the stored selection names, read only at startup.
pub struct CatalogUpdate {
    pub hosts: Vec<SavedHost>,
    pub wsl: Vec<WslHost>,
    /// The endpoint ID the stored selection names, read only at startup.
    selection: Option<Option<String>>,
}

impl CatalogUpdate {
    /// Whether `id` is an endpoint this catalog offers and the user may select.
    fn offers(&self, id: &str) -> bool {
        if let Some(profile) = id.strip_prefix(SAVED_PREFIX) {
            return self
                .hosts
                .iter()
                .any(|host| host.enabled && host.id == profile);
        }
        id.strip_prefix(WSL_PREFIX)
            .is_some_and(|distro| self.wsl.iter().any(|host| host.distro == distro))
    }
}

/// Load both catalogs. A WSL distribution chosen last wins over upstream's
/// selection, which cannot name one and so says Local whenever it was chosen.
fn load(development: bool, startup: bool) -> Result<CatalogUpdate> {
    // Only Windows has distributions; elsewhere the file is never read.
    let wsl = if cfg!(windows) {
        spaceport_client::load_wsl_hosts(development)?
    } else {
        Default::default()
    };
    if !startup {
        return Ok(CatalogUpdate {
            hosts: spaceport_client::load_saved_hosts(development)?,
            wsl: wsl.hosts,
            selection: None,
        });
    }
    let (hosts, selected) = spaceport_client::load_saved_host_selection(development)?;
    let selection = match wsl.selected_host() {
        Some(host) => Some(format!("{WSL_PREFIX}{}", host.distro)),
        None => selected.map(|id| format!("{SAVED_PREFIX}{id}")),
    };
    Ok(CatalogUpdate {
        hosts,
        wsl: wsl.hosts,
        selection: Some(selection),
    })
}

/// Persist the choice of `selected`, an endpoint ID or `None` for Local, to
/// whichever store can name it, clearing the other.
fn store(development: bool, selected: Option<&str>) -> Result<()> {
    let profile = selected.and_then(|id| id.strip_prefix(SAVED_PREFIX));
    let distro = selected.and_then(|id| id.strip_prefix(WSL_PREFIX));
    spaceport_client::store_saved_host_selection(development, profile)?;
    spaceport_client::store_wsl_selection(development, distro)?;
    Ok(())
}

impl Catalog {
    /// The catalog this window's target keeps. An explicit socket has none:
    /// it is given one machine to attach to and no device list to show.
    pub fn new(target: &ConnectTarget) -> Self {
        let development = match target {
            ConnectTarget::Socket(_) => None,
            ConnectTarget::Session { development, .. } => Some(*development),
            _ => Some(false),
        };
        Self::with_development(development)
    }

    /// A catalog that reads and writes nothing.
    pub fn inactive() -> Self {
        Self::with_development(None)
    }

    fn with_development(development: Option<bool>) -> Self {
        Self {
            development,
            pending: None,
            next_poll: Instant::now(),
            desired: None,
            initialized: false,
            restore_pending: false,
            queued_write: None,
            writing: None,
            last_failure: None,
        }
    }

    /// What a load found, when one finished since the last poll.
    pub fn poll(&mut self) -> Option<Result<CatalogUpdate>> {
        let development = self.development?;
        if let Some(result) = self.pending.as_ref().and_then(|rx| rx.try_recv().ok()) {
            self.pending = None;
            self.next_poll = Instant::now() + Duration::from_secs(2);
            if let Some(failure) = self.new_failure(&result) {
                failure.warn("Host catalog"); // i18n: data
            }
            return Some(result);
        }
        if self.pending.is_none() && Instant::now() >= self.next_poll {
            let (tx, rx) = mpsc::sync_channel(1);
            self.pending = Some(rx);
            let startup = !self.initialized;
            if let Err(error) = std::thread::Builder::new()
                .name("herdr-gui-catalog".into())
                .spawn(move || {
                    let _ = tx.send(load(development, startup));
                })
            {
                self.pending = None;
                self.next_poll = Instant::now() + Duration::from_secs(2);
                return Some(Err(error.into()));
            }
        }
        None
    }

    /// The storage failure behind `result` when it differs from the last one.
    fn new_failure(&mut self, result: &Result<CatalogUpdate>) -> Option<StorageFailure> {
        let failure = result
            .as_ref()
            .err()
            .and_then(|error| StorageFailure::find(error));
        if failure == self.last_failure {
            return None;
        }
        self.last_failure = failure.clone();
        failure
    }

    /// Whether the saved devices have been read once, so a device missing
    /// from the window is really not saved rather than not loaded yet.
    pub fn loaded(&self) -> bool {
        self.initialized
    }

    /// Take a load's answer: the first one restores the stored selection, and
    /// a later one cancels it once the entry it names is gone.
    pub fn accept(&mut self, update: &CatalogUpdate) {
        if !self.initialized {
            self.desired = update.selection.clone().flatten();
            self.restore_pending = self.desired.is_some();
            self.initialized = true;
        }
        if self.desired.as_ref().is_some_and(|id| !update.offers(id)) {
            self.desired = None;
            self.restore_pending = false;
        }
    }

    /// Remember `id` as this window's choice, an endpoint ID or `LOCAL`.
    pub fn choose(&mut self, id: &str) {
        // Also cancels an in-flight startup restore when Local is clicked.
        self.initialized = true;
        self.restore_pending = false;
        self.desired = (id != LOCAL).then(|| id.to_owned());
        if self.development.is_some() {
            self.queued_write = Some(self.desired.clone());
        }
    }

    /// Write a queued choice on its own thread, one at a time, and answer with
    /// what the last finished write failed with.
    pub fn poll_write(&mut self) -> Option<Error> {
        let development = self.development?;
        let mut error = None;
        if let Some(result) = self.writing.as_ref().and_then(|rx| rx.try_recv().ok()) {
            self.writing = None;
            error = result.err();
        }
        // Serialize this client's writes so rapid choices cannot finish backwards.
        if self.writing.is_none()
            && let Some(selected) = self.queued_write.take()
        {
            let (tx, rx) = mpsc::sync_channel(1);
            match std::thread::Builder::new()
                .name("herdr-gui-selection".into())
                .spawn(move || {
                    let _ = tx.send(store(development, selected.as_deref()));
                }) {
                Ok(_) => self.writing = Some(rx),
                Err(e) => error = Some(e.into()),
            }
        }
        error
    }
}

/// One saved device as its catalog describes it.
pub struct Device {
    pub id: String,
    pub label: String,
    pub target: ConnectTarget,
    pub enabled: bool,
}

/// Every saved device in sidebar order: SSH hosts as upstream lists them, then
/// WSL distributions in the order they were added. A distribution has no
/// disabled state; removing it is how it stops being dialled.
pub fn devices(hosts: Vec<SavedHost>, wsl: Vec<WslHost>) -> Vec<Device> {
    let ssh = hosts.into_iter().map(|host| Device {
        id: format!("{SAVED_PREFIX}{}", host.id),
        label: host.label,
        target: ConnectTarget::Ssh {
            target: host.target,
            session: host.session,
        },
        enabled: host.enabled,
    });
    let wsl = wsl.into_iter().map(|host| Device {
        id: format!("{WSL_PREFIX}{}", host.distro),
        label: host.distro.clone(),
        target: ConnectTarget::Wsl {
            distro: host.distro,
            session: host.session,
        },
        enabled: true,
    });
    ssh.chain(wsl).collect()
}

/// Whether a saved entry differs from the one this endpoint was last reconciled
/// against, which is what an edit to a device's saved profile looks like. Both
/// sides are the entry the device was *saved* with, never the session its
/// connection is on: the sessions list deliberately points an endpoint at other
/// sessions of the same device, so the live target is not the device's identity.
pub fn entry_changed(saved: &ConnectTarget, entry: &ConnectTarget) -> bool {
    saved != entry
}

#[cfg(test)]
mod tests;
