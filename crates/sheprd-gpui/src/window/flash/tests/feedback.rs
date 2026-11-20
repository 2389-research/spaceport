// ABOUTME: Checks warning feedback lifetimes through the real window owner.
// ABOUTME: Keeps success feedback short while preserving time to read failures.
use super::*;

#[gpui::test]
fn warnings_outlive_success_feedback(cx: &mut TestAppContext) {
    let (view, cx) = cx.add_window_view(fixture_window);
    view.update(cx, |view, cx| {
        let start = Instant::now();
        view.show_flash(Flash::warning("Could not copy this text"), cx);
        assert!(!view.tick_flash(start + Duration::from_secs(3)));
        assert!(view.flash.is_some());
        assert!(Flash::warning("failed").duration() >= Duration::from_secs(10));
        assert_eq!(Flash::success("copied").duration(), Duration::from_secs(2));
    });
}

#[gpui::test]
fn warning_history_survives_success_and_notice_expiry(cx: &mut TestAppContext) {
    let (view, cx) = cx.add_window_view(fixture_window);
    cx.update(|window, cx| {
        view.update(cx, |view, cx| {
            view.show_flash(Flash::warning("Could not copy this text"), cx);
            view.notices.clear();
            view.show_flash(Flash::success("Copied another selection"), cx);
            let command = sheprd_commands::COMMANDS
                .iter()
                .find(|entry| entry.name == "messages")
                .expect("the shared catalog exposes warning history")
                .command;
            view.command(command, window, cx);
        })
    });
    cx.update(|window, cx| crate::sidebar::layout_tests::full_draw(window, cx).clear(cx));
    assert!(
        cx.debug_bounds("message-text: Could not copy this text")
            .is_some()
    );
    assert!(
        cx.debug_bounds("message-text: Copied another selection")
            .is_none()
    );
}

#[gpui::test]
fn hovering_warning_pauses_its_expiry(cx: &mut TestAppContext) {
    let (view, cx) = cx.add_window_view(fixture_window);
    view.update(cx, |view, cx| {
        view.show_flash(Flash::warning("Could not copy"), cx)
    });
    cx.update(|window, cx| crate::sidebar::layout_tests::full_draw(window, cx).clear(cx));
    let bounds = cx.debug_bounds("flash").expect("warning is painted");
    cx.simulate_mouse_move(bounds.center(), None, gpui::Modifiers::default());
    view.update(cx, |view, _| {
        let expires = view.flash.as_ref().expect("warning stays").1;
        assert!(!view.tick_flash(expires + Duration::from_secs(30)));
        assert!(view.flash.is_some());
    });
}

#[gpui::test]
fn history_can_scroll_to_older_warnings_with_the_keyboard(cx: &mut TestAppContext) {
    let (view, cx) = cx.add_window_view(fixture_window);
    cx.simulate_resize(gpui::size(gpui::px(640.), gpui::px(480.)));
    cx.update(|window, cx| {
        view.update(cx, |view, cx| {
            for index in 0..100 {
                view.notices
                    .show(crate::notice::Cause::Command, format!("warning {index}"));
            }
            view.open_messages(window, cx);
        })
    });
    cx.update(|window, cx| crate::sidebar::layout_tests::full_draw(window, cx).clear(cx));
    cx.simulate_keystrokes("end");
    cx.update(|window, cx| crate::sidebar::layout_tests::full_draw(window, cx).clear(cx));
    let row = cx
        .debug_bounds("message-text: warning 68")
        .expect("oldest retained warning");
    let list = cx.debug_bounds("message-history").expect("history list");
    assert!(
        row.bottom() <= list.bottom(),
        "oldest {row:?}; list {list:?}"
    );
}
