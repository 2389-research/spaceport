// ABOUTME: Exercises normalized layout bindings through GPUI's real matching value types.
// ABOUTME: Proves portable mapper delegation and configured ownership without claiming native OS coverage.
use super::*;
use gpui::KeybindingKeystroke;

fn keys(entries: &[(&str, &str)]) -> Keymap {
    Keymap::with_overrides(
        &entries
            .iter()
            .map(|(name, key)| ((*name).into(), crate::keymap::Binding::One((*key).into())))
            .collect(),
        &Default::default(),
        &DaemonKeys::default(),
    )
    .unwrap()
}

/// A representative shifted layout result; the actual app supplies its platform mapper.
fn shifted_number(mut bound: Keystroke) -> KeybindingKeystroke {
    if bound.key == "1" && bound.modifiers.shift {
        bound.key = "!".into();
        bound.modifiers.shift = false;
    }
    KeybindingKeystroke::from_keystroke(bound)
}

#[test]
fn a_mapped_direct_binding_matches_the_native_key_representation() {
    let keymap = keys(&[("focus_tab_1", "ctrl-shift-1")]);
    let typed = Keystroke::parse("ctrl-!").unwrap();
    assert!(typed.should_match(&shifted_number(Keystroke::parse("ctrl-shift-1").unwrap())));
    assert_eq!(
        keymap.direct(&typed, shifted_number),
        Some(crate::keymap::Direct::Configured(
            crate::controls::Command::TabNumber(1)
        ))
    );
}

#[test]
fn a_canonical_named_binding_keeps_priority_over_another_mapped_binding() {
    let keymap = keys(&[
        ("focus_tab_1", "ctrl-shift-1"),
        ("toggle_sidebar", "ctrl-!"),
    ]);
    assert_eq!(
        keymap.direct(&Keystroke::parse("ctrl-!").unwrap(), shifted_number),
        Some(crate::keymap::Direct::Configured(
            crate::controls::Command::ToggleSidebar
        ))
    );
}

#[gpui::test]
fn a_mapped_configured_binding_keeps_its_key_from_a_daemon_custom_symbol(cx: &mut TestAppContext) {
    let (view, cx) = cx.add_window_view(crate::sidebar::layout_tests::fixture_window);
    view.update(cx, |view, _| {
        view.config.keybindings = keys(&[("focus_tab_1", "ctrl-shift-1")]);
        view.live.status = ConnectionStatus::Connected;
        Arc::make_mut(view.live.snapshot.as_mut().unwrap()).commands = vec![ClientShellCommand {
            command_id: "custom.symbol".into(),
            description: None,
            action: ClientShellCommandAction::Shell,
            binding_label: "ctrl+!".into(),
            binding_labels: vec!["ctrl+!".into()],
        }];
        let typed = Keystroke::parse("ctrl-!").unwrap();
        assert!(typed.should_match(&shifted_number(Keystroke::parse("ctrl-shift-1").unwrap())));
        assert!(matches!(
            view.bound(&typed, false, false, shifted_number),
            Some(crate::window::prefix::Bound::Command(
                crate::controls::Command::TabNumber(1)
            ))
        ));
    });
}
