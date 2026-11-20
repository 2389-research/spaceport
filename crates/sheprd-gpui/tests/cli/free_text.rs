// ABOUTME: Exercises free-text parsing through the real CLI executable.
// ABOUTME: Uses private socket state so notes cannot reach a running app.
use super::*;

#[test]
fn free_text_reaches_command_execution_without_triggering_options() {
    let state = tempfile::Builder::new()
        .prefix("spt")
        .tempdir_in(std::env::temp_dir())
        .unwrap();
    for args in [
        &["notes", "set", "rerun", "with", "-h"][..],
        &["notes", "set", "fix", "--force", "push"],
        &["notes", "set", "keep", "--json"],
        &["notes", "set", "--", "--help", "--json"],
    ] {
        let output = cli_in(args, Some(state.path()));
        assert_eq!(output.status.code(), Some(3), "{args:?}: {output:?}");
        assert!(output.stdout.is_empty(), "{args:?}: {output:?}");
        assert!(
            String::from_utf8_lossy(&output.stderr).contains("Spaceport is not running"),
            "{args:?}: {output:?}"
        );
    }
}
