use super::*;

#[test]
fn direct_browser_page_command_and_shortcut_are_absent() {
    assert!(COMMANDS.iter().all(|info| info.name != "new_browser_tab"));
    assert!(serde_json::from_str::<Command>("\"NewBrowserTab\"").is_err());
    assert!(
        COMMANDS
            .iter()
            .flat_map(|info| info.base_shortcuts())
            .all(|shortcut| *shortcut != "cmd-shift-b")
    );
    assert!(
        COMMANDS
            .iter()
            .any(|info| info.command == Command::NewCoreTerminal)
    );
}
