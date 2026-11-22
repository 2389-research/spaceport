// ABOUTME: Collects typed failures from the shared host helpers.
// ABOUTME: Preserves the cause when callers report a host-selection refusal.

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    HostScript(#[from] crate::host_script::Error),
}
