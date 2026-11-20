// ABOUTME: Checks the private target boundary for Core frame observation.
// ABOUTME: Requires matching fixture identity and refuses missing or unrelated targets.
use std::ffi::OsStr;

#[path = "../../../test-support/core_frame_scope.rs"]
mod scope;
use scope::matches_target;

const NONCE: &str = "00112233445566778899aabbccddeeff";

fn target() -> (String, String) {
    (
        format!("sp-{NONCE}"),
        format!(r"\\.\pipe\spaceport-core-{NONCE}"),
    )
}

#[test]
fn captured_core_scope_accepts_its_exact_private_runtime() {
    let (session, socket) = target();
    assert!(matches_target(
        Some(OsStr::new(NONCE)),
        Some(OsStr::new(&session)),
        Some(OsStr::new(&socket)),
    ));
}

#[test]
fn captured_core_scope_refuses_missing_and_unrelated_targets() {
    let (session, socket) = target();
    let nonce = Some(OsStr::new(NONCE));
    let session = Some(OsStr::new(&session));
    let socket = Some(OsStr::new(&socket));
    for values in [
        (None, session, socket),
        (nonce, None, socket),
        (nonce, session, None),
        (nonce, Some(OsStr::new("sp-other")), socket),
        (nonce, session, Some(OsStr::new(r"\\.\pipe\herdr-core"))),
    ] {
        assert!(!matches_target(values.0, values.1, values.2));
    }
}

#[test]
fn captured_core_scope_refuses_malformed_nonces() {
    let (session, socket) = target();
    for nonce in [
        "",
        "001122",
        "gg112233445566778899aabbccddeeff",
        "éééééééééééééééé",
    ] {
        assert!(!matches_target(
            Some(OsStr::new(nonce)),
            Some(OsStr::new(&session)),
            Some(OsStr::new(&socket)),
        ));
    }
}
