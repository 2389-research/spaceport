// ABOUTME: Exercises Ask composer keys through the actual native input handler.
// ABOUTME: Preserves multiline drafts and modified Enter without submitting another turn.
use super::*;
use gpui::{Entity, VisualTestContext, size};

fn composer(cx: &mut TestAppContext) -> (Entity<View>, &mut VisualTestContext) {
    let (view, cx) =
        cx.add_window_view(|_, cx| View::new(Default::default(), Default::default(), cx));
    view.update(cx, |view, cx| {
        view.owner.update(cx, |owner, _| owner.configured = true);
        cx.notify();
    });
    cx.simulate_resize(size(px(680.), px(460.)));
    cx.update(|window, cx| {
        window.activate_window();
        view.update(cx, |view, cx| view.focus(window, cx));
    });
    for _ in 0..2 {
        cx.run_until_parked();
        cx.update(|window, cx| window.draw(cx).clear(cx));
    }
    let Some(field) = cx.debug_bounds("chat-field") else {
        panic!("mounted composer");
    };
    cx.simulate_click(field.center(), gpui::Modifiers::default());
    cx.update(|window, cx| {
        assert!(view.read(cx).owner.read(cx).configured);
        assert!(view.read(cx).input.read(cx).focus.is_focused(window));
        window.draw(cx).clear(cx);
    });
    (view, cx)
}

#[gpui::test]
fn shift_enter_inserts_a_newline_without_submitting(cx: &mut TestAppContext) {
    let (view, cx) = composer(cx);
    cx.simulate_input("First café");
    assert_eq!(
        view.read_with(cx, |view, cx| view.input.read(cx).text().to_owned()),
        "First café"
    );
    cx.simulate_keystrokes("shift-enter");
    cx.simulate_input("Second 🛸");
    view.read_with(cx, |view, cx| {
        assert_eq!(view.input.read(cx).text(), "First café\nSecond 🛸");
        assert!(!view.owner.read(cx).active);
        assert!(view.owner.read(cx).pending.is_none());
    });
}

#[gpui::test]
fn modified_enter_keeps_the_draft_without_submitting(cx: &mut TestAppContext) {
    let (view, cx) = composer(cx);
    cx.simulate_input("Keep my draft");
    assert_eq!(
        view.read_with(cx, |view, cx| view.input.read(cx).text().to_owned()),
        "Keep my draft"
    );
    cx.simulate_keystrokes("alt-enter");
    view.read_with(cx, |view, cx| {
        assert_eq!(view.input.read(cx).text(), "Keep my draft");
        assert!(view.owner.read(cx).problem.is_none());
        assert!(view.owner.read(cx).failed_draft.is_none());
    });
}
