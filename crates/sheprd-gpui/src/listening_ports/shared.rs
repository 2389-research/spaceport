// ABOUTME: Shares one listening-port worker per exact host across application windows.
// ABOUTME: Publishes immutable readings while the final window lease owns worker shutdown.
use super::{Host, Origin, Reading, Worker, spawn};
use gpui::Global;
use std::{
    cell::RefCell,
    collections::HashMap,
    rc::{Rc, Weak},
    sync::{Arc, mpsc},
};

#[cfg(all(test, unix))]
mod tests;

#[derive(Default)]
pub(super) struct Scans {
    hosts: HashMap<Host, Weak<Scan>>,
}

impl Global for Scans {}

pub(super) struct Scan {
    state: RefCell<State>,
}

struct State {
    worker: Option<Worker>,
    reading: Arc<Reading>,
}

impl Scans {
    pub(super) fn prune(&mut self) {
        self.hosts.retain(|_, scan| scan.strong_count() > 0);
    }

    pub(super) fn lease(&mut self, host: &Host) -> Option<Rc<Scan>> {
        if let Some(scan) = self.hosts.get(host).and_then(Weak::upgrade) {
            return Some(scan);
        }
        let worker = spawn(host.clone())?;
        let scan = Rc::new(Scan {
            state: RefCell::new(State {
                worker: Some(worker),
                reading: Arc::new(Reading {
                    origin: Origin::new(host),
                    ..Reading::default()
                }),
            }),
        });
        self.hosts.insert(host.clone(), Rc::downgrade(&scan));
        Some(scan)
    }
}

impl Scan {
    pub(super) fn poll(&self) -> Arc<Reading> {
        let mut state = self.state.borrow_mut();
        loop {
            let result = state
                .worker
                .as_ref()
                .map(|worker| worker.results.try_recv());
            match result {
                Some(Ok(result)) => {
                    Arc::make_mut(&mut state.reading).apply(result);
                }
                Some(Err(mpsc::TryRecvError::Disconnected)) => state.worker = None,
                Some(Err(mpsc::TryRecvError::Empty)) | None => break,
            }
        }
        state.reading.clone()
    }
}
