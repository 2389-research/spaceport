use super::*;

#[gpui::test]
fn plugin_actions_are_absent_from_palette_but_other_commands_remain(cx: &mut TestAppContext) {
    let (view, cx) = cx.add_window_view(fixture_window);
    view.update(cx, |view, _| {
        Arc::make_mut(view.live.snapshot.as_mut().unwrap()).commands = [
            ("plugin.audit", ClientShellCommandAction::PluginAction),
            ("shell.audit", ClientShellCommandAction::Shell),
        ]
        .into_iter()
        .map(|(command_id, action)| ClientShellCommand {
            command_id: command_id.into(),
            action,
            description: None,
            binding_label: "ctrl+alt+y".into(),
            binding_labels: vec!["ctrl+alt+y".into()],
        })
        .collect();
    });
    cx.update(|window, cx| {
        view.update(cx, |view, cx| {
            view.open_palette(Filter::Commands, window, cx)
        })
    });
    view.read_with(cx, |view, _| {
        let entries = &view.menu.palette.as_ref().unwrap().entries;
        assert!(!entries.iter().any(
            |entry| matches!(&entry.action, Action::Configured(id, _) if id == "plugin.audit")
        ));
        assert!(entries.iter().any(
            |entry| matches!(&entry.action, Action::Configured(id, _) if id == "shell.audit")
        ));
        assert!(
            entries
                .iter()
                .any(|entry| matches!(entry.action, Action::Native(_)))
        );
    });
}

#[test]
fn plugin_action_cannot_be_invoked_from_a_stale_palette_entry() {
    let mut snapshot = snapshot();
    snapshot.commands = vec![ClientShellCommand {
        command_id: "plugin.audit".into(),
        action: ClientShellCommandAction::PluginAction,
        description: None,
        binding_label: String::new(),
        binding_labels: Vec::new(),
    }];
    let target = Target::capture(&snapshot);
    assert!(
        target
            .invocation(
                &snapshot,
                "plugin.audit",
                ClientShellCommandAction::PluginAction
            )
            .is_err()
    );
}
