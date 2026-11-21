// ABOUTME: Qualifies passive copy counter observations for the native profile harness.
// ABOUTME: Carries per-connection scalar readings without estimating CPU cost or bytes.
use super::report::Observation;
use serde::Serialize;
use sheprd_client::patch_stats::{MutationCounters, ReconstructionCounters, Snapshot};
use std::collections::BTreeMap;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum Mode {
    // keep-sorted start
    Settled,
    Streaming,
    // keep-sorted end
}

#[derive(Debug, Eq, PartialEq, Serialize)]
pub(super) struct ConnectionWork {
    pub(super) before: Snapshot,
    pub(super) after: Snapshot,
}

#[derive(Debug, Eq, PartialEq, thiserror::Error)]
pub(super) enum Error {
    // keep-sorted start
    #[error("copy counters moved backwards for connection {connection_id}")]
    BackwardCounters { connection_id: u64 },
    #[error("an observation has conflicting readings for connection {connection_id}")]
    ConflictingReading { connection_id: u64 },
    #[error("copy counters overflowed for connection {connection_id}")]
    CounterOverflow { connection_id: u64 },
    #[error("terminal group {group} changed or lacks a counter owner")]
    CounterOwner { group: usize },
    #[error("copy counters are unavailable")]
    CounterUnavailable,
    #[error("connection {connection_id} attempted a mutation with an empty baseline")]
    EmptyBaseline { connection_id: u64 },
    #[error("copy observations have different or ambiguous terminal group mappings")]
    GroupMapping,
    #[error("streaming copy observations show no accepted updates or full surface messages")]
    NoAcceptedUpdate,
    #[error("copy observations have no eligible presented terminal groups")]
    NoTerminalGroups,
    #[error("connection {connection_id} used an unsupported popup baseline")]
    PopupBaseline { connection_id: u64 },
    #[error("terminal group {group} did not establish a popup-free presentation")]
    PopupState { group: usize },
    #[error("copy observations need the same nonzero GUI process ID")]
    ProcessIdentity,
    #[error("terminal group {group} lacks a current populated presentation on a live connection")]
    TerminalPresentation { group: usize },
    #[error("copy observations need the same nonzero window ID")]
    WindowIdentity,
    // keep-sorted end
}

// The interval can qualify even if an unsupported baseline was used earlier.
pub(super) fn validate(
    before: &Observation,
    after: &Observation,
    mode: Mode,
) -> Result<Vec<ConnectionWork>, Error> {
    if before.gui_pid == 0 || before.gui_pid != after.gui_pid {
        return Err(Error::ProcessIdentity);
    }
    if before.window_id == 0 || before.window_id != after.window_id {
        return Err(Error::WindowIdentity);
    }

    let before_groups = terminal_readings(before)?;
    let after_groups = terminal_readings(after)?;
    if before_groups.len() != after_groups.len() {
        return Err(Error::GroupMapping);
    }
    if before_groups.is_empty() {
        return Err(Error::NoTerminalGroups);
    }

    let mut connections = BTreeMap::new();
    for (group, (before_tab, before)) in before_groups {
        let &(after_tab, after) = after_groups.get(&group).ok_or(Error::GroupMapping)?;
        if before_tab != after_tab {
            return Err(Error::GroupMapping);
        }
        let connection_id = before.connection_id;
        if connection_id != after.connection_id {
            return Err(Error::CounterOwner { group });
        }
        let record = connections.entry(connection_id).or_insert(ConnectionWork {
            before: *before,
            after: *after,
        });
        if record.before != *before || record.after != *after {
            return Err(Error::ConflictingReading { connection_id });
        }
    }

    for record in connections.values() {
        if counters_decreased(&record.before, &record.after) {
            return Err(Error::BackwardCounters {
                connection_id: record.before.connection_id,
            });
        }
    }
    for ConnectionWork { before, after } in connections.values() {
        let connection_id = before.connection_id;
        if after.delta.popup_baselines_unsupported > before.delta.popup_baselines_unsupported
            || after.reuse.popup_baselines_unsupported > before.reuse.popup_baselines_unsupported
        {
            return Err(Error::PopupBaseline { connection_id });
        }
        if after.ordinary.empty_baselines > before.ordinary.empty_baselines
            || after.scroll.empty_baselines > before.scroll.empty_baselines
        {
            return Err(Error::EmptyBaseline { connection_id });
        }
    }
    if mode == Mode::Streaming
        && !connections
            .values()
            .any(|ConnectionWork { before, after }| {
                after.ordinary.successes > before.ordinary.successes
                    || after.scroll.successes > before.scroll.successes
                    || after.delta.accepted > before.delta.accepted
                    || after.reuse.accepted > before.reuse.accepted
                    || after.full_surface_messages > before.full_surface_messages
            })
    {
        return Err(Error::NoAcceptedUpdate);
    }
    Ok(connections.into_values().collect())
}

fn terminal_readings(
    observation: &Observation,
) -> Result<BTreeMap<usize, (&str, &Snapshot)>, Error> {
    let mut readings = BTreeMap::new();
    for group in observation
        .groups
        .iter()
        .filter(|group| group.shown_terminal)
    {
        let index = group.index;
        let presented = group
            .presented
            .as_ref()
            .filter(|frame| {
                group.connected
                    && frame.current
                    && frame.columns > 0
                    && frame.rows > 0
                    && !frame.pane_ids.is_empty()
                    && frame.nonblank_cells > 0
            })
            .ok_or(Error::TerminalPresentation { group: index })?;
        if presented.has_popup != Some(false) {
            return Err(Error::PopupState { group: index });
        }
        let tab = group
            .tab
            .as_deref()
            .filter(|tab| !tab.is_empty())
            .ok_or(Error::GroupMapping)?;
        let stats = group
            .patch_stats
            .as_ref()
            .ok_or(Error::CounterUnavailable)?;
        if stats.connection_id == 0 {
            return Err(Error::CounterOwner { group: index });
        }
        if stats.overflow {
            return Err(Error::CounterOverflow {
                connection_id: stats.connection_id,
            });
        }
        if readings.insert(index, (tab, stats)).is_some() {
            return Err(Error::GroupMapping);
        }
    }
    Ok(readings)
}

fn counters_decreased(before: &Snapshot, after: &Snapshot) -> bool {
    let Snapshot {
        connection_id: _,
        delta,
        full_surface_messages,
        ordinary,
        overflow: _,
        reuse,
        scroll,
    } = *before;
    full_surface_messages > after.full_surface_messages
        || mutation_decreased(ordinary, after.ordinary)
        || mutation_decreased(scroll, after.scroll)
        || reconstruction_decreased(delta, after.delta)
        || reconstruction_decreased(reuse, after.reuse)
}

fn mutation_decreased(before: MutationCounters, after: MutationCounters) -> bool {
    let MutationCounters {
        attempts,
        cow_copies,
        cow_elapsed_ns,
        empty_baselines,
        main_cells_copied,
        successes,
    } = before;
    attempts > after.attempts
        || cow_copies > after.cow_copies
        || cow_elapsed_ns > after.cow_elapsed_ns
        || empty_baselines > after.empty_baselines
        || main_cells_copied > after.main_cells_copied
        || successes > after.successes
}

fn reconstruction_decreased(before: ReconstructionCounters, after: ReconstructionCounters) -> bool {
    let ReconstructionCounters {
        accepted,
        attempts,
        main_cells_copied,
        popup_baselines_unsupported,
        reconstruction_elapsed_ns,
        reconstructions,
    } = before;
    accepted > after.accepted
        || attempts > after.attempts
        || main_cells_copied > after.main_cells_copied
        || popup_baselines_unsupported > after.popup_baselines_unsupported
        || reconstruction_elapsed_ns > after.reconstruction_elapsed_ns
        || reconstructions > after.reconstructions
}

#[cfg(test)]
mod tests;
