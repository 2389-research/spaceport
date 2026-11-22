// ABOUTME: Checks retired script commands stop at CLI parsing.
// ABOUTME: Protects the desktop from opening or running repository scripts.
use super::*;

#[test]
fn removed_script_commands_exit_before_starting_the_app() {
    for args in [
        &["scripts"][..],
        &["scripts", "list"],
        &["scripts", "setup"],
        &["scripts", "run"],
        &["scripts", "--help"],
    ] {
        usage_error(args, "Unknown option: scripts");
    }
    let help = cli(&["--help"]);
    assert!(help.status.success(), "{help:?}");
    assert!(!String::from_utf8_lossy(&help.stdout).contains("  scripts "));
}
