use super::*;
use crate::{
    Error,
    page::WorkspaceAction,
    workspace::{WorkspaceTarget, chosen_label, suggested_tab_name, suggested_workspace_name},
};
use spaceport_client::Method;

#[test]
fn rename_trims_unicode_whitespace_and_rejects_blank_labels() {
    let snapshot = snapshot();
    let target = WorkspaceTarget::new(&snapshot, &snapshot.workspaces[1]);
    for text in ["", " \t\r\n", "\u{2003}\u{3000}"] {
        assert!(matches!(
            target.request(&snapshot, WorkspaceAction::Rename, text),
            Err(Error::EmptyWorkspaceLabel)
        ));
    }
    assert_eq!(
        target
            .request(
                &snapshot,
                WorkspaceAction::Rename,
                " \u{3000}new label\u{2003} "
            )
            .unwrap(),
        (
            Method::WorkspaceRename,
            json!({"workspace_id": "w1", "label": "new label"})
        )
    );
}

#[test]
fn new_tab_and_workspace_names_follow_herdr() {
    let mut snapshot = snapshot();
    let target = WorkspaceTarget::new(&snapshot, &snapshot.workspaces[1]);
    assert_eq!(
        target
            .request(&snapshot, WorkspaceAction::NewTab, "ignored")
            .unwrap(),
        (
            Method::TabCreate,
            json!({"workspace_id": "w1", "focus": true})
        )
    );
    assert_eq!(
        target
            .request(&snapshot, WorkspaceAction::NewWorkspace, "ignored")
            .unwrap(),
        (
            Method::WorkspaceCreate,
            json!({"focus": true, "source_workspace_id": "w1"})
        )
    );
    // A proposal left alone, or cleared, lets the daemon choose the name.
    assert_eq!(chosen_label("  Build  ", Some("2")), Some("Build"));
    assert_eq!(chosen_label(" 2 ", Some("2")), None);
    assert_eq!(chosen_label(" \t", Some("2")), None);
    assert_eq!(chosen_label("2", None), Some("2"));
    snapshot.workspaces.retain(|w| w.workspace_id != "w1");
    assert!(matches!(
        target.request(&snapshot, WorkspaceAction::NewTab, ""),
        Err(Error::StaleWorkspace)
    ));
}

#[test]
fn the_note_is_never_a_daemon_request() {
    let snapshot = snapshot();
    let target = WorkspaceTarget::new(&snapshot, &snapshot.workspaces[1]);
    assert!(matches!(
        target.request(&snapshot, WorkspaceAction::Note, "a note"),
        Err(Error::LocalWorktreeNote)
    ));
}

#[test]
fn proposed_names_match_herdr_without_touching_the_daemon_host() {
    let snapshot = snapshot();
    let tabs = snapshot
        .tabs
        .iter()
        .filter(|tab| tab.workspace_id == "w1")
        .count();
    assert_eq!(suggested_tab_name(&snapshot, "w1"), (tabs + 1).to_string());
    assert_eq!(suggested_tab_name(&snapshot, "missing"), "1");
    assert_eq!(suggested_workspace_name("/home/me/code/herdr"), "herdr");
    assert_eq!(suggested_workspace_name("/home/me/code/herdr/"), "herdr");
    assert_eq!(suggested_workspace_name("/"), "/");
    assert_eq!(suggested_workspace_name(""), "workspace");
}

#[test]
fn a_target_names_its_own_workspace_and_fences_another_boot() {
    let snapshot = snapshot();
    let target = WorkspaceTarget::new(&snapshot, &snapshot.workspaces[3]);
    assert_eq!(
        target
            .request(&snapshot, WorkspaceAction::Rename, "new label")
            .unwrap(),
        (
            Method::WorkspaceRename,
            json!({"workspace_id": "w3", "label": "new label"})
        )
    );
    // A stale snapshot is refused before any request names its workspace.
    let mut replaced = snapshot.clone();
    replaced.boot_id = "replacement-boot".into();
    assert!(matches!(
        target.request(&replaced, WorkspaceAction::Rename, "new label"),
        Err(Error::StaleWorkspace)
    ));
}

#[test]
fn a_main_checkout_heads_its_repositorys_linked_checkouts() {
    let snapshot = snapshot();
    let main = WorkspaceTarget::new(&snapshot, &snapshot.workspaces[1]);
    assert_eq!(main.group_key(), Some(REPO_KEY));
    // A linked checkout of the same repository heads nothing itself.
    let linked = WorkspaceTarget::new(&snapshot, &snapshot.workspaces[2]);
    assert_eq!(linked.group_key(), None);
    // A main checkout with no linked checkout open keeps its worktree but
    // folds nothing.
    let alone = WorkspaceTarget::new(&snapshot, &snapshot.workspaces[3]);
    assert_eq!(alone.group_key(), None);
    assert_eq!(
        alone.worktree.map(|tree| tree.key),
        Some("/srv/elsewhere/.git".to_owned())
    );
}

#[test]
fn a_main_checkout_whose_shell_left_the_repository_is_no_worktree() {
    let mut snapshot = snapshot();
    // The main checkout's shell now stands outside the repository Herdr keeps
    // for it, so the snapshot's key names no repository of its own.
    snapshot.workspaces[1].new_workspace_cwd = "/home/me".into();
    let target = WorkspaceTarget::new(&snapshot, &snapshot.workspaces[1]);
    assert!(target.worktree.is_none());
    assert_eq!(target.group_key(), None);
}
