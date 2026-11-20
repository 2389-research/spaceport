// ABOUTME: Checks stable application state names from compiled build metadata.
// ABOUTME: Separates development profiles and worktrees while retaining preview paths.
use super::*;

#[test]
fn preview_versions_keep_the_installed_state_namespace() {
    assert_eq!(
        state_namespace(true, "release", None).as_deref(),
        Some("spaceport")
    );
    assert_eq!(
        state_namespace(true, "debug", Some("/src/preview")).as_deref(),
        Some("spaceport")
    );
}

#[test]
fn development_profiles_and_worktrees_have_distinct_stable_names() {
    let main = state_namespace(false, "debug", None).unwrap();
    let release = state_namespace(false, "release", None).unwrap();
    let first = state_namespace(false, "debug", Some("/src/worktrees/first")).unwrap();
    let second = state_namespace(false, "debug", Some("/src/worktrees/second")).unwrap();
    assert_ne!(main, "spaceport");
    assert_ne!(main, release);
    assert_ne!(main, first);
    assert_ne!(first, second);
    assert_eq!(
        Some(first),
        state_namespace(false, "debug", Some("/src/worktrees/first"))
    );
}

#[test]
fn state_names_cannot_escape_the_selected_root() {
    for invalid in ["", "..", "/tmp", "a/b", r"a\b", "a.b", "a b"] {
        assert!(state_dir_for(invalid).is_none(), "{invalid}");
        assert!(state_namespace(false, invalid, None).is_none(), "{invalid}");
    }
    assert!(state_namespace(false, "debug", Some("")).is_none());
}
