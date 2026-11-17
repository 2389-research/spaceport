// ABOUTME: Keeps successful uploads until their captured terminal accepts input.
// ABOUTME: Checks menu and projection waits without retrying submitted pastes.

use super::*;

#[gpui::test]
fn completed_upload_waits_for_menu_or_input_readiness(cx: &mut TestAppContext) {
    for (menu, is_popup) in [(true, false), (false, false), (true, true), (false, true)] {
        let (fixture, cx) = cx.add_window_view(fixture);
        let view = fixture.read_with(cx, |fixture, _| fixture.view.clone().unwrap());
        let mut peer = Peer::new();
        let (tx, rx) = mpsc::channel();
        view.update(cx, |view, cx| {
            peer.prepare(view, cx);
            let target = if is_popup {
                popup(view, "popup");
                InputTarget::Popup("popup".into())
            } else {
                InputTarget::Pane("w1:p1".into())
            };
            view.start_file_transfer_with(
                target,
                vec!["source".into()],
                |_, _, _, _| Ok(vec![REMOTE.into()]),
                move |_, _| {
                    tx.send(()).unwrap();
                    Ok(())
                },
                cx,
            );
            if menu {
                view.menu.page = Some(crate::menu::Page::Palette);
            } else {
                view.pending_toast = Some(1);
            }
            assert!(matches!(view.file_transfer, FileTransfer::Uploading(_)));
        });
        cx.run_until_parked();
        view.read_with(cx, |view, _| {
            assert!(
                view.file_transfer.active().is_some(),
                "successful upload must wait"
            );
            assert!(matches!(view.file_transfer, FileTransfer::Waiting { .. }));
            assert!(view.endpoints[0].toasts.entries.is_empty());
        });
        assert!(
            rx.try_recv().is_err(),
            "waiting upload must retain its files"
        );
        peer.sentinel(&view, cx);
        view.update(cx, |view, cx| {
            view.menu.page = None;
            view.pending_toast = None;
            Arc::make_mut(view.live.snapshot.as_mut().unwrap()).focused_pane_id =
                Some("different-pane".into());
            view.poll_file_transfer(cx);
            assert!(matches!(view.file_transfer, FileTransfer::Accepted(_)));
        });
        cx.run_until_parked();
        let events = vec![ClientPaneInputEvent::Paste(
            "/tmp/herdr-upload.ABCDEF123456/it\\'s\\ a\\ file".into(),
        )];
        assert_eq!(
            peer.receive(),
            if is_popup {
                ClientMessage::ClientShellPopupInput {
                    terminal_id: "popup".into(),
                    events,
                }
            } else {
                ClientMessage::ClientShellPaneInput {
                    pane_id: "w1:p1".into(),
                    events,
                }
            }
        );
        view.update(cx, |view, cx| view.poll_file_transfer(cx));
        peer.sentinel(&view, cx);
        view.read_with(cx, |view, _| assert!(view.file_transfer.active().is_none()));
        assert!(rx.try_recv().is_err());
    }
}

#[gpui::test]
fn waiting_upload_cleans_on_cancel_stale_target_or_window_drop(cx: &mut TestAppContext) {
    for case in 0..3 {
        let (fixture, cx) = cx.add_window_view(fixture);
        let view = fixture.read_with(cx, |fixture, _| fixture.view.clone().unwrap());
        let weak = view.downgrade();
        let mut peer = Peer::new();
        let (tx, rx) = mpsc::channel();
        view.update(cx, |view, cx| {
            peer.prepare(view, cx);
            view.start_file_transfer_with(
                InputTarget::Pane("w1:p1".into()),
                vec!["source".into()],
                |_, _, _, _| Ok(vec![REMOTE.into()]),
                move |host, paths| {
                    tx.send((host.to_owned(), paths.to_vec())).unwrap();
                    Ok(())
                },
                cx,
            );
            view.menu.page = Some(crate::menu::Page::Palette);
        });
        cx.run_until_parked();
        view.read_with(cx, |view, _| {
            assert!(matches!(view.file_transfer, FileTransfer::Waiting { .. }));
        });
        assert!(rx.try_recv().is_err());
        if case == 2 {
            fixture.update(cx, |fixture, _| fixture.view = None);
            drop(view);
            cx.update(|_, _| {});
            assert!(weak.upgrade().is_none());
        } else {
            view.update(cx, |view, cx| {
                if case == 0 {
                    view.cancel_file_transfer(cx);
                } else {
                    view.selection_epoch += 1;
                    view.endpoints[0].connection.target = ConnectTarget::Ssh {
                        target: "other.invalid".into(),
                        session: "default".into(),
                    };
                    view.poll_file_transfer(cx);
                }
                assert!(matches!(view.file_transfer, FileTransfer::Cleaning(_)));
            });
            peer.sentinel(&view, cx);
        }
        cx.run_until_parked();
        assert_eq!(rx.try_recv().unwrap(), (HOST.into(), vec![REMOTE.into()]));
        assert!(rx.try_recv().is_err(), "cleanup runs once");
    }
}

#[gpui::test]
fn refused_waiting_paste_is_never_retried(cx: &mut TestAppContext) {
    let (fixture, cx) = cx.add_window_view(fixture);
    let view = fixture.read_with(cx, |fixture, _| fixture.view.clone().unwrap());
    let mut peer = Peer::new();
    let (tx, rx) = mpsc::channel();
    view.update(cx, |view, cx| {
        peer.prepare(view, cx);
        view.start_file_transfer_with(
            InputTarget::Pane("w1:p1".into()),
            vec!["source".into()],
            |_, _, _, _| Ok(vec![REMOTE.into()]),
            move |_, _| {
                tx.send(()).unwrap();
                Ok(())
            },
            cx,
        );
        view.pending_toast = Some(1);
    });
    cx.run_until_parked();
    let stopped = Peer::new();
    stopped.client.handle.disconnect();
    view.update(cx, |view, cx| {
        view.pending_toast = None;
        view.endpoints[0].connection.handle = Some(stopped.client.handle.clone());
        view.poll_file_transfer(cx);
        assert!(matches!(view.file_transfer, FileTransfer::Cleaning(_)));
        view.endpoints[0].connection.handle = Some(peer.client.handle.clone());
        view.poll_file_transfer(cx);
        assert!(matches!(view.file_transfer, FileTransfer::Cleaning(_)));
    });
    cx.run_until_parked();
    assert_eq!(rx.try_recv(), Ok(()));
    peer.sentinel(&view, cx);
    view.read_with(cx, |view, _| {
        assert!(matches!(view.file_transfer, FileTransfer::Idle))
    });
}
