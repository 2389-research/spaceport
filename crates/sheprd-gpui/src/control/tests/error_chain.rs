// ABOUTME: Checks control replies keep the cause of a local failure.
// ABOUTME: Protects diagnostics when a worker module is removed.
use super::*;

#[test]
fn control_error_includes_its_source() {
    let error = crate::Error::LocalGit(std::io::Error::other("git could not start"));
    assert!(describe_error(&error).contains(": git could not start"));
}
