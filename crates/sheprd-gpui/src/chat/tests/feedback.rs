// ABOUTME: Keeps gesture feedback above chat panes drawn over terminal cells.
// ABOUTME: Exercises pointer hit testing against the real chat overlay.
use super::{
    overlay::{attach_with, open},
    *,
};
use crate::{sidebar::layout_tests::full_draw, window::Flash};
use gpui::{Modifiers, TestAppContext};
use sheprd_client::ConnectTarget;
use std::time::Duration;

#[gpui::test]
fn a_warning_above_chat_accepts_hover_and_keeps_its_reading_time(cx: &mut TestAppContext) {
    let (view, _peer, cx) = open(cx, ConnectTarget::Local);
    let program = Program::new();
    let _stream = attach_with(
        &view,
        &program,
        snapshot(cursor(1, 0), vec![agent(1, "Ready.")]),
        cx,
    );
    view.update(cx, |view, cx| {
        view.config.clipboard_toast.position = crate::config::ClipboardToastPosition::TopLeft;
        view.show_flash(Flash::warning("Could not copy"), cx)
    });
    cx.update(|window, cx| full_draw(window, cx).clear(cx));
    let chat = cx.debug_bounds("chat-view").unwrap();
    let flash = cx.debug_bounds("flash").unwrap();
    assert!(
        chat.contains(&flash.center()),
        "the chat covers the warning's position"
    );
    cx.simulate_mouse_move(flash.center(), None, Modifiers::default());
    view.update(cx, |view, _| {
        let until = view.flash.as_ref().unwrap().1;
        assert!(
            !view.tick_flash(until + Duration::from_secs(1)),
            "hover reaches the warning above chat"
        );
        assert!(view.flash.is_some());
    });
}
