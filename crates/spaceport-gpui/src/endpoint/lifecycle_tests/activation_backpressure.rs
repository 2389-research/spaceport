// ABOUTME: Exercises activation fences under real client queue backpressure.
// ABOUTME: Keeps unsent barriers on their selected host without replaying focus changes.
use super::*;

fn saturate(
    handle: &ClientHandle,
    boot: &str,
    inbox: &Arc<Mutex<LiveState>>,
    server: &mut Server,
) -> (usize, serde_json::Value) {
    handle.focus_pane(boot, "w1:p1").unwrap();
    handle.focus_pane(boot, "w1:p1").unwrap();
    let ClientMessage::ClientShellEndpointRequest { request, .. } = server.receive() else {
        panic!("missing request holding the API lease");
    };
    let first = serde_json::from_str(&request).unwrap();
    // The worker reads this marker after retaining the second request behind
    // the unanswered lease. It then cannot drain another queued command.
    write_message(
        &mut server.stream,
        &ServerMessage::SemanticNotification(crate::notifications::tests::notification(
            "queue-gate",
        )),
        MAX_GRAPHICS_FRAME_SIZE,
    )
    .unwrap();
    wait_until(|| !inbox.lock().unwrap().notifications.is_empty());
    let mut queued = 2;
    for _ in 0..4096 {
        match handle.focus_pane(boot, "w1:p1") {
            Ok(_) => queued += 1,
            Err(spaceport_client::Error::Full) => break,
            Err(error) => panic!("unexpected queue failure: {error}"),
        }
    }
    assert!(queued < 4096, "the fixture must saturate the real queue");
    (queued, first)
}

fn activation_recovers(cx: &mut gpui::TestAppContext, initial: bool) {
    let (fixture, cx) = cx.add_window_view(|window, cx| {
        Fixture(cx.new(|cx| crate::sidebar::layout_tests::fixture_window(window, cx)))
    });
    let view = fixture.update(cx, |fixture, _| fixture.0.clone());
    let (remote, mut server) = connected_endpoint("ssh:backpressure");
    let (handle, boot, frame, inbox) = view.update(cx, |view, cx| {
        prepare_mouse(view, remote, cx);
        let frame = view.live.surface.clone().unwrap();
        let handle = view.endpoints[1].connection.handle.clone().unwrap();
        let boot = view.live.snapshot.as_ref().unwrap().boot_id.clone();
        (
            handle,
            boot,
            frame,
            view.endpoints[1].connection.inbox.clone(),
        )
    });
    let (queued, first) = saturate(&handle, &boot, &inbox, &mut server);
    view.update(cx, |view, cx| {
        if initial {
            view.endpoints[1].initial_surface = false;
        } else {
            view.fence_focus_change(Some(NavigationTarget::Pane("w1:p1".into())));
        }
        view.poll_endpoints(cx);
        assert_eq!(
            view.endpoints[view.selected_endpoint].id,
            "ssh:backpressure"
        );
        assert!(!view.input_ready());
        assert!(matches!(
            view.live.activation,
            crate::state::activation::Fence::Retry { .. }
        ));
    });
    // Drain only requests actually queued before saturation. The next request
    // must be the activation barrier, rather than a replayed focus operation.
    let peer = std::thread::spawn(move || {
        server.respond(&first);
        for _ in 1..queued {
            let ClientMessage::ClientShellEndpointRequest { request, .. } = server.receive() else {
                panic!("expected an original focus request");
            };
            let request: serde_json::Value = serde_json::from_str(&request).unwrap();
            assert_eq!(request["method"], Method::PaneFocus.as_str());
            server.respond(&request);
        }
        loop {
            match server.receive() {
                ClientMessage::ClientShellResize { .. } if initial => {}
                ClientMessage::ClientShellEndpointRequest { request, .. } => {
                    let request: serde_json::Value = serde_json::from_str(&request).unwrap();
                    assert_eq!(request["method"], Method::ClientShellSurfaceSet.as_str());
                    // Deposit the surface and release its lock before the
                    // acknowledgement wakes the reader-driven projection wait.
                    inbox.lock().unwrap().apply(ClientEvent::Surface(frame));
                    background_commands::acknowledge_activation(&mut server, &request);
                    return server;
                }
                message => panic!("unexpected retry frame: {message:?}"),
            }
        }
    });
    view.update(cx, |view, cx| {
        poll_until(view, cx, |view, cx| {
            view.poll_endpoints(cx);
            assert_eq!(
                view.endpoints[view.selected_endpoint].id,
                "ssh:backpressure"
            );
            view.input_ready()
        });
    });
    let mut server = peer.join().unwrap();
    view.update(cx, |view, cx| {
        view.poll_endpoints(cx);
        view.send(
            ClientPaneInputEvent::TextCommit("after recovery".into()),
            cx,
        );
    });
    assert!(matches!(
        server.receive(),
        ClientMessage::ClientShellPaneInput { .. }
    ));
}

#[gpui::test]
fn activation_full_keeps_the_selected_host_fenced(cx: &mut gpui::TestAppContext) {
    activation_recovers(cx, false);
}

#[gpui::test]
fn initial_activation_full_retries_on_the_selected_host(cx: &mut gpui::TestAppContext) {
    activation_recovers(cx, true);
}

#[gpui::test]
fn activation_retry_does_not_cross_a_selection_epoch(cx: &mut gpui::TestAppContext) {
    let (fixture, cx) = cx.add_window_view(|window, cx| {
        Fixture(cx.new(|cx| crate::sidebar::layout_tests::fixture_window(window, cx)))
    });
    let view = fixture.update(cx, |fixture, _| fixture.0.clone());
    let (remote, mut server) = connected_endpoint("ssh:backpressure");
    let (handle, boot, inbox) = view.update(cx, |view, cx| {
        prepare_mouse(view, remote, cx);
        let handle = view.endpoints[1].connection.handle.clone().unwrap();
        let boot = view.live.snapshot.as_ref().unwrap().boot_id.clone();
        (handle, boot, view.endpoints[1].connection.inbox.clone())
    });
    saturate(&handle, &boot, &inbox, &mut server);
    view.update(cx, |view, cx| {
        view.fence_focus_change(Some(NavigationTarget::Pane("w1:p2".into())));
        let epoch = view.selection_epoch;
        view.select_endpoint(LOCAL, cx).unwrap();
        view.poll_endpoints(cx);
        assert_eq!(view.selected_endpoint, 0);
        assert!(view.selection_epoch > epoch);
        assert!(
            handle.is_disconnected(),
            "the full release retires its exact source"
        );
        assert!(!matches!(
            view.live.activation,
            crate::state::activation::Fence::Retry { .. }
        ));
        assert!(view.pending_navigation.is_none());
    });
}
