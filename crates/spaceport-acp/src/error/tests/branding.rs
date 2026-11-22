// ABOUTME: Checks the attach-listener diagnostic and its original I/O cause.
// ABOUTME: Names Spaceport without changing private endpoint paths or error types.
use super::*;
use std::error::Error as _;

#[test]
fn attach_listener_failure_names_spaceport_and_keeps_its_cause() {
    let path = std::path::PathBuf::from("private-fixture.socket");
    let error = Error::Bind {
        path: path.clone(),
        source: io::Error::new(io::ErrorKind::PermissionDenied, "fixture refusal"),
    };
    assert_eq!(
        error.to_string(),
        format!("could not listen for Spaceport at {}", path.display())
    );
    assert_eq!(
        error
            .source()
            .and_then(|source| source.downcast_ref::<io::Error>())
            .map(io::Error::kind),
        Some(io::ErrorKind::PermissionDenied)
    );
}
