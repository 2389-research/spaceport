// ABOUTME: Exercises the actual setup helper's terminal-preparation refusal.
// ABOUTME: Requires branded diagnostics and shell status before any GUI startup.
use super::*;

#[test]
fn setup_without_a_terminal_names_spaceport() {
    let output = cli(&["__device-setup-session", "/nonexistent/spaceport-setup"]);
    assert_eq!(output.status.code(), Some(127), "{output:?}");
    assert!(output.stdout.is_empty(), "{output:?}");
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(
        stderr.starts_with("Spaceport could not prepare the setup terminal:"),
        "{stderr}"
    );
    assert!(!stderr.contains("Sheprd"), "{stderr}");
}
