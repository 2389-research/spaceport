// ABOUTME: Checks that built-in agent discovery trusts only absolute search roots.
// ABOUTME: Rejects PATH entries that silently grant a checkout executable authority.
use super::*;

#[test]
fn search_paths_discard_relative_entries_and_relative_home() {
    let absolute = std::env::temp_dir();
    let path =
        std::env::join_paths([Path::new("."), Path::new(""), Path::new("bin"), &absolute]).unwrap();
    let searched = search_path(Some(&path), Some(Path::new("home")));
    let dirs: Vec<_> = std::env::split_paths(&searched).collect();
    assert!(dirs.iter().all(|dir| dir.is_absolute()), "{dirs:?}");
    assert!(dirs.contains(&absolute));
}

#[test]
fn missing_agent_commands_never_fall_back_to_os_path_search() {
    let error = command(Agent::Opencode, None).unwrap_err();
    assert_eq!(error.kind(), io::ErrorKind::NotFound);
}

#[cfg(windows)]
#[test]
fn custom_commands_preserve_the_requested_windows_extension() {
    let root = tempfile::tempdir().unwrap();
    for name in ["fixture.cmd", "fixture.exe"] {
        std::fs::write(root.path().join(name), "fixture").unwrap();
    }
    assert_eq!(
        custom_program(OsStr::new("fixture.cmd"), Some(root.path().as_os_str())).unwrap(),
        root.path().join("fixture.cmd").into_os_string(),
    );
}

#[cfg(unix)]
#[test]
fn custom_explicit_paths_preserve_non_utf8_names() {
    use std::os::unix::ffi::OsStringExt;
    let program = OsString::from_vec(b"./fixture-\xff".to_vec());
    assert_eq!(custom_program(&program, None).unwrap(), program);
}

#[cfg(target_os = "linux")]
#[test]
fn custom_command_lookup_preserves_non_utf8_names() {
    use std::os::unix::{ffi::OsStringExt, fs::PermissionsExt};
    let root = tempfile::tempdir().unwrap();
    let name = OsString::from_vec(b"fixture-\xff".to_vec());
    let program = root.path().join(&name);
    std::fs::write(&program, "#!/bin/sh\nexit 0\n").unwrap();
    std::fs::set_permissions(&program, std::fs::Permissions::from_mode(0o700)).unwrap();
    assert_eq!(
        custom_program(&name, Some(root.path().as_os_str())).unwrap(),
        program.into_os_string()
    );
}
