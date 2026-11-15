//! A failed port scan is shown with its host-shell cause, in words about
//! the shell rather than plan usage, which the shell no longer reads.
use super::*;
use sheprd_usage::shell::{Error as Shell, START};

/// What the window keeps for a scan that failed with `cause`: the scan's
/// message followed by its localized causes.
fn shown(cause: Shell) -> String {
    let mut reading = Reading::default();
    reading.apply(Err(super::super::Error::Scan(Box::new(cause.into())).into()));
    reading.error.unwrap()
}

#[test]
fn a_failed_scan_names_the_host_shell_not_usage() {
    let failures = [
        Shell::Timeout,
        Shell::Unreachable,
        Shell::Size,
        Shell::Process {
            operation: START,
            source: std::io::Error::other("no such file"),
        },
    ];
    for cause in failures {
        let text = shown(cause);
        assert!(text.starts_with("Could not read listening ports"), "{text}");
        assert!(!text.to_lowercase().contains("usage"), "{text}");
    }
}
