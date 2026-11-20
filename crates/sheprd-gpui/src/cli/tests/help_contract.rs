// ABOUTME: Checks help and local command options through the real launch parser.
// ABOUTME: Keeps JSON formatting explicit and rejects window options with no target.
use super::*;

#[test]
fn help_is_a_control_entry_point_and_keeps_json() {
    assert!(matches!(
        LaunchOptions::parse(["help"]).unwrap().mode,
        LaunchMode::Control(control::Invocation {
            command: control::ControlCommand::Help(None),
            ..
        })
    ));
    let LaunchMode::Control(invocation) = LaunchOptions::parse(["tab", "--help", "--json"])
        .unwrap()
        .mode
    else {
        panic!("not help")
    };
    assert!(invocation.json);
}

#[test]
fn local_commands_refuse_a_window_they_cannot_use() {
    for args in [
        &["skill", "--window", "7"][..],
        &["commands", "--window", "7"],
        &["logs", "--window", "7"],
        &["processes", "--window", "7"],
        &["config", "get", "theme", "--window", "7"],
        &["theme", "list", "--window", "7"],
        &["language", "list", "--window", "7"],
        &["sound", "list", "--window", "7"],
    ] {
        assert!(
            LaunchOptions::parse(args.iter().copied()).is_err(),
            "{args:?}"
        );
    }
}

#[test]
fn completion_scripts_have_an_explicit_supported_shell() {
    for shell in ["bash", "zsh", "fish"] {
        assert!(
            LaunchOptions::parse(["completions", shell]).is_ok(),
            "{shell}"
        );
    }
    assert!(LaunchOptions::parse(["completions", "unknown"]).is_err());
}
