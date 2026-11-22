#![allow(clippy::unwrap_used, clippy::expect_used)]
use super::*;

#[test]
fn notes_name_their_repository_by_its_directory() {
    assert_eq!(repository_name("/src/app/.git"), "app");
    assert_eq!(repository_name("/src/app/.git/"), "app");
    assert_eq!(repository_name("/srv/repos/tool.git"), "tool.git");
    assert_eq!(repository_name("/"), "/");
    assert_eq!(
        describe_note(&NoteInfo {
            host: "local".into(),
            repository: "/src/app/.git".into(),
            branch: "feature".into(),
            text: "waiting on review".into(),
        }),
        "feature (app, local): waiting on review"
    );
}

#[test]
fn a_mute_says_what_silences_the_worktree() {
    let mute = |muted, host_muted| MuteInfo {
        host: "studio".into(),
        workspace: "herdr".into(),
        muted,
        host_muted,
    };
    assert_eq!(describe_mute(&mute(true, false)), "herdr is muted");
    assert_eq!(describe_mute(&mute(false, false)), "herdr is not muted");
    assert_eq!(
        describe_mute(&mute(false, true)),
        "herdr is silent because its host studio is muted"
    );
    assert_eq!(
        describe_mute(&mute(true, true)),
        "herdr is muted, and so is its host studio"
    );
}
