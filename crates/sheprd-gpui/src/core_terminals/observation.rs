//! The shared Core metadata subscription and Spaceport document observation.
//! A worker owns blocking Core reads; windows consume only published facts.

use super::workspace_document::{self, NAME, VersionedDocument};
use gpui::{App, Global};
use sheprd_backend::{Backend as _, Change, TerminalInfo};
use sheprd_core_client::{
    CoreBackend, CoreBatch,
    wire::{Doc, MachineList, MachineView},
};
use std::{
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    time::{Duration, Instant},
};

const MACHINE_REFRESH: Duration = Duration::from_secs(2);
fn needs_document(batch: &CoreBatch) -> bool {
    batch.missed
        || batch
            .documents
            .iter()
            .any(|notice| notice.machine.is_none() && notice.name == NAME)
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum Error {
    #[error(transparent)]
    Core(#[from] sheprd_core_client::Error),
    #[error(transparent)]
    Document(#[from] workspace_document::Error),
    #[error("could not start the Core observation worker")]
    Worker(#[from] std::io::Error),
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum Freshness {
    #[default]
    Missing,
    Synchronizing,
    Fresh,
    Stale,
}

#[derive(Clone, Default)]
pub(super) struct Projection {
    epoch: u64,
    freshness: Freshness,
    terminals: Vec<TerminalInfo>,
    document: Option<VersionedDocument>,
    error: Option<Arc<Error>>,
    machines: Option<Arc<[MachineView]>>,
    machine_error: Option<Arc<Error>>,
    revision: u64,
}

impl Projection {
    pub(super) fn epoch(&self) -> u64 {
        self.epoch
    }

    pub fn begin(&mut self, epoch: u64) {
        self.epoch = epoch;
        self.freshness = Freshness::Synchronizing;
        self.terminals.clear();
        self.document = None;
        self.error = None;
        self.machines = None;
        self.machine_error = None;
        self.revision += 1;
    }

    pub fn initial(
        &mut self,
        epoch: u64,
        terminals: Vec<TerminalInfo>,
        document: Result<VersionedDocument, Error>,
    ) {
        if epoch != self.epoch {
            return;
        }
        self.terminals = terminals;
        self.terminals.sort_by(|a, b| a.id.cmp(&b.id));
        self.replace_document(document);
        self.freshness = Freshness::Fresh;
        self.revision += 1;
    }

    pub fn batch(
        &mut self,
        epoch: u64,
        batch: CoreBatch,
        document: Option<Result<VersionedDocument, Error>>,
    ) {
        if epoch != self.epoch || self.freshness != Freshness::Fresh {
            return;
        }
        let changed = !batch.changes.is_empty() || document.is_some() || batch.missed;
        for change in batch.changes {
            match change {
                Change::Upsert(info) => {
                    if let Some(old) = self.terminals.iter_mut().find(|old| old.id == info.id) {
                        *old = info;
                    } else {
                        self.terminals.push(info);
                    }
                }
                Change::Removed(id) => self.terminals.retain(|info| info.id != id),
                Change::Resync(terminals) => self.terminals = terminals,
            }
        }
        if let Some(document) = document {
            self.replace_document(document);
        } else if batch.missed {
            self.document = None;
            self.error = None;
        }
        if changed {
            self.terminals.sort_by(|a, b| a.id.cmp(&b.id));
            self.revision += 1;
        }
    }

    pub fn lost(&mut self, epoch: u64) {
        if epoch != self.epoch {
            return;
        }
        self.freshness = Freshness::Stale;
        self.terminals.clear();
        self.document = None;
        self.error = None;
        self.machines = None;
        self.machine_error = None;
        self.revision += 1;
    }

    pub fn refresh_machines(&mut self, epoch: u64, result: Result<MachineList, Error>) {
        if epoch != self.epoch || self.freshness != Freshness::Fresh {
            return;
        }
        match result {
            Ok(catalog) => {
                let changed = self.machines.as_deref() != Some(catalog.machines.as_slice())
                    || self.machine_error.is_some();
                self.machines = Some(catalog.machines.into());
                self.machine_error = None;
                if changed {
                    self.revision += 1;
                }
            }
            Err(error) => {
                self.machines = None;
                self.machine_error = Some(Arc::new(error));
                self.revision += 1;
            }
        }
    }

    fn fail(&mut self, epoch: u64, error: Error) {
        self.lost(epoch);
        if epoch == self.epoch {
            self.error = Some(Arc::new(error));
        }
    }

    fn replace_document(&mut self, result: Result<VersionedDocument, Error>) {
        match result {
            Ok(document) => {
                self.document = Some(document);
                self.error = None;
            }
            Err(error) => {
                self.document = None;
                self.error = Some(Arc::new(error));
            }
        }
    }

    pub fn terminals(&self) -> &[TerminalInfo] {
        &self.terminals
    }

    pub fn document(&self) -> Option<&VersionedDocument> {
        self.document.as_ref()
    }

    pub fn machines(&self) -> Option<&[MachineView]> {
        self.machines.as_deref()
    }

    pub fn machine_error(&self) -> Option<&Error> {
        self.machine_error.as_deref()
    }

    pub fn fresh(&self) -> bool {
        self.freshness == Freshness::Fresh
    }

    pub fn stale(&self) -> bool {
        self.freshness == Freshness::Stale
    }

    pub fn error(&self) -> Option<&Error> {
        self.error.as_deref()
    }

    pub fn revision(&self) -> u64 {
        self.revision
    }

    pub fn started(&self) -> bool {
        self.epoch != 0
    }
}

fn parse_document(
    result: Result<Doc, sheprd_core_client::Error>,
) -> Result<VersionedDocument, Error> {
    let doc = result?;
    Ok(VersionedDocument::read(doc)?)
}

fn sort_machines(mut catalog: MachineList) -> MachineList {
    catalog.machines.sort_by(|a, b| {
        a.machine
            .name
            .cmp(&b.machine.name)
            .then(a.machine.id.cmp(&b.machine.id))
    });
    catalog
}

type Mailbox = Arc<Mutex<Option<Projection>>>;
type MachineMailbox = Arc<Mutex<Option<Result<MachineList, Error>>>>;

fn publish(mailbox: &Mailbox, projection: &Projection) {
    replace_latest(mailbox, projection.clone());
}

fn replace_latest<T>(mailbox: &Arc<Mutex<Option<T>>>, latest: T) {
    let mut slot = match mailbox.lock() {
        Ok(slot) => slot,
        Err(poisoned) => poisoned.into_inner(),
    };
    *slot = Some(latest);
}

fn take_latest<T>(mailbox: &Arc<Mutex<Option<T>>>) -> Option<T> {
    let mut slot = match mailbox.lock() {
        Ok(slot) => slot,
        Err(poisoned) => poisoned.into_inner(),
    };
    slot.take()
}

/// One app-wide metadata subscription. Core attachments remain pane-owned.
#[derive(Default)]
pub(super) struct Observation {
    projection: Projection,
    mailbox: Option<Mailbox>,
    running: Option<Arc<AtomicBool>>,
    stop: Option<Arc<AtomicBool>>,
    last_start: Option<Instant>,
}

impl Global for Observation {}

impl Drop for Observation {
    fn drop(&mut self) {
        if let Some(stop) = &self.stop {
            stop.store(true, Ordering::Release);
        }
    }
}

impl Observation {
    pub fn install(cx: &mut App) {
        cx.set_global(Self::default());
    }

    pub fn ensure(&mut self, backend: CoreBackend) {
        if self.mailbox.is_some()
            || self
                .last_start
                .is_some_and(|then| then.elapsed() < Duration::from_secs(2))
        {
            return;
        }
        let epoch = self.projection.epoch + 1;
        self.projection.begin(epoch);
        self.last_start = Some(Instant::now());
        let mailbox = Arc::new(Mutex::new(None));
        let stop = Arc::new(AtomicBool::new(false));
        let running = Arc::new(AtomicBool::new(true));
        let worker_stop = Arc::clone(&stop);
        let worker_running = Arc::clone(&running);
        let worker_mailbox = Arc::clone(&mailbox);
        match std::thread::Builder::new()
            .name("herdr-core-observe".into())
            .spawn(move || run(backend, epoch, worker_stop, worker_running, worker_mailbox))
        {
            Ok(_) => {
                self.mailbox = Some(mailbox);
                self.running = Some(running);
                self.stop = Some(stop);
            }
            Err(error) => self.projection.fail(epoch, Error::Worker(error)),
        }
    }

    pub fn poll(&mut self) -> bool {
        let before = self.projection.revision();
        if let Some(latest) = self.mailbox.as_ref().and_then(take_latest)
            && latest.epoch == self.projection.epoch
        {
            self.projection = latest;
        }
        if self
            .running
            .as_ref()
            .is_some_and(|running| !running.load(Ordering::Acquire))
        {
            if self.projection.fresh() {
                self.projection.lost(self.projection.epoch);
            }
            self.mailbox = None;
            self.running = None;
            self.stop = None;
        }
        self.projection.revision() != before
    }

    pub fn projection(&self) -> &Projection {
        &self.projection
    }
}

#[cfg(test)]
pub(super) fn set_document_for_test(cx: &mut App, doc: Doc) {
    if !cx.has_global::<Observation>() {
        Observation::install(cx);
    }
    let observation = cx.global_mut::<Observation>();
    let epoch = observation.projection.epoch + 1;
    observation.projection.begin(epoch);
    observation
        .projection
        .initial(epoch, vec![], Ok(VersionedDocument::read(doc).unwrap()));
}

#[cfg(test)]
pub(super) fn set_machines_for_test(cx: &mut App, machines: Option<Vec<MachineView>>) {
    if !cx.has_global::<Observation>() {
        Observation::install(cx);
    }
    let observation = cx.global_mut::<Observation>();
    let epoch = observation.projection.epoch + 1;
    observation.projection.begin(epoch);
    if let Some(machines) = machines {
        observation.projection.freshness = Freshness::Fresh;
        observation
            .projection
            .refresh_machines(epoch, Ok(sort_machines(MachineList { machines })));
    } else {
        observation.projection.lost(epoch);
    }
}

#[cfg(test)]
pub(super) fn set_machine_catalog_pending_for_test(cx: &mut App) {
    if !cx.has_global::<Observation>() {
        Observation::install(cx);
    }
    let observation = cx.global_mut::<Observation>();
    let epoch = observation.projection.epoch + 1;
    observation.projection.begin(epoch);
}

#[cfg(test)]
pub(super) fn set_machine_error_for_test(cx: &mut App, error: sheprd_core_client::Error) {
    if !cx.has_global::<Observation>() {
        Observation::install(cx);
    }
    let observation = cx.global_mut::<Observation>();
    let epoch = observation.projection.epoch + 1;
    observation.projection.begin(epoch);
    observation.projection.freshness = Freshness::Fresh;
    observation
        .projection
        .refresh_machines(epoch, Err(Error::Core(error)));
}

fn run(
    backend: CoreBackend,
    epoch: u64,
    stop: Arc<AtomicBool>,
    running: Arc<AtomicBool>,
    mailbox: Mailbox,
) {
    let mut projection = Projection::default();
    projection.begin(epoch);
    if let Err(error) = watch(backend, epoch, &stop, &mailbox, &mut projection) {
        projection.fail(epoch, error);
        publish(&mailbox, &projection);
    }
    stop.store(true, Ordering::Release);
    running.store(false, Ordering::Release);
}

fn watch(
    backend: CoreBackend,
    epoch: u64,
    stop: &Arc<AtomicBool>,
    mailbox: &Mailbox,
    projection: &mut Projection,
) -> Result<(), Error> {
    let (terminals, mut changes) = backend.watch()?;
    let document = parse_document(backend.core().doc_get(NAME, None));
    projection.initial(epoch, terminals, document);
    publish(mailbox, projection);
    let machine_results: MachineMailbox = Arc::new(Mutex::new(None));
    let read_backend = backend.clone();
    let read_stop = Arc::clone(stop);
    let read_results = Arc::clone(&machine_results);
    if let Err(error) = std::thread::Builder::new()
        .name("herdr-core-machines".into())
        .spawn(move || {
            while !read_stop.load(Ordering::Acquire) {
                let result = read_backend
                    .core()
                    .machine_list()
                    .map(sort_machines)
                    .map_err(Error::Core);
                replace_latest(&read_results, result);
                let next = Instant::now() + MACHINE_REFRESH;
                while !read_stop.load(Ordering::Acquire) && Instant::now() < next {
                    std::thread::sleep(
                        next.saturating_duration_since(Instant::now())
                            .min(Duration::from_millis(100)),
                    );
                }
            }
        })
    {
        projection.refresh_machines(epoch, Err(Error::Worker(error)));
        publish(mailbox, projection);
    }
    while !stop.load(Ordering::Acquire) {
        let batch = changes.next_batch(Duration::from_millis(200))?;
        let document =
            needs_document(&batch).then(|| parse_document(backend.core().doc_get(NAME, None)));
        if batch.missed || !batch.changes.is_empty() || document.is_some() {
            projection.batch(epoch, batch, document);
            publish(mailbox, projection);
        }
        if let Some(result) = take_latest(&machine_results) {
            projection.refresh_machines(epoch, result);
            publish(mailbox, projection);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests;
