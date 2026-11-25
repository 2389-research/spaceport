//! The failures this crate returns, as the person using the app reads them
//! (docs/i18n.md): one file each, beside the `Error` it words. Every enum
//! keeps its English `Display` for logs, and an enum wrapping another crate's
//! error calls that error's own `localize()`, so each failure has one message
//! as it has one definition. The messages are `error-usage.ftl` and
//! `error-shell.ftl`.
// i18n: migrated

mod root;
mod shell;

#[cfg(test)]
mod tests;
