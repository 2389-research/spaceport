#![allow(clippy::unwrap_used)]

use super::*;
use serde_json::json;

#[test]
fn tab_response_names_both_tab_and_pane() {
    assert_eq!(
        created_tab(&json!({"result":{"type":"tab_created","tab":{"tab_id":"t2"},"root_pane":{"pane_id":"p3"}}})).unwrap(),
        Some(("t2".into(), "p3".into()))
    );
    assert_eq!(
        created_tab(&json!({"result":{"type":"tab_created","tab":{"tab_id":""},"root_pane":{"pane_id":"p3"}}})).unwrap(),
        None
    );
    assert_eq!(
        created_tab(&json!({"result":{"type":"workspace_created"}})).unwrap(),
        None
    );
}

#[test]
fn tab_response_keeps_daemon_refusal() {
    assert!(matches!(
        created_tab(&json!({"error":{"code":"denied","message":"no"}})),
        Err(crate::Error::Daemon(crate::daemon::Error::Response(_)))
    ));
}
