// ABOUTME: Checks Windows state discovery across desktop and Git Bash environments.
// ABOUTME: Exercises the actual root selector without reading personal environment values.
use super::*;

fn root(vars: &[(&str, &str)]) -> Option<PathBuf> {
    state_root_for(
        |name| {
            vars.iter()
                .find(|(key, _)| *key == name)
                .map(|(_, value)| (*value).into())
        },
        true,
    )
}

#[test]
fn windows_local_app_data_wins_over_a_shell_home() {
    assert_eq!(
        root(&[
            ("HOME", "shell-home"),
            ("LOCALAPPDATA", "local"),
            ("USERPROFILE", "profile")
        ]),
        Some(PathBuf::from("local"))
    );
}

#[test]
fn windows_profile_fallback_wins_over_a_shell_home() {
    assert_eq!(
        root(&[
            ("HOME", "shell-home"),
            ("LOCALAPPDATA", ""),
            ("USERPROFILE", "profile")
        ]),
        Some(PathBuf::from("profile").join("AppData/Local"))
    );
}

#[test]
fn explicit_windows_state_root_stays_authoritative() {
    assert_eq!(
        root(&[
            ("XDG_STATE_HOME", "explicit"),
            ("HOME", "shell-home"),
            ("LOCALAPPDATA", "local")
        ]),
        Some(PathBuf::from("explicit"))
    );
}
