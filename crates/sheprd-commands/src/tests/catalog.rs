use super::*;
use std::collections::HashSet;

/// What the config file and the key bindings rely on, whatever the table
/// holds: a name or a default shortcut picks one command, and a command has
/// one row.
#[test]
fn catalog_names_commands_and_base_shortcuts_are_unique() {
    let mut commands = HashSet::new();
    let mut names = HashSet::new();
    let mut shortcuts = HashSet::new();
    for info in COMMANDS {
        assert!(commands.insert(info.command), "{:?}", info.command);
        assert!(!info.name.is_empty(), "{:?}", info.command);
        assert!(names.insert(info.name), "{}", info.name);
        assert!(info.message.starts_with("commands-"), "{}", info.message);
        for shortcut in info.base_shortcuts() {
            assert!(shortcuts.insert(shortcut), "{shortcut}");
        }
    }
}

#[test]
fn every_catalog_command_deserializes_from_its_variant_name() {
    for info in COMMANDS {
        let command = info.command;
        let value = match command {
            Command::TabNumber(number) => json!({"TabNumber": number}),
            Command::WorkspaceNumber(number) => json!({"WorkspaceNumber": number}),
            Command::AgentNumber(number) => json!({"AgentNumber": number}),
            _ => json!(format!("{command:?}")),
        };
        assert_eq!(serde_json::from_value::<Command>(value).unwrap(), command);
    }
}

/// A numbered row names each of its commands and shortcuts by the number.
#[test]
fn numbered_commands_end_their_name_and_shortcuts_with_their_number() {
    let numbered: Vec<_> = COMMANDS
        .iter()
        .filter_map(|info| Some((info, info.command.number()?.to_string())))
        .collect();
    assert!(!numbered.is_empty());
    for (info, number) in numbered {
        assert!(info.name.ends_with(&format!("_{number}")), "{}", info.name);
        for shortcut in info.base_shortcuts() {
            assert!(shortcut.ends_with(&format!("-{number}")), "{shortcut}");
        }
    }
}

/// The commands `request` turns into a daemon call. Every other command in
/// the catalog stays in the window, including ones added later.
fn has_daemon_request(command: Command) -> bool {
    use Command::*;
    matches!(
        command,
        Workspace
            | Tab
            | SplitRight
            | SplitDown
            | NextTab
            | PreviousTab
            | MoveTabPrevious
            | MoveTabNext
            | FocusLeft
            | FocusRight
            | FocusUp
            | FocusDown
            | NextPane
            | PreviousPane
            | SwapLeft
            | SwapRight
            | SwapUp
            | SwapDown
            | ResizeLeft
            | ResizeRight
            | ResizeUp
            | ResizeDown
            | Zoom
            | ClearPane
            | EditScrollback
            | ClosePane
            | CloseTab
            | TabNumber(_)
    )
}

#[test]
fn gui_commands_never_send_daemon_requests() {
    let s = snapshot();
    let mut window_commands = 0;
    for info in COMMANDS
        .iter()
        .filter(|info| !has_daemon_request(info.command))
    {
        assert!(request(info.command, &s).is_none(), "{:?}", info.command);
        window_commands += 1;
    }
    assert!(window_commands > 0);
}
