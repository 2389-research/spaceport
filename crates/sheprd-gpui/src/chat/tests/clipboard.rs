// ABOUTME: Verifies async chat clipboard ownership against real framed conversations.
// ABOUTME: Covers immediate Send refusal, session cancellation, and attachment routing.
use super::{
    composer::{idle, prompt_text},
    overlay::{attach_with, open, poll},
    *,
};
use gpui::{ClipboardItem, Entity, TestAppContext, VisualTestContext};
use sheprd_chat_protocol::Command;
use sheprd_client::ConnectTarget;

pub(super) fn pending(
    view: &Entity<crate::SheprdWindow>,
    cx: &mut VisualTestContext,
) -> (
    String,
    Entity<crate::multiline_input::MultilineInput>,
    futures_channel::oneshot::Sender<crate::Result<Option<ClipboardItem>>>,
) {
    let (sender, receiver) = futures_channel::oneshot::channel();
    let (id, input) = view.read_with(cx, |view, _| {
        let pane = view.chats.panes().next().unwrap();
        (pane.target.pane_id.clone(), pane.composer.clone().unwrap())
    });
    cx.update(|window, cx| {
        view.update(cx, |view, cx| {
            view.start_chat_paste_with(
                &id,
                &input,
                |cx| {
                    Ok(cx
                        .background_executor()
                        .spawn(async move { receiver.await.unwrap() }))
                },
                window,
                cx,
            );
        })
    });
    cx.run_until_parked();
    assert!(input.read_with(cx, |input, _| input.is_pasting()));
    (id, input, sender)
}

#[gpui::test]
fn send_and_completion_refuse_synchronously_and_never_submit_later(cx: &mut TestAppContext) {
    let (view, _peer, cx) = open(cx, ConnectTarget::Local);
    let program = Program::new();
    let mut stream = attach_with(&view, &program, idle(), cx);
    cx.simulate_input("draft");
    let (id, input, sender) = pending(&view, cx);
    view.update(cx, |view, cx| view.send_chat_composer(&id, cx));
    cx.simulate_keystrokes("enter");
    cx.update(|window, cx| {
        view.update(cx, |view, cx| {
            view.complete_chat_command(&id, "help", window, cx)
        })
    });
    assert_eq!(
        input.read_with(cx, |input, _| input.text().to_owned()),
        "draft"
    );
    sender
        .send(Ok(Some(ClipboardItem::new_string(" paste".into()))))
        .unwrap();
    cx.run_until_parked();
    assert_eq!(
        input.read_with(cx, |input, _| input.text().to_owned()),
        "draft paste"
    );
    // A real later command proves that neither refused Send queued a prompt.
    view.update(cx, |view, cx| view.chat_send(&id, Command::Cancel, cx));
    assert!(matches!(
        read(&mut stream),
        Some(ClientMessage::Command {
            command: Command::Cancel,
            ..
        })
    ));
    cx.simulate_keystrokes("enter");
    assert_eq!(prompt_text(&mut stream), "draft paste");
}

#[gpui::test]
fn a_new_epoch_cancels_a_pending_read_before_it_finishes(cx: &mut TestAppContext) {
    let (view, _peer, cx) = open(cx, ConnectTarget::Local);
    let program = Program::new();
    let mut stream = attach_with(&view, &program, idle(), cx);
    let (_id, input, sender) = pending(&view, cx);
    send(&mut stream, &snapshot(cursor(2, 0), vec![]));
    wait_until(|| {
        poll(&view, &program, cx);
        !input.read_with(cx, |input, _| input.is_pasting())
    });
    let _ = sender.send(Ok(Some(ClipboardItem::new_string("stale".into()))));
    cx.run_until_parked();
    assert_eq!(input.read_with(cx, |input, _| input.text().to_owned()), "");
}

#[gpui::test]
fn a_replacement_target_never_receives_old_clipboard_files(cx: &mut TestAppContext) {
    let (view, _peer, cx) = open(cx, ConnectTarget::Local);
    let program = Program::new();
    let _stream = attach_with(&view, &program, idle(), cx);
    let (_id, input, sender) = pending(&view, cx);
    view.update(cx, |view, _| {
        let mut target = view.chats.panes().next().unwrap().target.clone();
        target.source = Source::Program(EndpointId::random());
        view.chats
            .reconcile(vec![target], Instant::now(), &mut |_, _| Err(Error::Closed));
    });
    sender
        .send(Ok(Some(ClipboardItem {
            entries: vec![gpui::ClipboardEntry::ExternalPaths(gpui::ExternalPaths(
                vec![PathBuf::from("/fixture/old-attachment")].into(),
            ))],
        })))
        .unwrap();
    cx.run_until_parked();
    view.read_with(cx, |view, _| {
        assert!(view.chats.panes().next().unwrap().attachments.is_empty())
    });
    assert_eq!(input.read_with(cx, |input, _| input.text().to_owned()), "");
}

#[gpui::test]
fn ordinary_sequence_advancement_preserves_the_pending_paste(cx: &mut TestAppContext) {
    let (view, _peer, cx) = open(cx, ConnectTarget::Local);
    let program = Program::new();
    let mut stream = attach_with(&view, &program, idle(), cx);
    let (id, input, sender) = pending(&view, cx);
    send(&mut stream, &snapshot(cursor(1, 1), vec![]));
    wait_until(|| {
        poll(&view, &program, cx);
        view.read_with(cx, |view, _| {
            view.chats.pane(&id).unwrap().session.since() == Some(cursor(1, 1))
        })
    });
    assert!(input.read_with(cx, |input, _| input.is_pasting()));
    sender
        .send(Ok(Some(ClipboardItem::new_string("current".into()))))
        .unwrap();
    cx.run_until_parked();
    assert_eq!(
        input.read_with(cx, |input, _| input.text().to_owned()),
        "current"
    );
}
