use super::*;
use crate::daemon::Error;
use std::{path::PathBuf, process::ExitStatus};

#[cfg(unix)]
fn exit_status() -> ExitStatus {
    use std::os::unix::process::ExitStatusExt;
    ExitStatus::from_raw(256)
}

#[cfg(windows)]
fn exit_status() -> ExitStatus {
    use std::os::windows::process::ExitStatusExt;
    ExitStatus::from_raw(1)
}

#[test]
fn every_daemon_variant_has_a_message() {
    let errors = every_daemon_error();
    assert_localized(&errors);
    assert_no_wire_words(&errors);
}

/// Every variant of the daemon's error, for the checks above.
fn every_daemon_error() -> Vec<Error> {
    vec![
        Error::Cancelled,
        Error::Exited(exit_status()),
        Error::Spawn {
            source: std::io::Error::other("no such file"),
        },
        Error::Timeout(PathBuf::from("/tmp/herdr.sock")),
    ]
}
