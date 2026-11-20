// ABOUTME: Profiles an explicitly selected private Herdr TUI session in the release GUI.
// ABOUTME: Separates process CPU measurement semantics from native workload qualification.
#![cfg(all(any(target_os = "macos", windows), feature = "integration-test"))]

#[cfg(target_os = "macos")]
#[path = "working_profile/mod.rs"]
mod working_profile;

#[cfg(windows)]
#[path = "working_profile/windows_contracts.rs"]
mod windows_contracts;

#[cfg(target_os = "macos")]
#[test]
#[ignore = "requires release macOS GUI and explicit HERDR_TEST_BINARY; owns a private TUI server"]
fn working_terminal_groups_profile() -> anyhow::Result<()> {
    working_profile::run()
}
