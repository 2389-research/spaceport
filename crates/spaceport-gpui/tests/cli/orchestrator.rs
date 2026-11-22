//! The desktop executable refuses the retired orchestration command family.
//! The retained Devices command still uses its private control target.

use super::*;

#[test]
fn orchestrator_cli_refuses_before_contacting_the_private_app_socket() {
    let parent = if cfg!(unix) {
        std::path::PathBuf::from("/tmp")
    } else {
        std::env::temp_dir()
    };
    let state = tempfile::Builder::new()
        .prefix("spc")
        .tempdir_in(parent)
        .unwrap();

    for args in [
        &["orchestrator"][..],
        &["orchestrator", "open", "--repo", "/src/app"],
        &["orchestrator", "tasks", "--json", "--window", "7"],
        &["orchestrator", "--help"],
    ] {
        let output = cli_in(args, Some(state.path()));
        assert_eq!(output.status.code(), Some(2), "{args:?}: {output:?}");
        assert!(output.stdout.is_empty(), "{args:?}: {output:?}");
        assert!(
            String::from_utf8_lossy(&output.stderr).contains("Unknown option: orchestrator"),
            "{args:?}: {output:?}"
        );
    }

    let help = cli_in(&["--help"], Some(state.path()));
    assert!(help.status.success(), "{help:?}");
    assert!(!String::from_utf8_lossy(&help.stdout).contains("orchestrator"));

    let retained = cli_in(&["devices", "--json", "--window", "7"], Some(state.path()));
    assert_eq!(retained.status.code(), Some(3), "{retained:?}");
    let error: serde_json::Value = serde_json::from_slice(&retained.stdout).unwrap();
    assert_eq!(error["error"]["code"], "not_running");

    #[cfg(unix)]
    assert!(!state.path().join("spaceport/control.sock").exists());
}
