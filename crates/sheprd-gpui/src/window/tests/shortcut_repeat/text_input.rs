// ABOUTME: Protects preferred character input from raw direct shortcuts.
// ABOUTME: Exercises the production shortcut scope around a real focused text field.
use super::*;
use gpui::{
    AppContext as _, Context, Entity, IntoElement, KeyDownEvent, Keystroke, ParentElement as _,
    PlatformInput, Render, TestAppContext, VisualTestContext, Window,
};

struct TextWindow {
    input: Entity<crate::search_input::SearchInput>,
    view: Entity<SheprdWindow>,
}

impl Render for TextWindow {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.view
            .update(cx, |_, cx| SheprdWindow::shortcut_scope(cx))
            .child(self.input.clone())
    }
}

fn text_window(window: &mut Window, cx: &mut Context<TextWindow>) -> TextWindow {
    let view = cx.new(|cx| fixture_window(window, cx));
    let input = cx.new(crate::search_input::SearchInput::new);
    window.focus(&input.read(cx).focus.clone(), cx);
    TextWindow { input, view }
}

fn preferred(cx: &mut VisualTestContext, key: &str, held: bool) -> bool {
    let result = cx.update(|window, cx| {
        window.dispatch_event(
            PlatformInput::KeyDown(KeyDownEvent {
                keystroke: Keystroke {
                    key_char: Some("ñ".into()),
                    ..crate::keymap::parse_keystroke(key).unwrap()
                },
                is_held: held,
                prefer_character_input: true,
            }),
            cx,
        )
    });
    if result.propagate {
        cx.simulate_input("ñ");
    }
    result.propagate
}

#[gpui::test]
fn preferred_text_bypasses_direct_catalog_and_custom_shortcuts(cx: &mut TestAppContext) {
    for custom in [false, true] {
        cx.update(crate::actions::bind_keys);
        let (fixture, cx) = cx.add_window_view(text_window);
        let (view, input) = fixture.read_with(cx, |fixture, _| {
            (fixture.view.clone(), fixture.input.clone())
        });
        view.update(cx, |view, _| {
            view.keymap = crate::keymap::Keymap::with_overrides(
                &[(
                    "new_window".into(),
                    crate::keymap::Binding::One("ctrl-alt-n".into()),
                )]
                .into(),
                &Default::default(),
                &crate::keymap::DaemonKeys::default(),
            )
            .unwrap();
            view.live.status = crate::state::ConnectionStatus::Connected;
            Arc::make_mut(view.live.snapshot.as_mut().unwrap()).commands =
                vec![sheprd_client::protocol::ClientShellCommand {
                    command_id: "custom.text".into(),
                    description: None,
                    action: sheprd_client::protocol::ClientShellCommandAction::Shell,
                    binding_label: "ctrl+alt+y".into(),
                    binding_labels: vec!["ctrl+alt+y".into()],
                }];
        });
        cx.update(|window, cx| window.draw(cx).clear(cx));
        let before = cx.update(|_, cx| main_windows(cx).len());
        assert!(preferred(
            cx,
            if custom { "ctrl-alt-y" } else { "ctrl-alt-n" },
            false
        ));
        assert_eq!(cx.update(|_, cx| main_windows(cx).len()), before);
        input.read_with(cx, |input, _| assert_eq!(input.text(), "ñ"));
    }
}

#[gpui::test]
fn preferred_text_yields_even_after_a_direct_repeatable_shortcut(cx: &mut TestAppContext) {
    let (fixture, cx) = cx.add_window_view(text_window);
    let (view, input) = fixture.read_with(cx, |fixture, _| {
        (fixture.view.clone(), fixture.input.clone())
    });
    view.update(cx, |view, _| {
        view.keymap = crate::keymap::Keymap::with_overrides(
            &[(
                "increase_font_size".into(),
                crate::keymap::Binding::One("ctrl-alt-n".into()),
            )]
            .into(),
            &Default::default(),
            &crate::keymap::DaemonKeys::default(),
        )
        .unwrap();
    });
    cx.update(|window, cx| window.draw(cx).clear(cx));
    press(cx, "ctrl-alt-n", false);
    let size = view.read_with(cx, |view, _| view.config.terminal.size);
    assert!(preferred(cx, "ctrl-alt-n", true));
    assert_eq!(
        view.read_with(cx, |view, _| view.config.terminal.size),
        size
    );
    input.read_with(cx, |input, _| assert_eq!(input.text(), "ñ"));
}

#[gpui::test]
fn an_explicit_prefix_keeps_its_chord_when_character_input_is_preferred(cx: &mut TestAppContext) {
    let (fixture, cx) = cx.add_window_view(text_window);
    let (view, input) = fixture.read_with(cx, |fixture, _| {
        (fixture.view.clone(), fixture.input.clone())
    });
    cx.update(|window, cx| window.draw(cx).clear(cx));
    press(cx, "ctrl-b", false);
    assert!(!preferred(cx, "b", false));
    assert!(!view.read_with(cx, |view, _| view.sidebar_visible));
    assert!(!preferred(cx, "b", true));
    input.read_with(cx, |input, _| assert_eq!(input.text(), ""));
}

#[gpui::test]
fn resize_mode_owns_a_previously_captured_direct_repeat(cx: &mut TestAppContext) {
    let (fixture, cx) = cx.add_window_view(text_window);
    let view = fixture.read_with(cx, |fixture, _| fixture.view.clone());
    view.update(cx, |view, _| {
        view.keymap = crate::keymap::Keymap::with_overrides(
            &[(
                "increase_font_size".into(),
                crate::keymap::Binding::One("ctrl-alt-n".into()),
            )]
            .into(),
            &Default::default(),
            &crate::keymap::DaemonKeys::default(),
        )
        .unwrap();
    });
    cx.update(|window, cx| window.draw(cx).clear(cx));
    press(cx, "ctrl-alt-n", false);
    let size = view.read_with(cx, |view, _| view.config.terminal.size);
    view.update_in(cx, |view, window, cx| {
        view.command(Command::ResizeMode, window, cx)
    });
    assert!(view.read_with(cx, |view, _| view.resize_mode));
    assert!(press(cx, "ctrl-alt-n", true));
    assert_eq!(
        view.read_with(cx, |view, _| view.config.terminal.size),
        size
    );
}
