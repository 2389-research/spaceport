use super::*;

#[gpui::test]
fn general_settings_have_no_automatic_worktree_removal(cx: &mut TestAppContext) {
    let (view, cx) = cx.add_window_view(skill_fixture);
    view.update(cx, |view, cx| {
        view.section = Section::General;
        cx.notify();
    });
    cx.simulate_resize(size(px(960.), px(2200.)));
    cx.update(|window, cx| crate::sidebar::layout_tests::full_draw(window, cx).clear(cx));

    assert!(cx.debug_bounds("general-merged-worktrees-toggle").is_none());
    assert!(cx.debug_bounds("general-smooth-scroll-toggle").is_some());
}
