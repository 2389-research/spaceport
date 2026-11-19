// ABOUTME: Keeps opted-in chat smoothing subordinate to OS reduced motion.
// ABOUTME: Exercises live transcript growth and an already moving chat list.
use super::*;

#[gpui::test]
fn reduced_motion_shows_new_reply_text_without_smoothing(cx: &mut TestAppContext) {
    let _scope = crate::accessibility::policy::tests::Scope::new();
    cx.update(crate::accessibility::policy::tests::reduce);
    let (view, _peer, cx) = gliding(cx);
    let program = Program::new();
    let left = grow_last(&view, &program, cx);
    assert!(
        left <= 1.,
        "reduced motion shows new output at once: {left}"
    );
}

#[gpui::test]
fn enabling_reduced_motion_finishes_an_active_chat_glide(cx: &mut TestAppContext) {
    let _scope = crate::accessibility::policy::tests::Scope::new();
    let (view, _peer, cx) = gliding(cx);
    let program = Program::new();
    assert!(grow_last(&view, &program, cx) > 20.);
    cx.update(|_, cx| crate::accessibility::policy::tests::reduce(cx));
    redraw(&view, cx);
    assert!(
        behind(&view, cx) <= 1.,
        "an active glide settles immediately"
    );
    assert!(!view.update(cx, |view, _| view.chats.glide(crate::motion::now(), true)));
}
