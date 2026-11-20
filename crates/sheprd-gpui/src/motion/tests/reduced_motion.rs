// ABOUTME: Checks that reduced motion finishes manual transitions immediately.
// ABOUTME: Exercises the same app policy that controls GPUI animations.
use super::*;

#[gpui::test]
fn reduced_motion_finishes_manual_transitions(cx: &mut gpui::TestAppContext) {
    let _scope = crate::accessibility::policy::tests::Scope::new();
    cx.update(crate::accessibility::policy::tests::reduce);
    let now = Instant::now();
    assert_eq!(progress(now, now, ENTER), None);
    let mut panel = Toggle::default();
    panel.set(true, now, false);
    assert_eq!(panel.shown(now), 1.);
    assert!(!panel.moving(now));
    panel.set(false, now, false);
    assert_eq!(panel.shown(now), 0.);
    assert!(!panel.moving(now));
}
