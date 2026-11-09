use super::*;

#[test]
fn git_menu_is_absent_but_workspace_and_terminal_commands_remain() {
    assert!(COMMANDS.iter().all(|info| info.name != "git_menu"));
    assert!(serde_json::from_str::<Command>("\"GitMenu\"").is_err());
    assert!(
        COMMANDS
            .iter()
            .any(|info| info.command == Command::Workspace)
    );
    assert!(
        COMMANDS
            .iter()
            .any(|info| info.command == Command::NewCoreTerminal)
    );
}
