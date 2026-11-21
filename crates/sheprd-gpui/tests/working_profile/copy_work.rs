// ABOUTME: Qualifies passive copy counter observations for the native profile harness.
// ABOUTME: Carries per-connection scalar readings without estimating CPU cost or bytes.
use super::report::Observation;
use serde::Serialize;
use sheprd_client::patch_stats::Snapshot;

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
    #[error("copy observations are for different windows")]
    WindowIdentity,
    // keep-sorted end
}

// The interval can qualify even if an unsupported baseline was used earlier.
pub(super) fn validate(
    _before: &Observation,
    _after: &Observation,
    _mode: Mode,
) -> Result<Vec<ConnectionWork>, Error> {
    // Refuse until the contract tests reach their intended runtime failures in CI.
    Err(Error::CounterUnavailable)
}

#[cfg(test)]
mod tests;
