// ABOUTME: Checks installed agent discovery and ordered executable search paths.
// ABOUTME: Uses native absolute fixture paths on every supported platform.
use super::*;

#[cfg(unix)]
#[test]
fn only_agents_with_their_own_command_are_installed() {
    let dir = tempfile::tempdir().unwrap();
    // npx alone would only download Claude's and Codex's adapters.
    install(dir.path(), "npx");
    install(dir.path(), "opencode");
    install(dir.path(), "codex-acp");
    let path = dir.path().as_os_str();
    assert_eq!(installed(Some(path)), [Agent::Codex, Agent::Opencode]);
    assert!(installed(None).is_empty());
}

#[test]
fn the_search_path_adds_user_and_system_folders_after_path() {
    let root = tempfile::tempdir().unwrap();
    let home = root.path().join("home");
    let bin = root.path().join("bin");
    let input = std::env::join_paths([&bin]).unwrap();
    let path = search_path(Some(&input), Some(&home));
    let dirs: Vec<PathBuf> = std::env::split_paths(&path).collect();
    assert_eq!(dirs[0], bin);
    assert!(dirs.contains(&home.join(".local/bin")));
    assert!(dirs.contains(&home.join(".opencode/bin")));
    let without_home = search_path(None, None);
    assert!(std::env::split_paths(&without_home).all(|dir| !dir.starts_with(&home)));
}
