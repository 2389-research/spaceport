//! A main checkout whose shell moved into another repository. Herdr keeps the
//! repository the workspace was first given while its label follows the
//! shell, as after a Teleport opened it on one repository and its shell later
//! moved to another. Worktree requests then name the directory the workspace
//! is in, so they branch the repository its label names.

use super::*;
use crate::menu::{
    workspace::Source,
    worktree_shortcut::{Unavailable, WorktreeShortcut},
};

const MOVED: &str = if cfg!(windows) {
    "C:/fixture/wgsextract-cli"
} else {
    "/fixture/wgsextract-cli"
};

/// The fixture's main checkout, moved to a sibling repository's directory.
fn moved() -> sheprd_client::protocol::ClientShellSnapshot {
    let mut snapshot = sidebar::layout_tests::snapshot(7);
    snapshot.workspaces[3].new_workspace_cwd = MOVED.into();
    snapshot.workspaces[3].label = "wgsextract-cli".into();
    snapshot.workspaces[3].branch = Some("main".into());
    snapshot.focused_workspace_id = Some("w3".into());
    snapshot
}

#[test]
fn checkout_paths_follow_the_endpoint_shape() {
    use crate::worktree::left_checkout;
    let key = "/home/mac/src/herdr-gpui/.git";
    for cwd in [
        "/home/mac/src/herdr-gpui",
        "/home/mac/src/herdr-gpui/",
        "/home/mac/src/herdr-gpui/crates/sheprd-gpui",
    ] {
        assert!(!left_checkout(key, cwd), "{cwd}");
    }
    for cwd in [
        "/home/mac/src/wgsextract-cli",
        "/home/mac/src/herdr-gpui-other",
        "/home/mac",
    ] {
        assert!(left_checkout(key, cwd), "{cwd}");
    }
    // A Windows endpoint may report either separator and any case.
    let windows = r"C:\Users\mac\src\herdr-gpui\.git";
    assert!(!left_checkout(
        windows,
        "c:/users/mac/src/HERDR-GPUI/crates"
    ));
    assert!(left_checkout(windows, r"C:\Users\mac\src\wgsextract-cli"));
    // Nothing is known to have moved without a directory or a `<root>/.git` key.
    assert!(!left_checkout(key, ""));
    assert!(!left_checkout("/srv/git/herdr-gpui.git", "/home/mac"));
    assert!(!left_checkout("/.git", "/home/mac"));
}

/// A moved workspace opens checkouts from its current directory.
#[test]
fn a_moved_main_checkout_opens_from_the_directory_it_is_in() {
    let snapshot = moved();
    let target =
        WorkspaceTarget::for_linked_repository(&snapshot, &snapshot.workspaces[3]).unwrap();
    assert_eq!(target.source, Source::Directory(MOVED.into()));
    assert_eq!(target.worktree, None);
    assert!(target.can_create());
    assert_eq!(
        target
            .request(&snapshot, WorkspaceAction::OpenWorktree, "/elsewhere")
            .unwrap(),
        (
            Method::WorktreeOpen,
            serde_json::json!({"cwd": MOVED, "path": "/elsewhere", "focus": true,
                "trust_repository": false})
        )
    );
    assert_eq!(
        target.list_params(),
        serde_json::json!({"cwd": MOVED, "trust_repository": false})
    );
    // Its shortcut opens the dialog rather than refusing.
    assert_eq!(WorktreeShortcut::Open.target(&snapshot), Ok("w3".into()));
}

/// A workspace that stays in its repository still names itself.
#[test]
fn a_main_checkout_in_its_repository_names_its_workspace() {
    let snapshot = sidebar::layout_tests::snapshot(7);
    let target = WorkspaceTarget::new(&snapshot, &snapshot.workspaces[3]);
    assert_eq!(target.source, Source::Workspace);
    assert_eq!(
        target.list_params(),
        serde_json::json!({"workspace_id": "w3", "trust_repository": false})
    );
}

/// A dialog opened before the shell moved, or after it moved again, refuses
/// rather than opening a checkout in a different repository.
#[test]
fn a_main_checkout_that_moves_after_the_dialog_opened_refuses() {
    let before = sidebar::layout_tests::snapshot(7);
    let after = moved();
    let target = WorkspaceTarget::for_linked_repository(&before, &before.workspaces[3]).unwrap();
    let moved_target =
        WorkspaceTarget::for_linked_repository(&after, &after.workspaces[3]).unwrap();
    let mut again = moved();
    again.workspaces[3].new_workspace_cwd = format!("{MOVED}-again");
    for action in [WorkspaceAction::OpenWorktree] {
        for (target, snapshot) in [
            (&target, &after),
            (&moved_target, &before),
            (&moved_target, &again),
        ] {
            assert!(matches!(
                target.request(snapshot, action, "branch"),
                Err(crate::Error::WorkspaceRepositoryChanged)
            ));
        }
    }
}

/// A linked checkout still names its repository. When the only main checkout
/// open for it has wandered off, the repository's own root is named, since
/// the daemon refuses a linked checkout as the source.
#[test]
fn a_linked_checkout_uses_its_repository_root() {
    let snapshot = moved();
    let target =
        WorkspaceTarget::for_linked_repository(&snapshot, &snapshot.workspaces[4]).unwrap();
    let root = sidebar::layout_tests::REPO_KEY.trim_end_matches("/.git");
    assert_eq!(
        target.source,
        Source::Repository {
            root: root.into(),
            key: sidebar::layout_tests::REPO_KEY.into(),
        }
    );
    // Opening a checkout goes through the linked checkout rather than the
    // moved main checkout, whose own menu would list the directory it moved
    // to; its dialog then lists and opens by the repository's root.
    let mut focused = snapshot.clone();
    focused.focused_workspace_id = Some("w4".into());
    assert_eq!(WorktreeShortcut::Open.target(&focused), Ok("w4".into()));
    assert_eq!(
        target.list_params(),
        serde_json::json!({"cwd": root, "trust_repository": false})
    );
    assert_eq!(
        target
            .request(&snapshot, WorkspaceAction::OpenWorktree, "/elsewhere")
            .unwrap()
            .1,
        serde_json::json!({"cwd": root, "path": "/elsewhere", "focus": true,
            "trust_repository": false})
    );
    // With no main checkout at all, there is still nothing to open through.
    let mut closed = focused.clone();
    closed.workspaces.remove(3);
    assert_eq!(
        WorktreeShortcut::Open.target(&closed),
        Err(Unavailable::MainCheckoutClosed)
    );
    // A main checkout still in the repository is preferred over a moved one.
    let mut both = focused;
    let mut stayed = sidebar::layout_tests::snapshot(7).workspaces[3].clone();
    stayed.workspace_id = "w9".into();
    both.workspaces.push(stayed);
    assert_eq!(WorktreeShortcut::Open.target(&both), Ok("w9".into()));
    let target = WorkspaceTarget::for_linked_repository(&both, &both.workspaces[4]).unwrap();
    assert_eq!(
        (target.id.as_str(), &target.source),
        ("w9", &Source::Workspace)
    );
}

/// A main checkout moved to a directory outside any repository reports no
/// branch. A linked checkout routed through it still opens worktrees of its
/// repository until the main checkout is kept for another.
#[test]
fn a_linked_checkout_opens_while_its_main_checkout_is_outside_git() {
    let mut snapshot = moved();
    snapshot.workspaces[3].branch = None;
    let target =
        WorkspaceTarget::for_linked_repository(&snapshot, &snapshot.workspaces[4]).unwrap();
    assert!(matches!(target.source, Source::Repository { .. }));
    assert!(target.can_create());
    assert!(
        target
            .request(&snapshot, WorkspaceAction::OpenWorktree, "/elsewhere")
            .is_ok()
    );
    let mut rekeyed = snapshot;
    if let Some(tree) = rekeyed.workspaces[3].worktree.as_mut() {
        tree.key = "/fixture/other/.git".into();
    }
    assert!(matches!(
        target.request(&rekeyed, WorkspaceAction::OpenWorktree, "/elsewhere"),
        Err(crate::Error::WorkspaceRepositoryChanged)
    ));
}
