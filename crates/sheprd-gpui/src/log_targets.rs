//! The crates whose events reach the log: this app and every Herdr crate it
//! links, so moving code into a crate of its own never silences it.

pub(crate) const LOG_TARGETS: &[&str] = &[
    // keep-sorted start
    "sheprd_backend",
    "sheprd_chat_protocol",
    "sheprd_client",
    "sheprd_commands",
    "sheprd_control_protocol",
    "sheprd_core_client",
    "sheprd_credentials",
    "sheprd_diagnostics",
    "sheprd_gpui",
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
