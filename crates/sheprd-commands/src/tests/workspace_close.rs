// ABOUTME: Exposes workspace closing through the shared app command catalog.
// ABOUTME: Keeps termination behind the app's captured confirmation rather than direct RPC.
use super::*;

#[test]
fn workspace_closing_is_an_app_command_that_requires_confirmation() {
    let info = COMMANDS
        .iter()
        .find(|info| info.name == "close_workspace")
        .expect("workspace closing must be discoverable in commands and the palette");
    assert!(request(info.command, &snapshot()).is_none());
    assert!(info.base_shortcuts().is_empty());
}
