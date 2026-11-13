// ABOUTME: Selects the actual GUI crate and its linked crates for diagnostic capture.
// ABOUTME: Keeps compiler-derived GUI identity aligned with default tracing targets.
//! The crates whose events reach the log: this app and every Herdr crate it
//! links, so moving code into a crate of its own never silences it.

pub(crate) const LOG_TARGETS: &[&str] = &[
    env!("CARGO_CRATE_NAME"),
    // keep-sorted start
    "sheprd_backend",
    "sheprd_chat_protocol",
    "sheprd_client",
    "sheprd_commands",
    "sheprd_control_protocol",
    "sheprd_core_client",
    "sheprd_diagnostics",
    "sheprd_i18n",
    "sheprd_protocol",
    "sheprd_remote",
    "sheprd_terminal",
    "sheprd_usage",
    // keep-sorted end
];

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests;
