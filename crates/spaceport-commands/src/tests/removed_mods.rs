use super::*;

#[test]
fn client_mod_panel_command_is_absent() {
    assert!(!COMMANDS.iter().any(|info| info.name == "toggle_mod_panel"));
}
