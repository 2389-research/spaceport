// ABOUTME: Rejects custom updater commands before application startup.
// ABOUTME: Keeps informational CLI modes independent of the removed engine.
use super::*;

#[test]
fn removed_updater_commands_are_usage_errors() {
    for args in [
        vec!["update"],
        vec!["update", "check"],
        vec!["update", "status"],
        vec!["--herdr-apply-update"],
        vec!["--herdr-install-package"],
        vec!["--herdr-install-package-root"],
        vec!["--herdr-relaunch-package"],
    ] {
        assert!(LaunchOptions::parse(args.clone()).is_err(), "{args:?}");
    }
}
