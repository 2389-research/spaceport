// ABOUTME: Exercises retired updater arguments through the desktop executable.
// ABOUTME: Usage errors leave isolated state empty and keep informational CLI modes usable.
use super::*;

#[test]
fn updater_commands_and_helpers_exit_before_app_startup() {
    let state = tempfile::tempdir().unwrap();
    for args in [
        vec!["update"],
        vec!["update", "status"],
        vec!["update", "check"],
        vec!["--herdr-apply-update"],
        vec!["--herdr-install-package"],
        vec!["--herdr-install-package-root"],
        vec!["--herdr-relaunch-package"],
    ] {
        let output = cli_in(&args, Some(state.path()));
        assert_eq!(output.status.code(), Some(2), "{args:?}: {output:?}");
        assert!(output.stdout.is_empty(), "{args:?}: {output:?}");
        assert!(
            String::from_utf8_lossy(&output.stderr).contains("Unknown option:"),
            "{args:?}: {output:?}"
        );
    }
    let help = cli_in(&["--help"], Some(state.path()));
    assert!(help.status.success(), "{help:?}");
    assert!(!String::from_utf8_lossy(&help.stdout).contains("Check for an app update"));
    let info = cli_in(&["--build-info"], Some(state.path()));
    assert!(info.status.success(), "{info:?}");
    assert!(String::from_utf8_lossy(&info.stdout).contains("worktree="));
    assert!(std::fs::read_dir(state.path()).unwrap().next().is_none());
}
