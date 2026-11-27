//! The failures this crate returns, as the person using the app reads them
//! (docs/i18n.md): one file each, beside the `Error` it words. Every enum
//! keeps its English `Display` for logs, and an enum wrapping another crate's
//! error calls that error's own `localize()`, so each failure has one message
//! as it has one definition. The messages are `error-host-script.ftl`,
//! `error-shell.ftl` and `error-devices.ftl`.
// i18n: migrated

mod host_shell;
mod root;
mod setup;

#[cfg(test)]
mod tests;
