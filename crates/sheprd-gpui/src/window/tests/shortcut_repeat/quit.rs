// ABOUTME: Preserves app-level Quit through real main-window shortcut dispatch.
// ABOUTME: Keeps popup input isolation and held-shortcut suppression intact.
use super::*;
use std::{cell::Cell, rc::Rc};

#[gpui::test]
fn quit_reaches_the_app_with_a_popup_and_consumes_its_held_tail(cx: &mut gpui::TestAppContext) {
    let quits = Rc::new(Cell::new(0));
    let observed = quits.clone();
    cx.update(|cx| {
        crate::actions::bind_keys(cx);
        cx.on_action(move |_: &crate::actions::Quit, _| observed.set(observed.get() + 1));
    });
    let (view, cx) = cx.add_window_view(fixture_window);
    view.update_in(cx, |view, window, cx| view.open_keybinds(window, cx));
    cx.run_until_parked();
    cx.update(|window, cx| window.draw(cx).clear(cx));
    let (quit, toggle, visible) = view.read_with(cx, |view, _| {
        assert!(view.menu.page.is_some());
        (
            view.keymap().primary(Command::Quit).to_owned(),
            view.keymap().primary(Command::ToggleSidebar).to_owned(),
            view.sidebar_visible,
        )
    });
    // Other shortcuts go on to the popup's field and run nothing behind it.
    press(cx, &toggle, false);
    assert_eq!(view.read_with(cx, |view, _| view.sidebar_visible), visible);
    assert!(press(cx, &quit, false));
    assert_eq!(
        quits.get(),
        1,
        "popup Quit must reach the app action handler"
    );
    assert!(view.read_with(cx, |view, _| view.menu.page.is_some()));
    assert!(press(cx, &quit, true));
    assert_eq!(quits.get(), 1, "held Quit must not dispatch again");
    assert!(press(cx, &quit, false));
    assert_eq!(quits.get(), 2, "a fresh Quit press must still dispatch");
}
