//! The Spaceport control endpoint has a distinct name for each user and
//! explicit state root on Windows.
use super::*;

#[test]
fn windows_pipe_name_is_independent_of_upstream() {
    let first = windows_pipe_name("S-1-5-21-1", None);
    assert_eq!(
        first,
        PathBuf::from(r"\\.\pipe\spaceport-control-S-1-5-21-1")
    );
    assert_ne!(first, windows_pipe_name("S-1-5-21-2", None));
    assert_ne!(
        first,
        PathBuf::from(r"\\.\pipe\herdr-gpui-control-S-1-5-21-1")
    );
    let isolated = windows_pipe_name("S-1-5-21-1", Some(Path::new(r"C:\state\a")));
    assert_ne!(first, isolated);
    assert_eq!(
        isolated,
        windows_pipe_name("S-1-5-21-1", Some(Path::new("c:/STATE/a/")))
    );
}

#[test]
fn development_pipes_are_scoped_without_changing_preview_defaults() {
    let root = Path::new("state");
    let preview = root.join("spaceport");
    let development = root.join("spaceport-development-debug");
    assert_eq!(windows_state_scope(&preview, false), None);
    assert_eq!(windows_state_scope(&preview, true), Some(preview.as_path()));
    assert_eq!(
        windows_state_scope(&development, false),
        Some(development.as_path())
    );
    assert_ne!(
        windows_pipe_name("S-1-5-21-1", windows_state_scope(&preview, false)),
        windows_pipe_name("S-1-5-21-1", windows_state_scope(&development, false)),
    );
}
