// ABOUTME: Checks that direct shortcuts leave an open menu page's keys to the page.
// ABOUTME: Covers a dialog's text field; palette/tests/direct_shortcuts covers its list keys.
use super::*;

/// Binds `command` directly to `key`, as the GUI's own `[keybindings]` does.
fn bind_directly(
    command: &str,
    key: &str,
    view: &gpui::Entity<crate::SheprdWindow>,
    cx: &mut VisualTestContext,
) {
    let overrides = [(command.to_owned(), crate::keymap::Binding::One(key.into()))].into();
    view.update(cx, |view, _| {
        view.keymap =
            Keymap::with_overrides(&overrides, &Default::default(), &DaemonKeys::default())
                .unwrap();
    });
    cx.update(|window, cx| {
        window.focus(&view.read(cx).focus.clone(), cx);
        window.draw(cx).clear(cx);
    });
}

#[gpui::test]
fn a_direct_shortcut_typed_in_a_menu_field_reaches_the_field(cx: &mut TestAppContext) {
    let (view, cx) = cx.add_window_view(crate::sidebar::layout_tests::fixture_window);
    // Select All, which a field acts on, bound to a window command instead.
    let [.., select_all] = crate::actions::EDIT_KEYS;
    bind_directly("toggle_sidebar", select_all, &view, cx);
    let sidebar = |cx: &mut VisualTestContext| view.read_with(cx, |view, _| view.sidebar_visible);
    let visible = sidebar(cx);
    assert!(press(select_all, cx));
    assert_eq!(
        sidebar(cx),
        !visible,
        "the direct shortcut runs from the terminal"
    );

    view.update_in(cx, |view, window, cx| view.open_keybinds(window, cx));
    cx.run_until_parked();
    let search = |cx: &mut VisualTestContext| {
        view.read_with(cx, |view, cx| {
            view.menu
                .keybinds_search
                .as_ref()
                .map(|search| search.read(cx).text().to_owned())
        })
    };
    press("a", cx);
    press("b", cx);
    assert_eq!(search(cx).as_deref(), Some("ab"));
    let visible = sidebar(cx);
    press(select_all, cx);
    press("c", cx);
    assert_eq!(
        search(cx).as_deref(),
        Some("c"),
        "the field's own Select All never ran"
    );
    assert_eq!(sidebar(cx), visible);
    view.read_with(cx, |view, _| assert!(view.menu.page.is_some()));
}
