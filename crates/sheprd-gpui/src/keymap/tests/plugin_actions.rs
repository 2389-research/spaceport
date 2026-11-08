use super::*;
use sheprd_client::protocol::ClientShellCommandAction;

#[test]
fn plugin_action_cannot_take_a_custom_shortcut() {
    let mut plugin = custom("plugin.audit", &["ctrl+alt+y", "prefix+y"]);
    plugin.action = ClientShellCommandAction::PluginAction;
    let shell = custom("shell.audit", &["ctrl+alt+y", "prefix+y"]);
    let commands = [plugin, shell];
    for (key, prefixed) in [("ctrl-alt-y", false), ("y", true)] {
        let chosen = Keymap::default()
            .custom_command(&commands, &keystroke(key), prefixed)
            .map(|command| command.command_id.as_str());
        assert_eq!(chosen, Some("shell.audit"), "{key}");
    }
}
