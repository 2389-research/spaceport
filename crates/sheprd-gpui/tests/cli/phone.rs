//! The desktop executable refuses retired phone commands before app startup.
//! Its retained local command catalog and missing-app status use private state.

use super::*;

#[test]
fn phone_refusal_keeps_local_cli_available_in_private_state() {
    let parent = if cfg!(unix) {
        std::path::PathBuf::from("/tmp")
    } else {
        std::env::temp_dir()
    };
    let state = tempfile::Builder::new()
        .prefix("spc")
        .tempdir_in(parent)
        .unwrap();

    let removed = cli_in(&["remote", "status"], Some(state.path()));
    assert_eq!(removed.status.code(), Some(2), "{removed:?}");
    assert!(removed.stdout.is_empty(), "{removed:?}");
    assert!(
        String::from_utf8_lossy(&removed.stderr).contains("Unknown option: remote"),
        "{removed:?}"
    );

    let retained = cli_in(&["commands", "--json"], Some(state.path()));
    assert!(retained.status.success(), "{retained:?}");
    let commands: serde_json::Value = serde_json::from_slice(&retained.stdout).unwrap();
    assert!(
        commands
            .as_array()
            .unwrap()
            .iter()
            .any(|command| command["name"] == "toggle_sidebar")
    );
    assert!(retained.stderr.is_empty(), "{retained:?}");

    let missing = cli_in(&["status", "--json"], Some(state.path()));
    assert_eq!(missing.status.code(), Some(3), "{missing:?}");
    let error: serde_json::Value = serde_json::from_slice(&missing.stdout).unwrap();
    assert_eq!(error["error"]["code"], "not_running");
    assert!(missing.stderr.is_empty(), "{missing:?}");

    #[cfg(unix)]
    assert!(!state.path().join("spaceport/control.sock").exists());
}
