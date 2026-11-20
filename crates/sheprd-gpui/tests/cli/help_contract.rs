// ABOUTME: Checks help and skill output through the compiled executable.
// ABOUTME: Keeps informational commands local and delivers requested JSON documents.
use super::*;

#[test]
fn top_level_help_includes_shared_options_and_exit_codes() {
    let output = cli(&["--help"]);
    assert!(output.status.success(), "{output:?}");
    let text = String::from_utf8(output.stdout).unwrap();
    assert!(text.contains("Exit status:"), "{text}");
    assert!(
        text.contains("--json") && text.contains("--window"),
        "{text}"
    );
}

#[test]
fn help_and_skill_honor_json_output() {
    for args in [
        &["help", "--json"][..],
        &["--help", "--json"],
        &["--json", "--help"],
        &["tab", "--help", "--json"],
        &["skill", "--json"],
    ] {
        let output = cli(args);
        assert!(output.status.success(), "{args:?}: {output:?}");
        assert!(
            serde_json::from_slice::<serde_json::Value>(&output.stdout).is_ok(),
            "{args:?}: {output:?}"
        );
    }
}

#[test]
fn completion_scripts_are_available_without_an_app() {
    let state = tempfile::tempdir().unwrap();
    for shell in ["bash", "zsh", "fish"] {
        let output = cli_in(&["completions", shell, "--json"], Some(state.path()));
        assert!(output.status.success(), "{shell}: {output:?}");
        let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(value["shell"], shell);
        assert!(
            value["script"]
                .as_str()
                .is_some_and(|script| !script.is_empty())
        );
    }
}

#[test]
fn a_named_window_never_falls_back_to_untargeted_local_settings() {
    warm_up();
    for args in [
        &["language", "status", "--window", "7", "--json"][..],
        &["config", "check", "--window", "7", "--json"],
        &["theme", "--window", "7", "--json", "set", "0x96f"],
    ] {
        let root = tempfile::tempdir().unwrap();
        let child = Command::new(env!("CARGO_BIN_EXE_spaceport"))
            .args(args)
            .env("HOME", root.path())
            .env("XDG_CONFIG_HOME", root.path().join("config"))
            .env("XDG_STATE_HOME", root.path().join("state"))
            .env_remove("HERDR_SOCKET_PATH")
            .env_remove("HERDR_CLIENT_SOCKET_PATH")
            .env_remove("HERDR_PANE_ID")
            .env_remove("HERDR_WORKSPACE_ID")
            .env_remove("HC_TERMINAL_ID")
            .env_remove("HC_TERMINAL_SOCKET")
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        let output = wait(child, args, COMMAND_LIMIT);
        assert_eq!(output.status.code(), Some(3), "{args:?}: {output:?}");
        if args[0] == "theme" {
            assert!(
                !root.path().join("config").exists(),
                "named-window refusal wrote local settings"
            );
        }
    }
}
