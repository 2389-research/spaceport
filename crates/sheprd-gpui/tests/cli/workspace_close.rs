// ABOUTME: Checks workspace-close discovery through the real Spaceport executable.
// ABOUTME: Keeps CLI invocation from bypassing the running app's confirmation.
use super::*;

#[test]
fn workspace_close_is_discoverable_and_requires_the_app() {
    let state = tempfile::tempdir().unwrap();
    let output = cli_in(&["commands", "--json"], Some(state.path()));
    assert!(output.status.success(), "{output:?}");
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(
        value["commands"]
            .as_array()
            .unwrap()
            .iter()
            .any(|command| command["name"] == "close_workspace")
    );
    let output = cli_in(&["run", "close_workspace", "--json"], Some(state.path()));
    assert_eq!(output.status.code(), Some(3), "{output:?}");
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(value.get("error").is_some(), "{value}");
}
