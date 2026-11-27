//! The failures this crate returns, as the person using the app reads them
//! (docs/i18n.md): one file each, beside the `Error` it words. Every enum
//! keeps its English `Display` for logs, and an enum wrapping another crate's
//! error calls that error's own `localize()`, so each failure has one message
//! as it has one definition. The messages are `error-host-script.ftl`,
//! `error-devices.ftl`, and the host script's own `error-host-script.ftl`.
// i18n: migrated

mod root;
mod setup;

#[cfg(test)]
mod tests;
