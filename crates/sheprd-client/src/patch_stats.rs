// ABOUTME: Defines scalar surface patch counters for a single connection.
// ABOUTME: Assigns process-local identities for the private test observer.
use serde::{Deserialize, Serialize};
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicU64, Ordering},
};

/// Cumulative surface counters for one connection within this process.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct Snapshot {
    pub connection_id: u64,
    pub delta: ReconstructionCounters,
    pub full_surface_messages: u64,
    pub ordinary: MutationCounters,
    pub overflow: bool,
    pub reuse: ReconstructionCounters,
    pub scroll: MutationCounters,
}

/// Work performed by in-place patches and any copy-on-write they trigger.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct MutationCounters {
    pub attempts: u64,
    pub cow_copies: u64,
    /// Elapsed time inside `Arc::make_mut`.
    pub cow_elapsed_ns: u64,
    pub empty_baselines: u64,
    pub main_cells_copied: u64,
    pub successes: u64,
}

/// Work performed by encodings that reconstruct a replacement surface.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct ReconstructionCounters {
    pub accepted: u64,
    pub attempts: u64,
    pub main_cells_copied: u64,
    /// Uses of a popup baseline, which the native measurement cannot qualify.
    pub popup_baselines_unsupported: u64,
    pub reconstruction_elapsed_ns: u64,
    pub reconstructions: u64,
}

pub(crate) type Record = Arc<Mutex<Snapshot>>;

pub(crate) fn new_record() -> Option<Record> {
    static LAST_CONNECTION_ID: AtomicU64 = AtomicU64::new(0);
    let previous = LAST_CONNECTION_ID
        .try_update(Ordering::Relaxed, Ordering::Relaxed, |id| id.checked_add(1))
        .ok()?;
    Some(Arc::new(Mutex::new(Snapshot {
        connection_id: previous + 1,
        ..Snapshot::default()
    })))
}
