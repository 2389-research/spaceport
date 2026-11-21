// ABOUTME: Defines scalar surface patch counters for a single connection.
// ABOUTME: Measures baseline copies without retaining any surface.
use crate::protocol::{self, PaneSurfaceFrame};
use serde::{Deserialize, Serialize};
use std::{
    sync::{
        Arc, Mutex,
        atomic::{AtomicU64, Ordering},
    },
    time::{Duration, Instant},
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
    /// Completed calls to apply a patch, including validation failures.
    pub attempts: u64,
    pub cow_copies: u64,
    /// Wall time inside `Arc::make_mut` for actual nonempty main-grid copies.
    pub cow_elapsed_ns: u64,
    pub empty_baselines: u64,
    pub main_cells_copied: u64,
    pub successes: u64,
}

/// Work performed by encodings that reconstruct a replacement surface.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct ReconstructionCounters {
    pub accepted: u64,
    /// Completed reconstruction calls, including failures.
    pub attempts: u64,
    pub main_cells_copied: u64,
    /// Delta popup patches reconstructed from the old popup grid, outside the native scope.
    pub popup_baselines_unsupported: u64,
    /// Wall time spent in reconstruction calls, including failures.
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

// A poisoned record remains unavailable to the observer. Terminal work continues.
fn update(record: Option<&Mutex<Snapshot>>, write: impl FnOnce(&mut Snapshot, &mut bool)) {
    if let Some(record) = record
        && let Ok(mut snapshot) = record.lock()
    {
        let mut overflow = snapshot.overflow;
        write(&mut snapshot, &mut overflow);
        snapshot.overflow = overflow;
    }
}

fn add(counter: &mut u64, amount: u128, overflow: &mut bool) {
    if let Some(next) = u64::try_from(amount)
        .ok()
        .and_then(|amount| counter.checked_add(amount))
    {
        *counter = next;
    } else {
        *counter = u64::MAX;
        *overflow = true;
    }
}

pub(crate) fn record_full_surface(record: Option<&Mutex<Snapshot>>) {
    update(record, |stats, overflow| {
        add(&mut stats.full_surface_messages, 1, overflow);
    });
}

pub(crate) fn apply_patch(
    record: Option<&Mutex<Snapshot>>,
    select: impl FnOnce(&mut Snapshot) -> &mut MutationCounters,
    current: &mut Arc<PaneSurfaceFrame>,
    apply: impl FnOnce(&mut PaneSurfaceFrame) -> protocol::Result<()>,
) -> protocol::Result<()> {
    if record.is_none() {
        return apply(Arc::make_mut(current));
    }
    let cell_count = current.frame.cells.len();
    let cells_before = current.frame.cells.as_ptr();
    let started = Instant::now();
    let frame = Arc::make_mut(current);
    let elapsed = started.elapsed();
    // A weak-only relocation moves the Vec without copying its allocation.
    let copied = cell_count != 0 && cells_before != frame.frame.cells.as_ptr();
    let result = apply(frame);
    update(record, |stats, overflow| {
        let counts = select(stats);
        add(&mut counts.attempts, 1, overflow);
        if result.is_ok() {
            add(&mut counts.successes, 1, overflow);
        }
        if copied {
            add(&mut counts.cow_copies, 1, overflow);
            add(&mut counts.main_cells_copied, cell_count as u128, overflow);
            add(&mut counts.cow_elapsed_ns, elapsed.as_nanos(), overflow);
        }
        if cell_count == 0 {
            add(&mut counts.empty_baselines, 1, overflow);
        }
    });
    result
}

pub(crate) struct ReconstructionWork {
    cells_copied: Option<usize>,
    elapsed: Duration,
    patches_popup: bool,
}

pub(crate) fn measure_reconstruction(
    base: &PaneSurfaceFrame,
    patches_popup: bool,
    reconstruct: impl FnOnce(&PaneSurfaceFrame) -> protocol::Result<PaneSurfaceFrame>,
) -> (protocol::Result<PaneSurfaceFrame>, ReconstructionWork) {
    let started = Instant::now();
    let result = reconstruct(base);
    let elapsed = started.elapsed();
    let cells_copied = result.is_ok().then_some(base.frame.cells.len());
    (
        result,
        ReconstructionWork {
            cells_copied,
            elapsed,
            patches_popup,
        },
    )
}

impl ReconstructionWork {
    pub(crate) fn record(
        self,
        record: Option<&Mutex<Snapshot>>,
        select: impl FnOnce(&mut Snapshot) -> &mut ReconstructionCounters,
        accepted: bool,
    ) {
        update(record, |stats, overflow| {
            let counts = select(stats);
            add(&mut counts.attempts, 1, overflow);
            add(
                &mut counts.reconstruction_elapsed_ns,
                self.elapsed.as_nanos(),
                overflow,
            );
            if let Some(cells_copied) = self.cells_copied {
                add(&mut counts.reconstructions, 1, overflow);
                add(
                    &mut counts.main_cells_copied,
                    cells_copied as u128,
                    overflow,
                );
                if self.patches_popup {
                    add(&mut counts.popup_baselines_unsupported, 1, overflow);
                }
            }
            if accepted {
                add(&mut counts.accepted, 1, overflow);
            }
        });
    }
}
