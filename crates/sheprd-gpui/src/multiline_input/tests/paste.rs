// ABOUTME: Verifies clipboard acquisition leaves the UI execution context.
// ABOUTME: Exercises the rendered field's real bounded replacement path.
use super::*;
use gpui::EntityInputHandler;
use std::sync::atomic::{AtomicBool, Ordering};

fn pending(
    input: &Entity<MultilineInput>,
    cx: &mut VisualTestContext,
) -> futures_channel::oneshot::Sender<crate::Result<Option<gpui::ClipboardItem>>> {
    let (sender, receiver) = futures_channel::oneshot::channel();
    cx.update(|window, cx| {
        let read = cx
            .background_executor()
            .spawn(async move { receiver.await.unwrap() });
        input.update(cx, |input, cx| {
            let ticket = input.begin_paste(window, cx).unwrap();
            input.await_paste(&ticket, read, window, cx);
        });
    });
    cx.run_until_parked();
    sender
}

fn finish(
    sender: futures_channel::oneshot::Sender<crate::Result<Option<gpui::ClipboardItem>>>,
    cx: &mut VisualTestContext,
) {
    let _ = sender.send(Ok(Some(gpui::ClipboardItem::new_string("pasted".into()))));
    cx.run_until_parked();
}

#[gpui::test]
fn clipboard_acquisition_runs_outside_the_ui_context(cx: &mut TestAppContext) {
    let (input, cx) = field(cx);
    let on_ui = Arc::new(AtomicBool::new(true));
    let read_on_ui = on_ui.clone();
    cx.update(|window, cx| {
        let executor = cx.background_executor().clone();
        input.update(cx, |input, cx| {
            input.paste_from(
                move || {
                    read_on_ui.store(executor.is_main_thread(), Ordering::Release);
                    Ok(Some(gpui::ClipboardItem::new_string("pasted".into())))
                },
                window,
                cx,
            );
        });
    });
    cx.run_until_parked();
    assert!(
        !on_ui.load(Ordering::Acquire),
        "clipboard acquisition ran on the UI context"
    );
    assert_eq!(
        input.read_with(cx, |input, _| input.text().to_owned()),
        "pasted"
    );
}

#[gpui::test]
fn native_edits_cancel_pending_paste_without_delaying_typing(cx: &mut TestAppContext) {
    let (input, cx) = field(cx);
    let sender = pending(&input, cx);
    cx.simulate_input("typed now");
    assert_eq!(
        input.read_with(cx, |input, _| input.text().to_owned()),
        "typed now"
    );
    assert!(!input.read_with(cx, |input, _| input.is_pasting()));
    finish(sender, cx);
    assert_eq!(
        input.read_with(cx, |input, _| input.text().to_owned()),
        "typed now"
    );
}

#[gpui::test]
fn selection_return_reset_and_ime_cancel_pending_paste(cx: &mut TestAppContext) {
    let (input, cx) = field(cx);
    for action in 0..3 {
        input.update(cx, |input, cx| input.set_text_at("draft", 5, cx));
        let sender = pending(&input, cx);
        match action {
            0 => cx.simulate_keystrokes("left right"),
            1 => input.update(cx, |input, cx| input.set_text_at("draft", 5, cx)),
            _ => cx.update(|window, cx| {
                input.update(cx, |input, cx| {
                    input.replace_and_mark_text_in_range(None, "", None, window, cx);
                })
            }),
        }
        assert!(!input.read_with(cx, |input, _| input.is_pasting()));
        finish(sender, cx);
        assert_eq!(
            input.read_with(cx, |input, _| input.text().to_owned()),
            "draft"
        );
    }
}

#[gpui::test]
fn leaving_and_returning_focus_cannot_revive_a_paste(cx: &mut TestAppContext) {
    let (input, cx) = field(cx);
    cx.update(|window, _| window.activate_window());
    draw(cx);
    let sender = pending(&input, cx);
    let other = cx.update(|window, cx| {
        let other = cx.focus_handle();
        window.focus(&other, cx);
        other
    });
    // GPUI publishes blur subscriptions at the end of the focused frame.
    draw(cx);
    assert!(!input.read_with(cx, |input, _| input.is_pasting()));
    cx.update(|window, cx| window.focus(&input.read(cx).focus.clone(), cx));
    finish(sender, cx);
    drop(other);
    assert_eq!(input.read_with(cx, |input, _| input.text().to_owned()), "");
    assert!(!input.read_with(cx, |input, _| input.is_pasting()));
}

#[gpui::test]
fn pending_owner_actions_refuse_then_paste_obeys_the_prompt_byte_cap(cx: &mut TestAppContext) {
    let (input, cx) = field(cx);
    let text = "a".repeat(sheprd_chat_protocol::MAX_PROMPT_SIZE);
    input.update(cx, |input, cx| input.set_text_at(&text, text.len(), cx));
    let sender = pending(&input, cx);
    assert!(input.update(cx, |input, cx| input.refuse_while_pasting(cx)));
    assert!(input.read_with(cx, |input, _| input.error.is_some()));
    finish(sender, cx);
    input.read_with(cx, |input, _| {
        assert_eq!(input.text(), text);
        assert!(input.error.is_some());
        assert!(!input.is_pasting());
    });
}

#[gpui::test]
fn clipboard_errors_clear_preparation_and_remain_visible(cx: &mut TestAppContext) {
    let (input, cx) = field(cx);
    let sender = pending(&input, cx);
    sender
        .send(Err(crate::window::clipboard::Error::Timeout.into()))
        .unwrap();
    cx.run_until_parked();
    input.read_with(cx, |input, _| {
        assert!(!input.is_pasting());
        assert_eq!(input.text(), "");
        assert!(
            input
                .error
                .as_ref()
                .is_some_and(|error| error.contains("timed out"))
        );
    });
}
