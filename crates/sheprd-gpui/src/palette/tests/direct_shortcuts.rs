// ABOUTME: Checks that a direct shortcut on a palette key leaves that key to the palette.
// ABOUTME: Drives the real window keystroke path with a configured direct binding.
use super::*;
use crate::keymap::{DaemonKeys, Keymap};
use gpui::VisualTestContext;

#[gpui::test]
fn a_direct_shortcut_on_a_palette_key_moves_the_palette(cx: &mut TestAppContext) {
    let (view, cx) = cx.add_window_view(fixture_window);
    let overrides = [(
        "toggle_sidebar".to_owned(),
        crate::keymap::Binding::One("ctrl-n".into()),
    )]
    .into();
    view.update_in(cx, |view, window, cx| {
        view.keymap =
            Keymap::with_overrides(&overrides, &Default::default(), &DaemonKeys::default())
                .unwrap();
        view.open_palette(Filter::All, window, cx);
    });
    cx.run_until_parked();
    let state = |cx: &mut VisualTestContext| {
        view.read_with(cx, |view, _| {
            let palette = view.menu.palette.as_ref().unwrap();
            (
                palette.selected,
                palette.filtered.len(),
                view.sidebar_visible,
            )
        })
    };
    let (selected, count, visible) = state(cx);
    assert!(count > 1, "the fixture palette needs rows to move through");
    cx.update(|window, cx| {
        window.dispatch_keystroke(crate::keymap::parse_keystroke("ctrl-n").unwrap(), cx)
    });
    cx.run_until_parked();
    assert_eq!(state(cx), ((selected + 1) % count, count, visible));
}
