// ABOUTME: Checks retained Herdr CLI pane and process response decoding.
// ABOUTME: Keeps unknown wire fields and statuses from inventing a settled agent.
#![allow(clippy::unwrap_used)]

use super::*;

#[test]
fn decodes_the_cli_snapshot_envelope_and_ignores_unknown_fields() {
    let json = r#"{"id":"cli:api:snapshot","result":{"type":"session_snapshot","snapshot":{
        "version":"0.9.1","protocol":22,"future":true,
        "workspaces":[],"tabs":[],"layouts":[],
        "panes":[{"pane_id":"w1:p1","tab_id":"w1:t1","cwd":"/w/feat","agent":"claude",
          "agent_session":{"agent":"claude","kind":"id","source":"herdr:claude","value":"abc"}}],
        "agents":[]}}}"#;
    let snapshot = serde_json::from_str::<Envelope<SnapshotResult>>(json)
        .unwrap()
        .result
        .snapshot;
    let pane = snapshot.pane("w1:p1").unwrap();
    assert_eq!(pane.tab_id, "w1:t1");
    assert_eq!(pane.agent_session.as_ref().unwrap().kind, SessionKind::Id);
    assert_eq!(pane.status, PaneStatus::Other);
    assert!(snapshot.pane("w2:p1").is_none());
}

#[test]
fn unknown_or_absent_status_does_not_mean_idle() {
    for (status, expected) in [
        ("done", PaneStatus::Done),
        ("idle", PaneStatus::Idle),
        ("working", PaneStatus::Working),
        ("waiting", PaneStatus::Other),
        ("future", PaneStatus::Other),
    ] {
        let pane: Pane = serde_json::from_value(serde_json::json!({
            "pane_id": "p1", "tab_id": "t1", "agent_status": status,
        }))
        .unwrap();
        assert_eq!(pane.status, expected);
    }
}

#[test]
fn process_response_keeps_the_foreground_and_shell_identities() {
    let info = serde_json::from_str::<Envelope<ProcessInfoResult>>(
        r#"{"result":{"process_info":{"foreground_process_group_id":9,"shell_pid":7,
            "foreground_processes":[{"pid":12,"argv":["node","x"]},{"pid":9,"argv":["npm","run","dev"]}]}}}"#,
    ).unwrap().result.process_info;
    assert_eq!(info.foreground_process_group_id, Some(9));
    assert_eq!(info.shell_pid, Some(7));
    assert_eq!(info.foreground_processes[1].argv, ["npm", "run", "dev"]);
    let info: ProcessInfo = serde_json::from_str("{}").unwrap();
    assert_eq!(info.foreground_process_group_id, None);
    assert!(info.foreground_processes.is_empty());
}

#[test]
fn an_idle_shell_has_no_foreground_pids() {
    let info: ProcessInfo = serde_json::from_str(
        r#"{"foreground_process_group_id":7,"shell_pid":7,
            "foreground_processes":[{"pid":7,"argv":["-zsh"],"cwd":"/w"}]}"#,
    )
    .unwrap();
    assert!(info.foreground_pids().is_empty());
    let info: ProcessInfo = serde_json::from_str(
        r#"{"foreground_process_group_id":9,"shell_pid":7,
            "foreground_processes":[{"pid":12,"argv":["node","x"]},{"pid":9,"argv":["npm","run","dev"]}]}"#,
    ).unwrap();
    assert_eq!(info.foreground_pids(), [12, 9]);
    assert!(ProcessInfo::default().foreground_pids().is_empty());
}
