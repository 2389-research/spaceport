// ABOUTME: Navigates the actual Settings window through scoped keyboard events.
// ABOUTME: Keeps native composition focused while Tab reaches navigation controls.
use super::*;

#[gpui::test]
fn tab_reaches_search_and_sections_and_enter_selects_the_page(cx: &mut TestAppContext) {
    let (view, cx) = cx.add_window_view(skill_fixture);
    cx.update(|window, cx| {
        view.update(cx, |view, cx| {
            view.section = Section::Notifications;
            window.focus(&view.focus, cx);
        });
        crate::sidebar::layout_tests::full_draw(window, cx).clear(cx);
    });
    cx.simulate_keystrokes("tab");
    assert!(
        cx.update(|window, cx| view.read(cx).nav_search.read(cx).focus.is_focused(window)),
        "Tab did not reach Settings search"
    );
    cx.simulate_keystrokes("tab");
    cx.update(|window, cx| crate::sidebar::layout_tests::full_draw(window, cx).clear(cx));
    cx.simulate_keystrokes("enter");
    cx.simulate_event(KeyUpEvent {
        keystroke: crate::keymap::parse_keystroke("enter").unwrap(),
    });
    assert_eq!(view.read_with(cx, |view, _| view.section), Section::General);
}

#[gpui::test]
fn tab_does_not_take_focus_from_a_composing_settings_field(cx: &mut TestAppContext) {
    use gpui::EntityInputHandler;
    let (view, cx) = cx.add_window_view(skill_fixture);
    cx.update(|window, cx| {
        let input = view.read(cx).nav_search.clone();
        input.update(cx, |input, cx| {
            input.replace_and_mark_text_in_range(None, "é", None, window, cx)
        });
        let focus = input.read(cx).focus.clone();
        window.focus(&focus, cx);
        crate::sidebar::layout_tests::full_draw(window, cx).clear(cx);
    });
    // This key belongs to the IME; do not manufacture a committed tab character.
    cx.simulate_event(KeyDownEvent {
        keystroke: crate::keymap::parse_keystroke("tab").unwrap(),
        is_held: false,
        prefer_character_input: false,
    });
    assert!(cx.update(|window, cx| {
        let input = view.read(cx).nav_search.read(cx);
        input.focus.is_focused(window) && input.is_composing()
    }));
}

#[gpui::test]
fn tab_reveals_controls_below_the_settings_viewport(cx: &mut TestAppContext) {
    let (view, cx) = cx.add_window_view(skill_fixture);
    cx.simulate_resize(size(px(960.), px(600.)));
    cx.update(|window, cx| {
        window.focus(&view.read(cx).focus.clone(), cx);
        crate::sidebar::layout_tests::full_draw(window, cx).clear(cx);
    });
    for _ in 0..128 {
        cx.simulate_keystrokes("tab");
        cx.update(|window, cx| crate::sidebar::layout_tests::full_draw(window, cx).clear(cx));
        let reached = cx.update(|window, _| {
            window.context_stack().iter().any(|context| {
                context
                    .get("settings-row")
                    .is_some_and(|id| id.as_ref() == "general-copied-toast")
            })
        });
        if !reached {
            continue;
        }
        let row = cx.debug_bounds("general-copied-toast").unwrap();
        let body = view.read_with(cx, |view, _| view.body_scroll.bounds());
        assert!(
            row.top() >= body.top() && row.bottom() <= body.bottom(),
            "focused row {row:?} outside {body:?}"
        );
        return;
    }
    panic!("Tab never reached the lower Settings control");
}

#[gpui::test]
fn tab_stays_inside_an_open_font_picker(cx: &mut TestAppContext) {
    let (view, cx) = cx.add_window_view(skill_fixture);
    cx.update(|window, cx| {
        view.update(cx, |view, cx| {
            view.section = Section::Fonts;
            view.open_control_font_picker(crate::font_picker::FontTarget::All, window, cx)
        });
        crate::sidebar::layout_tests::full_draw(window, cx).clear(cx);
    });
    assert!(
        cx.update(|window, _| window
            .context_stack()
            .iter()
            .any(|context| context.contains("SettingsFontPicker"))),
        "fixture did not render its picker"
    );
    for key in ["tab", "shift-tab", "tab", "tab"] {
        press_key(cx, key);
        cx.update(|window, cx| crate::sidebar::layout_tests::full_draw(window, cx).clear(cx));
        assert!(
            cx.update(|window, _| window
                .context_stack()
                .iter()
                .any(|context| context.contains("SettingsFontPicker"))),
            "{key} left the modal picker"
        );
    }
}

#[gpui::test]
fn dropdown_keyboard_dismissal_and_selection_restore_the_trigger(cx: &mut TestAppContext) {
    let (view, cx) = cx.add_window_view(skill_fixture);
    view.update(cx, |view, _| {
        view.controls.preference_io = Some(controls::preferences::PreferenceIo::recording(|_| {}));
    });
    cx.update(|window, cx| crate::sidebar::layout_tests::full_draw(window, cx).clear(cx));
    let trigger = cx
        .debug_bounds("general-language-control")
        .unwrap()
        .center();
    cx.simulate_click(trigger, Default::default());
    let previous = view.read_with(cx, |view, _| {
        view.dropdown
            .as_ref()
            .unwrap()
            .previous_focus()
            .unwrap()
            .clone()
    });
    press_key(cx, "escape");
    assert!(view.read_with(cx, |view, _| view.dropdown.is_none()));
    assert!(cx.update(|window, _| previous.is_focused(window)));
    cx.update(|window, cx| crate::sidebar::layout_tests::full_draw(window, cx).clear(cx));
    press_key(cx, "enter");
    assert!(view.read_with(cx, |view, _| view.dropdown.is_some()));
    cx.update(|window, cx| crate::sidebar::layout_tests::full_draw(window, cx).clear(cx));
    press_key(cx, "enter");
    assert!(
        view.read_with(cx, |view, _| view.dropdown.is_none()),
        "selection reopened the list"
    );
    assert!(cx.update(|window, _| previous.is_focused(window)));
}

#[gpui::test]
fn an_invalid_number_keeps_its_focus_and_text_when_tab_is_pressed(cx: &mut TestAppContext) {
    let (view, cx) = cx.add_window_view(skill_fixture);
    let input = cx.update(|window, cx| {
        view.update(cx, |view, cx| {
            view.section = Section::Notifications;
            view.begin_number_edit(inputs::NumberField::ToastDelay, 1., window, cx);
            let input = view.number_edit.as_ref().unwrap().input.clone();
            input.update(cx, |input, cx| input.set_text_selected("invalid", cx));
            input
        })
    });
    cx.update(|window, cx| crate::sidebar::layout_tests::full_draw(window, cx).clear(cx));
    press_key(cx, "tab");
    assert!(cx.update(|window, cx| input.read(cx).focus.is_focused(window)));
    assert_eq!(
        input.read_with(cx, |input, _| input.text().to_owned()),
        "invalid"
    );
    assert!(view.read_with(cx, |view, _| view.number_edit.as_ref().unwrap().invalid));
}
