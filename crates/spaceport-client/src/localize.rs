//! The failures this crate returns, as the person using the app reads them
//! (docs/i18n.md): one file each, beside the `Error` it words. Every enum
//! keeps its English `Display` for logs, and an enum wrapping another crate's
//! error calls that error's own `localize()`, so each failure has one message
//! as it has one definition. The messages are `error-client.ftl`, and the
//! sub-errors' own files beside it.
// i18n: migrated

mod clipboard;
mod root;
mod sessions;
mod ssh;
mod terminal_attach;
mod wsl;

#[cfg(test)]
mod tests;
