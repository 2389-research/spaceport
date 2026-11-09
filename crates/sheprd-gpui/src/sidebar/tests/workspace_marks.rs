use super::*;
use crate::sidebar::workspaces::workspace_badge;

#[test]
fn workspace_badges_keep_notes_and_mutes_without_git_marks() {
    let snapshot = layout_tests::snapshot(1);
    assert_eq!(workspace_label(&snapshot.workspaces[0], false), "herdr");
    assert!(workspace_badge((None, false)).is_none());
    let note = workspace_badge((Some("review later"), false)).unwrap();
    assert!(note.noted);
    assert!(!note.muted);
    let muted = workspace_badge((None, true)).unwrap();
    assert!(!muted.noted);
    assert!(muted.muted);
}
