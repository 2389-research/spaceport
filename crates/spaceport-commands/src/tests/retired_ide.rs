// ABOUTME: Guards the command catalog against retired embedded IDE entries.
// ABOUTME: Checks that the embedded editor commands stay out of the catalog.
use super::*;

#[test]
fn vscode_has_no_native_command() {
    assert!(!COMMANDS.iter().any(|info| info.name == "open_code"));
    assert!(!COMMANDS.iter().any(|info| info.name == "open_px0"));
}

#[test]
fn source_navigation_has_no_native_command() {
    assert!(!COMMANDS.iter().any(|info| info.name == "go_to_file"));
    assert!(!COMMANDS.iter().any(|info| info.name == "go_to_symbol"));
    assert!(!COMMANDS.iter().any(|info| info.name == "open_px0"));
}
