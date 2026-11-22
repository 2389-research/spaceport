// ABOUTME: Names the crates whose tracing events the diagnostic log keeps.
// ABOUTME: Keeps the compiler-derived GUI crate name and the workspace prefix in one place.
//! The crates whose events reach the log: this app, and every workspace
//! crate, so moving code into a crate of its own never silences it.

use spaceport_diagnostics::Targets;

/// The log keeps this binary's own crate, named by the compiler so a rename
/// cannot drop it, and every crate whose name starts with `spaceport_`, as all
/// workspace crates the app links do. A workspace crate named any other way
/// must be added to `roots`, or its events are dropped without a trace.
pub(crate) const LOG_TARGETS: Targets = Targets {
    crate_prefix: "spaceport_",
    roots: &[env!("CARGO_CRATE_NAME")],
};

#[cfg(test)]
mod tests;
