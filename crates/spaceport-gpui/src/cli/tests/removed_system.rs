// ABOUTME: Checks that removed host-resource commands never start the app.
// ABOUTME: Keeps retained process inspection and service commands available.
use super::*;

#[test]
fn resource_dashboard_command_is_rejected_before_startup() {
    for args in [
        vec!["system"],
        vec!["system", "--json"],
        vec!["system", "--help"],
    ] {
        assert_eq!(
            LaunchOptions::parse(args).unwrap_err(),
            CliError::UnknownOption("system".into())
        );
    }
}
