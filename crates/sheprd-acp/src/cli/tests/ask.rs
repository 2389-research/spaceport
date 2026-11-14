// ABOUTME: Tests the restricted app-owned helper argument boundary.
// ABOUTME: Refuses arbitrary agent commands and incomplete identities.
use super::*;

#[test]
fn ask_requires_owned_paths_and_exact_identity() {
    assert!(
        parse(args(&[
            "ask",
            "--state-dir",
            "/tmp/ask",
            "--endpoint",
            "0123456789abcdef0123456789abcdef",
            "--broker",
            "/tmp/control.sock",
            "--conversation",
            "12345678-1234-1234-1234-123456789abc"
        ]))
        .is_ok()
    );
    assert!(parse(args(&["ask", "--state-dir", "/tmp/ask"])).is_err());
}

#[test]
fn ask_refuses_commands_adapters_and_extra_agent_arguments() {
    for extra in ["--", "--acp", "--load", "--cwd", "claude"] {
        assert!(
            parse(args(&[
                "ask",
                "--state-dir",
                "/tmp/ask",
                "--endpoint",
                "0123456789abcdef0123456789abcdef",
                "--broker",
                "/tmp/control.sock",
                "--conversation",
                "12345678-1234-1234-1234-123456789abc",
                extra
            ]))
            .is_err()
        );
    }
}
