// ABOUTME: Groups native working-profile measurement and workload helpers.
// ABOUTME: Keeps pure clock parsing checks independent of profile execution.
mod harness;
mod markers;
mod measurements;
mod peer;
#[cfg(windows)]
mod windows_contracts;
mod wire;

#[path = "../../src/working_profile/report.rs"]
mod report;

pub(super) use harness::run;
#[cfg(windows)]
pub(super) use harness::run_core;
