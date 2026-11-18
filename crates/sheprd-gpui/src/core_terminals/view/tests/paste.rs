// ABOUTME: Exercises Core clipboard completion against its actual view and worker.
// ABOUTME: Covers conditional key releases, focus cancellation, and visible read errors.
use super::*;
use crate::core_terminals::{tests::recording_link, worker::Command};
use gpui::{ClipboardItem, KeyDownEvent, KeyUpEvent, VisualTestContext};
use sheprd_backend::{ClientKeyKind, ClientPaneInputEvent};
use std::{
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

fn terminal(
    cx: &mut gpui::TestAppContext,
) -> (
    gpui::Entity<CoreTerminalView>,
    Arc<Mutex<Vec<Command>>>,
    &mut VisualTestContext,
) {
    let (owner, cx) = cx.add_window_view(fixture_window);
    let (link, received) = recording_link();
    let view = cx.update(|window, cx| {
        let owner = owner.read(cx);
        let look = owner.core_look();
        let painter = owner.painter.clone();
        let view = window.replace_root(cx, |_, cx| {
            CoreTerminalView::new(
                TerminalId::new("paste-view").unwrap(),
                link,
                look,
                painter,
                false,
                cx,
            )
        });
        window.focus(&view.read(cx).focus.clone(), cx);
        view
    });
    cx.run_until_parked();
    (view, received, cx)
}

fn inputs(received: &Mutex<Vec<Command>>, wanted: usize) -> Vec<ClientPaneInputEvent> {
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        let inputs: Vec<_> = received
            .lock()
            .unwrap()
            .iter()
            .filter_map(|command| match command {
                Command::Input(input) => Some(input.clone()),
                _ => None,
            })
            .collect();
        if inputs.len() >= wanted {
            return inputs;
        }
        assert!(Instant::now() < deadline, "missing Core input: {inputs:?}");
        std::thread::sleep(Duration::from_millis(2));
    }
}

#[gpui::test]
fn ctrl_v_probe_pairs_only_its_fallback_key_and_orders_following_input(
    cx: &mut gpui::TestAppContext,
) {
    for (text, release_first) in [
        (None, true),
        (Some("clipboard"), true),
        (None, false),
        (Some("clipboard"), false),
    ] {
        let (view, received, cx) = terminal(cx);
        let (sender, receiver) = futures_channel::oneshot::channel();
        let stroke = crate::keymap::parse_keystroke("ctrl-v").unwrap();
        let event = KeyDownEvent {
            keystroke: stroke.clone(),
            is_held: false,
            prefer_character_input: false,
        };
        view.update_in(cx, |view, window, cx| {
            view.screen.keys.releases = true;
            let fallback = crate::terminal::key_input(&event, false).unwrap();
            assert!(view.start_paste_with(
                Some(("v".into(), fallback)),
                |cx| {
                    Ok(cx
                        .background_executor()
                        .spawn(async move { receiver.await.unwrap() }))
                },
                window,
                cx
            ));
            if release_first {
                view.key_up(
                    &KeyUpEvent {
                        keystroke: stroke.clone(),
                    },
                    window,
                    cx,
                );
            }
        });
        cx.run_until_parked();
        sender
            .send(Ok(text.map(|text| ClipboardItem::new_string(text.into()))))
            .unwrap();
        cx.run_until_parked();
        view.update_in(cx, |view, window, cx| {
            if !release_first {
                view.key_up(&KeyUpEvent { keystroke: stroke }, window, cx);
            }
            view.send(ClientPaneInputEvent::TextCommit("following".into()));
        });
        let got = inputs(&received, if text.is_some() { 2 } else { 3 });
        if text.is_some() {
            assert_eq!(
                got,
                [
                    ClientPaneInputEvent::Paste("clipboard".into()),
                    ClientPaneInputEvent::TextCommit("following".into())
                ]
            );
        } else {
            assert!(matches!(
                got[0],
                ClientPaneInputEvent::Key {
                    kind: ClientKeyKind::Press,
                    tracks_release: true,
                    ..
                }
            ));
            assert!(matches!(
                got[1],
                ClientPaneInputEvent::Key {
                    kind: ClientKeyKind::Release,
                    ..
                }
            ));
            assert_eq!(got[2], ClientPaneInputEvent::TextCommit("following".into()));
        }
    }
}

#[gpui::test]
fn focus_change_cancels_only_prepared_input_and_preserves_accepted_typing(
    cx: &mut gpui::TestAppContext,
) {
    let (view, received, cx) = terminal(cx);
    let (sender, receiver) = futures_channel::oneshot::channel();
    view.update_in(cx, |view, window, cx| {
        assert!(view.start_paste_with(
            None,
            |cx| {
                Ok(cx
                    .background_executor()
                    .spawn(async move { receiver.await.unwrap() }))
            },
            window,
            cx
        ));
        view.send(ClientPaneInputEvent::TextCommit("accepted".into()));
    });
    cx.run_until_parked();
    cx.update(|window, cx| {
        let other = cx.focus_handle();
        window.focus(&other, cx);
    });
    cx.run_until_parked();
    let _ = sender.send(Ok(Some(ClipboardItem::new_string("stale".into()))));
    cx.run_until_parked();
    view.update_in(cx, |view, window, cx| {
        window.focus(&view.focus, cx);
        assert!(view.pending_paste.is_none());
        view.send(ClientPaneInputEvent::TextCommit("later".into()));
    });
    assert_eq!(
        inputs(&received, 2),
        [
            ClientPaneInputEvent::TextCommit("accepted".into()),
            ClientPaneInputEvent::TextCommit("later".into()),
        ]
    );
}

#[gpui::test]
fn explicit_read_failure_is_visible_and_preserves_accepted_input(cx: &mut gpui::TestAppContext) {
    let (view, received, cx) = terminal(cx);
    view.update_in(cx, |view, window, cx| {
        assert!(view.start_paste_with(
            None,
            |cx| {
                Ok(cx
                    .background_executor()
                    .spawn(async { Err(crate::window::clipboard::Error::Timeout.into()) }))
            },
            window,
            cx
        ));
        view.send(ClientPaneInputEvent::TextCommit("accepted".into()));
    });
    cx.run_until_parked();
    view.update(cx, |view, cx| {
        view.poll(cx);
        assert!(
            view.status()
                .is_some_and(|status| status.contains("timed out"))
        );
        assert!(view.pending_paste.is_none());
        view.send(ClientPaneInputEvent::TextCommit("later".into()));
    });
    assert_eq!(
        inputs(&received, 2),
        [
            ClientPaneInputEvent::TextCommit("accepted".into()),
            ClientPaneInputEvent::TextCommit("later".into()),
        ]
    );
}
