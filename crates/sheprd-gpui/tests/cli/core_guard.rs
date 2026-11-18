// ABOUTME: Runs Core-origin control commands without touching any live runtime.
// ABOUTME: Refuses implicit window targets and unsupported legacy workspace mutations.
use super::*;

fn from_core(args: &[&str]) -> Output {
    warm_up();
    let state = tempfile::tempdir().unwrap();
    let child = Command::new(env!("CARGO_BIN_EXE_spaceport"))
        .args(args)
        .env("XDG_STATE_HOME", state.path())
        .env("HC_TERMINAL_ID", "t7")
        .env("HC_TERMINAL_SOCKET", state.path().join("core.sock"))
        .env_remove("HERDR_PANE_ID")
        .env_remove("HERDR_WORKSPACE_ID")
        .env_remove("HERDR_SOCKET_PATH")
        .env_remove("HERDR_CLIENT_SOCKET_PATH")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    wait(child, args, COMMAND_LIMIT)
}

#[test]
fn core_panes_cannot_implicitly_close_a_frontmost_tab() {
    let output = from_core(&["tab", "close", "99", "--json"]);
    assert_eq!(output.status.code(), Some(5), "{output:?}");
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["error"]["code"], "no_window");
}

#[test]
fn core_notes_refuse_instead_of_targeting_a_legacy_checkout() {
    for args in [
        &["notes", "--json", "set", "fixture"][..],
        &["notes", "--window", "7", "--json", "set", "fixture"],
    ] {
        let output = from_core(args);
        assert_eq!(output.status.code(), Some(8), "{output:?}");
        let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(json["error"]["code"], "unsupported");
    }
}

#[test]
fn core_panes_can_inspect_app_targets_and_choose_an_explicit_window() {
    for args in [
        &["status", "--json"][..],
        &["window", "list", "--json"],
        &["tab", "close", "99", "--window", "7", "--json"],
    ] {
        let output = from_core(args);
        assert_eq!(output.status.code(), Some(3), "{output:?}");
    }
}

#[test]
fn core_panes_cannot_infer_a_chat_pane_from_window_focus() {
    let output = from_core(&["chat", "show", "--window", "7", "--json"]);
    assert_eq!(output.status.code(), Some(8), "{output:?}");
}
