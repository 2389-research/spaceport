//! The shared Core metadata subscription and Spaceport document observation.
//! A worker owns blocking Core reads; windows consume only published facts.

use super::workspace_document;
use gpui::{App, Global};
use sheprd_backend::{Backend as _, Change, TerminalInfo};
use sheprd_core_client::{CoreBackend, CoreBatch, wire::Doc};
use std::{
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    time::{Duration, Instant},
};

const DOCUMENT: &str = "spaceport";

fn needs_document(batch: &CoreBatch) -> bool {
    batch.missed
        || batch
            .documents
            .iter()
            .any(|notice| notice.machine.is_none() && notice.name == DOCUMENT)
}

#[derive(Debug, thiserror::Error)]
pub(super) enum Error {
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

#[derive(Clone, Debug)]
pub(super) struct VersionedDocument {
    pub version: u64,
    pub value: workspace_document::Document,
}

#[derive(Clone, Default)]
pub(super) struct Projection {
    epoch: u64,
    freshness: Freshness,
    terminals: Vec<TerminalInfo>,
    document: Option<VersionedDocument>,
    error: Option<Arc<Error>>,
    revision: u64,
}

impl Projection {
    pub fn begin(&mut self, epoch: u64) {
        self.epoch = epoch;
        self.freshness = Freshness::Synchronizing;
        self.terminals.clear();
        self.document = None;
        self.error = None;
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
        self.revision += 1;
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

    pub fn fresh(&self) -> bool {
        self.freshness == Freshness::Fresh
    }

    pub fn error(&self) -> Option<&Error> {
        self.error.as_deref()
    }

    pub fn revision(&self) -> u64 {
        self.revision
    }
}

fn parse_document(
    result: Result<Doc, sheprd_core_client::Error>,
) -> Result<VersionedDocument, Error> {
    let doc = result?;
    Ok(VersionedDocument {
        version: doc.version,
        value: workspace_document::Document::read(doc.value)?,
    })
}

type Mailbox = Arc<Mutex<Option<Projection>>>;

fn publish(mailbox: &Mailbox, projection: &Projection) {
    let latest = projection.clone();
    let mut slot = match mailbox.lock() {
        Ok(slot) => slot,
        Err(poisoned) => poisoned.into_inner(),
    };
    *slot = Some(latest);
}

fn take_latest(mailbox: &Mailbox) -> Option<Projection> {
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
    running.store(false, Ordering::Release);
}

fn watch(
    backend: CoreBackend,
    epoch: u64,
    stop: &AtomicBool,
    mailbox: &Mailbox,
    projection: &mut Projection,
) -> Result<(), Error> {
    let (terminals, mut changes) = backend.watch()?;
    let document = parse_document(backend.core().doc_get(DOCUMENT, None));
    projection.initial(epoch, terminals, document);
    publish(mailbox, projection);
    while !stop.load(Ordering::Acquire) {
        let batch = changes.next_batch(Duration::from_millis(200))?;
        let document =
            needs_document(&batch).then(|| parse_document(backend.core().doc_get(DOCUMENT, None)));
        if batch.missed || !batch.changes.is_empty() || document.is_some() {
            projection.batch(epoch, batch, document);
            publish(mailbox, projection);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests;
