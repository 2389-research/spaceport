// ABOUTME: Matches a diagnostic observer to its exact private Core fixture target.
// ABOUTME: Keeps target validation independent of process environment and UI code.
use std::ffi::OsStr;

pub(crate) fn matches_target(
    _nonce: Option<&OsStr>,
    _session: Option<&OsStr>,
    _socket: Option<&OsStr>,
) -> bool {
    false
}
