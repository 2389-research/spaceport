// ABOUTME: Holds the real inbox mutex while an accepted close batch advances.
// ABOUTME: Checks bounded waiting, ownership revalidation, and exactly-once dispatch.
use super::*;
use sheprd_client::protocol::{ClientMessage, ClientPaneInputEvent};

enum Outcome {
    // keep-sorted start
    ChangedOwner,
    Poisoned,
    Recover,
    Timeout,
    // keep-sorted end
}

fn accepted_batch(
    view: &mut SheprdWindow,
    window: &mut Window,
    cx: &mut Context<SheprdWindow>,
) -> crate::window::MockPeer {
    let mut peer = crate::window::MockPeer::advertising(&["tab.close"]);
    peer.prepare_local_tab(view, cx);
    let snapshot = Arc::make_mut(view.live.snapshot.as_mut().unwrap());
    for id in ["first", "second"] {
        let mut tab = snapshot.tabs[0].clone();
        tab.tab_id = id.into();
        snapshot.tabs.push(tab);
    }
    *view.endpoints[0].connection.inbox.lock().unwrap() = view.live.clone();
    view.close_tabs(
        vec![Pick::Herdr("first".into()), Pick::Herdr("second".into())],
        window,
        cx,
    );
    view.send_close(window, cx);
    assert!(view.browser.closing.is_some());
    let first = peer.request();
    assert_eq!(first["method"], "tab.close");
    assert_eq!(first["params"]["tab_id"], "first");
    peer.respond(
        &view.live.snapshot.as_ref().unwrap().boot_id,
        first["id"].as_str().unwrap(),
        &serde_json::json!({"id": first["id"], "result": {}}),
    );
    peer
}

fn no_more_requests(peer: &mut crate::window::MockPeer, view: &SheprdWindow) {
    let marker = ClientPaneInputEvent::TextCommit("close-batch-inspection-marker".into());
    peer.client
        .handle
        .send_input(
            &view.live.snapshot.as_ref().unwrap().boot_id,
            "w1:p1",
            [marker.clone()],
        )
        .unwrap();
    loop {
        match peer.receive() {
            ClientMessage::ClientShellPaneInput { events, .. } if events == [marker.clone()] => {
                return;
            }
            ClientMessage::ClientShellEndpointRequest { request, .. } => {
                panic!("close inspection queued an extra request: {request}");
            }
            _ => {}
        }
    }
}

fn check_contention(cx: &mut gpui::TestAppContext, outcome: Outcome) {
    let since = Instant::now();
    for phase in [
        Phase::Sent(since),
        Phase::Settling(since),
        Phase::Ready(since),
    ] {
        let (fixture, cx) = cx.add_window_view(|window, cx| {
            Fixture(cx.new(|cx| crate::sidebar::layout_tests::fixture_window(window, cx)))
        });
        let view = fixture.update(cx, |fixture, _| fixture.0.clone());
        cx.update(|window, cx| {
            view.update(cx, |view, cx| {
                let mut peer = accepted_batch(view, window, cx);
                let run = view.browser.closing.as_mut().unwrap();
                run.phase = phase;
                run.next = usize::from(!matches!(phase, Phase::Sent(_)));
                Arc::make_mut(view.live.snapshot.as_mut().unwrap())
                    .tabs
                    .retain(|tab| tab.tab_id != "first");
                let surface = view.live.surface.clone();
                if matches!(phase, Phase::Settling(_)) {
                    view.live.surface = None;
                }
                let inbox = view.endpoints[0].connection.inbox.clone();
                let mut guard = inbox.lock().unwrap();
                *guard = view.live.clone();
                if matches!(outcome, Outcome::Poisoned) {
                    drop(guard);
                    let poisoned = inbox.clone();
                    assert!(
                        std::thread::spawn(move || {
                            let _guard = poisoned.lock().unwrap();
                            panic!("owned poison fixture");
                        })
                        .join()
                        .is_err()
                    );
                    view.advance_close_run_at(window, cx, since + Duration::from_secs(1));
                    assert!(view.browser.closing.is_none());
                } else {
                    view.advance_close_run_at(window, cx, since + Duration::from_secs(1));
                    let run = view
                        .browser
                        .closing
                        .as_ref()
                        .unwrap_or_else(|| panic!("busy inspection must wait"));
                    assert_eq!(
                        std::mem::discriminant(&run.phase),
                        std::mem::discriminant(&phase)
                    );
                    let retained = match run.phase {
                        Phase::Ready(at) | Phase::Sent(at) | Phase::Settling(at) => at,
                    };
                    assert_eq!(retained, since, "contention must not renew the deadline");
                    if matches!(outcome, Outcome::Timeout) {
                        view.advance_close_run_at(window, cx, since + ANSWER_TIMEOUT);
                        assert!(view.browser.closing.is_none());
                        drop(guard);
                    } else {
                        view.live.surface = surface;
                        *guard = view.live.clone();
                        drop(guard);
                        if matches!(outcome, Outcome::ChangedOwner) {
                            view.selection_epoch += 1;
                        }
                        view.advance_close_run_at(window, cx, since + Duration::from_secs(2));
                        if matches!(outcome, Outcome::Recover) {
                            let second = peer.request();
                            assert_eq!(second["method"], "tab.close");
                            assert_eq!(second["params"]["tab_id"], "second");
                            view.advance_close_run_at(window, cx, since + Duration::from_secs(3));
                            assert!(view.browser.closing.is_some());
                        } else {
                            assert!(view.browser.closing.is_none());
                        }
                    }
                }
                no_more_requests(&mut peer, view);
            })
        });
    }
}

#[gpui::test]
fn a_busy_inbox_preserves_the_batch_until_inspection_can_resume(cx: &mut gpui::TestAppContext) {
    check_contention(cx, Outcome::Recover);
}

#[gpui::test]
fn a_busy_inbox_cannot_extend_the_batch_deadline(cx: &mut gpui::TestAppContext) {
    check_contention(cx, Outcome::Timeout);
}

#[gpui::test]
fn resumed_inspection_rejects_changed_ownership(cx: &mut gpui::TestAppContext) {
    check_contention(cx, Outcome::ChangedOwner);
}

#[gpui::test]
fn a_poisoned_inbox_stops_the_batch(cx: &mut gpui::TestAppContext) {
    check_contention(cx, Outcome::Poisoned);
}
