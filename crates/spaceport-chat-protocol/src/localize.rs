//! The failures this crate returns, as the person using the app reads them
//! (docs/i18n.md): one file each, beside the `Error` it words. Every enum
//! keeps its English `Display` for logs. The messages are
//! `error-chat-protocol.ftl` and the refusals' own `error-chat.ftl`.
// i18n: migrated

mod rejection;
mod root;

#[cfg(test)]
mod tests;
