// ABOUTME: Profiles explicitly selected private terminal runtimes in the release GUI.
// ABOUTME: Separates process CPU measurement semantics from native workload qualification.
#![cfg(all(any(target_os = "macos", windows), feature = "integration-test"))]

#[path = "working_profile/mod.rs"]
mod working_profile;

#[test]
#[ignore = "requires release native GUI and explicit HERDR_TEST_BINARY; owns a private TUI server"]
fn working_terminal_groups_profile() -> anyhow::Result<()> {
    working_profile::run()
}

#[cfg(windows)]
#[test]
#[ignore = "requires release Windows GUI, explicit Herdr/Core binaries, and recorder tools; owns private runtimes"]
fn working_tui_and_core_profile() -> anyhow::Result<()> {
    working_profile::run_core()
}
