use super::*;
use crate::{
    git::{Counted, Git, Status},
    sidebar::workspaces::workspace_badge,
};
use std::time::Instant;

#[test]
fn local_counts_keep_dirty_and_mute_marks_without_a_remote_probe() {
    let snapshot = layout_tests::snapshot(6);
    let child = &snapshot.workspaces[4];
    let now = Instant::now();
    let mut git = Git::default();
    git.seed_counts(
        crate::pull_request::Input {
            checkout: None,
            repo_key: Some(layout_tests::REPO_KEY.into()),
            branch: "worktree/sidebar-child".into(),
        },
        Counted {
            lines: Status {
                additions: 42,
                deletions: 7,
                untracked: 0,
            },
            dirty: true,
        },
        now,
    );
    let local = workspace_badge(child, Some(&git), (Some("note"), false)).unwrap();
    let counts = local.counts.unwrap();
    assert_eq!(
        (counts.additions.as_str(), counts.deletions.as_str()),
        ("+42", "-7")
    );
    assert!(local.dirty);
    assert!(local.noted);
    assert!(workspace_badge(child, None, (None, false)).is_none());
    assert!(workspace_badge(child, None, (None, true)).unwrap().muted);
}
