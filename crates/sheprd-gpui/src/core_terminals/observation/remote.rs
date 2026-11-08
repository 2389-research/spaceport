//! One selected machine's Spaceport document, separate from the hub's groups.
//! Immutable machine IDs and request generations fence delayed Core reads.

use super::{CoreBatch, Error, VersionedDocument};
use sheprd_core_client::wire::MachineView;
use std::sync::Arc;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct RemoteRequest {
    pub epoch: u64,
    pub generation: u64,
    pub machine_id: String,
    serial: u64,
}

#[derive(Clone)]
pub(in crate::core_terminals) enum RemoteStatus {
    Loading,
    Ready(VersionedDocument),
    Stale,
    Failed(Arc<Error>),
}

#[derive(Clone, Default)]
pub(super) struct Remote {
    connected: bool,
    current: Option<RemoteRequest>,
    next_serial: u64,
    status: Option<RemoteStatus>,
}

impl Remote {
    pub fn select(
        &mut self,
        epoch: u64,
        generation: u64,
        machine_id: Option<String>,
        machines: Option<&[MachineView]>,
    ) -> Option<RemoteRequest> {
        let Some(machine_id) = machine_id else {
            self.clear();
            return None;
        };
        self.current = Some(RemoteRequest {
            epoch,
            generation,
            machine_id,
            serial: 0,
        });
        self.connected = self
            .current
            .as_ref()
            .is_some_and(|target| connected(machines, &target.machine_id));
        self.status = Some(if self.connected {
            RemoteStatus::Loading
        } else if machines.is_some() {
            RemoteStatus::Stale
        } else {
            RemoteStatus::Loading
        });
        if self.connected { self.request() } else { None }
    }

    pub fn catalog(
        &mut self,
        epoch: u64,
        machines: Option<&[MachineView]>,
    ) -> Option<RemoteRequest> {
        let target = self.current.as_ref()?;
        if target.epoch != epoch {
            return None;
        }
        let now_connected = connected(machines, &target.machine_id);
        if !now_connected {
            self.connected = false;
            self.status = Some(RemoteStatus::Stale);
            if let Some(request) = &mut self.current {
                request.serial = 0;
            }
            return None;
        }
        if self.connected {
            return None;
        }
        self.connected = true;
        self.request()
    }

    pub fn invalidate(&mut self, epoch: u64, batch: &CoreBatch) -> Option<RemoteRequest> {
        if self
            .current
            .as_ref()
            .is_none_or(|target| target.epoch != epoch)
        {
            return None;
        }
        // Core reports a mutable machine name, not its permanent ID. Any
        // remote notice for this document invalidates the one selected ID.
        let changed = batch.missed
            || batch
                .documents
                .iter()
                .any(|notice| notice.machine.is_some() && notice.name == super::NAME);
        if !changed {
            return None;
        }
        if self.connected {
            self.request()
        } else {
            self.status = Some(RemoteStatus::Stale);
            None
        }
    }

    pub fn complete(&mut self, request: RemoteRequest, result: Result<VersionedDocument, Error>) {
        if self.current.as_ref() != Some(&request) || !self.connected {
            return;
        }
        self.status = Some(match result {
            Ok(document) => RemoteStatus::Ready(document),
            Err(error) => RemoteStatus::Failed(Arc::new(error)),
        });
    }

    pub fn status(&self) -> Option<&RemoteStatus> {
        self.status.as_ref()
    }

    pub fn matches(&self, epoch: u64, generation: u64, machine_id: &str) -> bool {
        self.current.as_ref().is_some_and(|target| {
            target.epoch == epoch
                && target.generation == generation
                && target.machine_id == machine_id
        })
    }

    pub fn clear(&mut self) {
        self.connected = false;
        self.current = None;
        self.status = None;
    }

    fn request(&mut self) -> Option<RemoteRequest> {
        self.next_serial = self.next_serial.wrapping_add(1);
        let target = self.current.as_mut()?;
        target.serial = self.next_serial;
        self.status = Some(RemoteStatus::Loading);
        Some(target.clone())
    }
}

fn connected(machines: Option<&[MachineView]>, id: &str) -> bool {
    machines.is_some_and(|machines| {
        machines
            .iter()
            .any(|view| view.machine.id == id && view.state == "connected")
    })
}
