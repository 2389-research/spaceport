// ABOUTME: Verifies workspace termination against an explicitly selected real Herdr TUI.
// ABOUTME: Preserves unrelated workspaces, checkout files, and the test-owned daemon.
use super::*;

#[test]
#[ignore = "requires explicit HERDR_TEST_BINARY; closes only a test-created workspace"]
fn workspace_close_preserves_files_and_other_workspaces() {
    let mut daemon = Daemon::start();
    let mut session = Session::open(&daemon);
    let initial = session.snapshot.as_ref().unwrap();
    let boot = initial.boot_id.clone();
    let surviving: Vec<_> = initial
        .workspaces
        .iter()
        .map(|workspace| workspace.workspace_id.clone())
        .collect();
    let checkout = daemon.sandbox.dir.join("workspace-close-fixture");
    fs::create_dir(&checkout).unwrap();
    let marker = checkout.join("keep.txt");
    fs::write(&marker, "keep this checkout\n").unwrap();
    let request = session
        .client
        .handle
        .request(
            &boot,
            Method::WorkspaceCreate,
            json!({
                "cwd": checkout, "focus": true, "label": "workspace-close-fixture"
            }),
        )
        .unwrap();
    let created = session.response(request);
    let workspace = created["workspace"]["workspace_id"]
        .as_str()
        .unwrap()
        .to_owned();
    if !session
        .snapshot
        .as_ref()
        .unwrap()
        .workspaces
        .iter()
        .any(|candidate| candidate.workspace_id == workspace)
    {
        session.until("created workspace", |event| matches!(event, ClientEvent::Snapshot(snapshot) if snapshot.workspaces.iter().any(|candidate| candidate.workspace_id == workspace)));
    }
    let request = session
        .client
        .handle
        .request(
            &boot,
            Method::WorkspaceClose,
            json!({
                "workspace_id": workspace, "close_group": false
            }),
        )
        .unwrap();
    session.response(request);
    if session
        .snapshot
        .as_ref()
        .unwrap()
        .workspaces
        .iter()
        .any(|candidate| candidate.workspace_id == workspace)
    {
        session.until("closed workspace", |event| matches!(event, ClientEvent::Snapshot(snapshot) if snapshot.workspaces.iter().all(|candidate| candidate.workspace_id != workspace)));
    }
    assert_eq!(fs::read_to_string(marker).unwrap(), "keep this checkout\n");
    let remaining = session.snapshot.as_ref().unwrap();
    assert!(surviving.iter().all(|id| {
        remaining
            .workspaces
            .iter()
            .any(|candidate| candidate.workspace_id == *id)
    }));
    session.detach();
    assert!(daemon.child.as_mut().unwrap().try_wait().unwrap().is_none());
    let reattached = Session::open(&daemon);
    assert_eq!(reattached.snapshot.as_ref().unwrap().boot_id, boot);
    assert!(
        reattached
            .snapshot
            .as_ref()
            .unwrap()
            .workspaces
            .iter()
            .all(|candidate| candidate.workspace_id != workspace)
    );
    reattached.detach();
}
