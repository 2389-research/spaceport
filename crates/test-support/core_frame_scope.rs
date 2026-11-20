// ABOUTME: Matches a diagnostic observer to its exact private Core fixture target.
// ABOUTME: Keeps target validation independent of process environment and UI code.
use std::ffi::OsStr;

pub(crate) fn matches_target(
    nonce: Option<&OsStr>,
    session: Option<&OsStr>,
    socket: Option<&OsStr>,
) -> bool {
    let (Some(nonce), Some(session), Some(socket)) = (nonce, session, socket) else {
        return false;
    };
    let Some(nonce) = nonce.to_str() else {
        return false;
    };
    nonce.len() == 32
        && nonce.bytes().all(|byte| byte.is_ascii_hexdigit())
        && session == OsStr::new(&format!("sp-{nonce}"))
        && socket == OsStr::new(&format!(r"\\.\pipe\spaceport-core-{nonce}"))
}
