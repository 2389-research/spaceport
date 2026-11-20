// ABOUTME: Checks complete Close Others consent before any native or terminal tab closes.
// ABOUTME: Uses an actual framed peer to prove cancellation has no queued close effects.
use super::*;
use crate::browser::{Location, Pick, Store};
use sheprd_client::protocol::{ClientMessage, ClientPaneInputEvent};

fn assert_no_close(
    peer: &mut crate::window::MockPeer,
    view: &Entity<SheprdWindow>,
    cx: &VisualTestContext,
) {
    let marker = ClientPaneInputEvent::TextCommit("unchanged-batch-marker".into());
    view.read_with(cx, |view, _| {
        peer.client.handle.send_input(
            &view.live.snapshot.as_ref().unwrap().boot_id,
            "w1:p1",
            [marker.clone()],
        )
    })
    .unwrap();
    loop {
        match peer.receive() {
            ClientMessage::ClientShellPaneInput { events, .. } if events == [marker.clone()] => {
                break;
            }
            ClientMessage::ClientShellEndpointRequest { request, .. } => {
                panic!("unconfirmed batch queued a close: {request}")
            }
            _ => {}
        }
    }
}

#[gpui::test]
fn close_others_captures_the_complete_batch_and_cancel_closes_nothing(cx: &mut TestAppContext) {
    let (view, cx) = cx.add_window_view(crate::sidebar::layout_tests::fixture_window);
    let mut peer = crate::window::MockPeer::advertising(&["tab.close"]);
    let page = cx.update(|window, cx| {
        view.update(cx, |view, cx| {
            peer.prepare_local_tab(view, cx);
            let snapshot = Arc::make_mut(view.live.snapshot.as_mut().unwrap());
            let mut other = snapshot.tabs[0].clone();
            other.tab_id = "other-terminal".into();
            other.label = "Other terminal".into();
            other.agent_status = AgentStatus::Idle;
            snapshot.tabs.push(other);
            *view.endpoints[0].connection.inbox.lock().unwrap() = view.live.clone();
            let (scope, workspace) = view.active_browser_key(cx).unwrap();
            let page = Store::update(cx, |store| {
                store.open(scope, &workspace, Some(Location::Devices), None)
            })
            .unwrap();
            view.close_tabs(
                vec![Pick::Page(page), Pick::Herdr("other-terminal".into())],
                window,
                cx,
            );
            assert!(
                cx.global::<Store>().get(page).is_some(),
                "no view may close before whole-batch consent"
            );
            assert_eq!(view.menu.page, Some(Page::ConfirmClose));
            let close = view.menu.close.as_ref().unwrap();
            assert!(close.label.contains("Devices"));
            assert!(close.label.contains("Other terminal"));
            assert!(!close.confirm_selected);
            view.dismiss_menu(window, cx);
            assert!(cx.global::<Store>().get(page).is_some());
            assert!(view.browser.closing.is_none());
            page
        })
    });
    assert_no_close(&mut peer, &view, cx);
    assert!(cx.read(|cx| cx.global::<Store>().get(page).is_some()));
}

#[gpui::test]
fn changed_targets_refuse_the_whole_batch_before_even_a_client_view_closes(
    cx: &mut TestAppContext,
) {
    for change in ["generation", "pending-pane", "inbox-pane", "view"] {
        let (view, cx) = cx.add_window_view(crate::sidebar::layout_tests::fixture_window);
        let mut peer = crate::window::MockPeer::advertising(&["tab.close"]);
        cx.update(|window, cx| {
            view.update(cx, |view, cx| {
                peer.prepare_local_tab(view, cx);
                let (scope, workspace) = view.active_browser_key(cx).unwrap();
                let page = Store::update(cx, |store| {
                    store.open(scope, &workspace, Some(Location::Devices), None)
                })
                .unwrap();
                let native = view.live.snapshot.as_ref().unwrap().tabs[0].tab_id.clone();
                view.config.confirm_close_tab = false;
                view.close_tabs(vec![Pick::Page(page), Pick::Herdr(native)], window, cx);
                assert_eq!(view.menu.page, Some(Page::ConfirmClose));
                let Target::Batch(batch) = &view.menu.close.as_ref().unwrap().target else {
                    panic!("one captured batch required");
                };
                assert_eq!(batch.items.len(), 2);
                assert_eq!(batch.pane_count(), 1);
                assert_eq!(batch.view_count(), 1);
                match change {
                    "generation" => view.endpoints[0].generation += 1,
                    "view" => {
                        Store::update(cx, |store| store.visited(page, None, Some("changed view")));
                    }
                    _ => {
                        let mut next = (**view.live.snapshot.as_ref().unwrap()).clone();
                        next.revision += 1;
                        let mut pane = next.panes[0].clone();
                        pane.pane_id = "new-unconfirmed-pane".into();
                        next.panes.push(pane);
                        if change == "pending-pane" {
                            view.live.pending_snapshot = Some(Arc::new(next));
                        } else {
                            view.endpoints[0]
                                .connection
                                .inbox
                                .lock()
                                .unwrap()
                                .pending_snapshot = Some(Arc::new(next));
                        }
                    }
                }
                view.send_close(window, cx);
                assert!(
                    view.menu.close.as_ref().unwrap().error.is_some(),
                    "{change}"
                );
                assert!(cx.global::<Store>().get(page).is_some(), "{change}");
                assert!(view.browser.closing.is_none());
            })
        });
        assert_no_close(&mut peer, &view, cx);
    }
}
