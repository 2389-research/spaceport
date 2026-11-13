// ABOUTME: Keeps local preferences in the shared native and in-window menu definitions.
// ABOUTME: Checks real menu actions against the application command catalog.
use super::*;

#[test]
fn application_menu_without_profile_keeps_local_controls() {
    let menus = menus(&Config::default());
    let actions: Vec<_> = menus[0]
        .items
        .iter()
        .filter_map(|item| match item {
            MenuItem::Action { action, .. } => Some(action),
            _ => None,
        })
        .collect();
    for command in [
        Command::Settings,
        Command::Keybinds,
        Command::Themes,
        Command::About,
    ] {
        assert!(
            actions
                .iter()
                .any(|action| action.partial_eq(&RunCommand { command })),
            "{command:?}"
        );
    }
}
