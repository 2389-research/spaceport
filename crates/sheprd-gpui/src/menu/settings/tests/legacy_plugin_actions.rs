use super::custom_shortcuts::{command, draw, install};
use sheprd_client::protocol::ClientShellCommandAction;

#[gpui::test]
fn keybinds_hide_plugin_actions_but_keep_other_commands(cx: &mut gpui::TestAppContext) {
    let (view, cx) = cx.add_window_view(crate::sidebar::layout_tests::fixture_window);
    install(
        &view,
        vec![
            command(
                "plugin.audit",
                Some("Plugin audit"),
                ClientShellCommandAction::PluginAction,
                &["ctrl+alt+y"],
            ),
            command(
                "shell.audit",
                Some("Shell audit"),
                ClientShellCommandAction::Shell,
                &["ctrl+alt+z"],
            ),
        ],
        cx,
    );
    cx.update(|window, cx| view.update(cx, |view, cx| view.open_keybinds(window, cx)));
    draw(cx);
    assert!(cx.debug_bounds("shortcut-Plugin audit").is_none());
    assert!(cx.debug_bounds("shortcut-Shell audit").is_some());
    assert!(cx.debug_bounds("shortcut-New Workspace").is_some());
}
