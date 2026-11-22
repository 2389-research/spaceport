// ABOUTME: Keeps desktop-owned chat conversion out of the command catalog.
// ABOUTME: Ordinary terminal commands remain available.
use super::*;

#[test]
fn removed_chat_lifecycle_command_is_absent() {
    assert!(!COMMANDS.iter().any(|info| info.name == "convert_to_chat"));
    assert!(serde_json::from_str::<Command>(r#""convert_to_chat""#).is_err());
    for name in ["new_core_terminal", "mirror_core_pane"] {
        assert!(COMMANDS.iter().any(|info| info.name == name), "{name}");
    }
}
