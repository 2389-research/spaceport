use super::*;

#[test]
fn a_drop_is_an_outage_until_a_connection_has_a_snapshot() {
    let (mut endpoint, server) = connected_endpoint("ssh:remote");
    assert_eq!(endpoint.outage(), None);
    server.stream.shutdown(std::net::Shutdown::Both).unwrap();
    wait_until(|| {
        endpoint.poll(Instant::now());
        endpoint.connection.handle.is_none()
    });
    assert!(endpoint.outage().is_some());
    assert_eq!(endpoint.status(), "reconnecting");
    // A retry replaces `live` with a fresh attempt; the outage outlives it,
    // whether or not that attempt has already failed.
    endpoint.connect(ConnectOptions::default(), false);
    assert!(endpoint.outage().is_some());
    assert_eq!(endpoint.status(), "reconnecting");

    let (mut endpoint, _server) = connected_endpoint("ssh:remote");
    endpoint.lose_connection_for_test("connection lost");
    endpoint.poll(Instant::now());
    assert_eq!(endpoint.outage(), None);
    assert_eq!(endpoint.status(), "online");
}

#[gpui::test]
fn a_dropped_remote_stays_selected_with_its_last_picture_dimmed(cx: &mut gpui::TestAppContext) {
    let (fixture, cx) = cx.add_window_view(|window, cx| {
        Fixture(cx.new(|cx| crate::sidebar::layout_tests::fixture_window(window, cx)))
    });
    let view = fixture.update(cx, |fixture, _| fixture.0.clone());
    let (endpoint, server) = connected_endpoint("ssh:remote");
    view.update(cx, |view, cx| {
        prepare_mouse(view, endpoint, cx);
        assert!(view.presentation.picture(&view.live).is_some());
        assert!(!view.presentation.stale());
        view.endpoints[1].set_retry_at_for_test(Instant::now() + Duration::from_secs(120));
    });
    server.stream.shutdown(std::net::Shutdown::Both).unwrap();
    view.update(cx, |view, cx| {
        // Wait for the disconnect state, not only the stopped handle: the poll
        // that lands it takes the endpoint's state for the window's, which would
        // discard the expired budget this test is about. A stopped handle is
        // seen one poll before its state arrives when the inbox is busy.
        project_until(view, cx, "the drop's disconnect state", |view| {
            let endpoint = &view.endpoints[1];
            endpoint.connection.handle.is_none() && !endpoint.live.status.is_connected()
        });
        // An expired activation budget is renewed while there is no snapshot.
        view.live
            .activation
            .transition(crate::state::activation::Transition::Reset(Some(
                Instant::now(),
            )));
        view.poll_endpoints(cx);
        assert_eq!(
            view.selected_endpoint, 1,
            "a drop must not fall back to Local"
        );
        assert!(
            view.live
                .activation
                .deadline()
                .is_some_and(|deadline| deadline > Instant::now()),
            "a drop must renew the expired activation budget"
        );
        assert!(view.presentation.stale());
        assert!(view.presentation.picture(&view.live).is_some());

        // Keys typed into the lost connection go nowhere, and say so.
        view.send(ClientPaneInputEvent::TextCommit("x".into()), cx);
        let (flash, _) = view.flash.as_ref().unwrap();
        assert!(flash.text.contains("Not connected"));
        assert_eq!(view.pending_input.len(), 0);

        // An automatic retry keeps the picture up while it connects.
        view.endpoints[1].set_retry_at_for_test(Instant::now());
        view.poll_endpoints(cx);
        assert_eq!(view.selected_endpoint, 1);
        assert_eq!(view.selected_generation, view.endpoints[1].generation);
        assert!(view.presentation.stale());
        assert!(view.presentation.picture(&view.live).is_some());

        // Leaving the endpoint leaves its picture behind.
        assert!(view.switch_endpoint(LOCAL, cx).is_ok());
        assert!(!view.presentation.stale());
        assert!(view.presentation.picture(&view.live).is_none());
    });
}

#[test]
fn a_refusal_only_the_user_can_fix_waits_the_longest_retry_delay() {
    for (failure, waits_longest) in [
        (SshFailure::Auth, true),
        (SshFailure::HostKey, true),
        (SshFailure::HerdrMissing, true),
        (SshFailure::Unreachable, false),
    ] {
        let (mut endpoint, _server) = connected_endpoint("ssh:remote");
        let error = spaceport_client::Error::Ssh(spaceport_client::ssh::Error::Refused(failure))
            .to_string();
        endpoint.connection.inbox.lock().unwrap().apply(
            ClientEvent::Disconnected {
                reason: error.clone(),
                ssh: Some(failure),
            },
            &crate::state::labels(),
        );
        endpoint.connection.handle.as_ref().unwrap().disconnect();
        let now = Instant::now();
        // The inbox may be busy for a poll; the state still lands on a later one.
        wait_until(|| {
            endpoint.poll(now);
            endpoint.live.ssh_failure.is_some()
        });
        let expected = if waits_longest {
            MAX_RETRY_DELAY
        } else {
            endpoint.retry_delay()
        };
        assert_eq!(endpoint.retry_at(), now + expected, "{failure:?}");
        assert_eq!(endpoint.outage(), Some(error.as_str()));
    }
}

/// The worker stops the handle before its disconnect state reaches the inbox,
/// so a poll can find the connection gone while `live` still holds the old
/// snapshot. An activation budget must not run out in that gap either.
#[gpui::test]
fn a_drop_seen_before_its_state_arrives_does_not_fall_back_to_local(cx: &mut gpui::TestAppContext) {
    let (fixture, cx) = cx.add_window_view(|window, cx| {
        Fixture(cx.new(|cx| crate::sidebar::layout_tests::fixture_window(window, cx)))
    });
    let view = fixture.update(cx, |fixture, _| fixture.0.clone());
    let (endpoint, _server) = connected_endpoint("ssh:remote");
    view.update(cx, |view, cx| {
        prepare_mouse(view, endpoint, cx);
        view.endpoints[1].set_retry_at_for_test(Instant::now() + Duration::from_secs(120));
        // Keep an acknowledged frame while its presentation budget expires.
        let snapshot = view.live.snapshot.as_ref().unwrap();
        let mut activation = crate::state::activation::Fence::Open;
        activation.transition(crate::state::activation::Transition::Queued {
            result: Ok("acknowledged".into()), boot: snapshot.boot_id.clone(), focus: None, active: true,
            now: Instant::now() - ACTIVATION_TIMEOUT,
        });
        activation.transition(crate::state::activation::Transition::Response {
            request: "acknowledged", result: Some(&serde_json::json!({
                "type": "client_shell_surface_set", "active": true, "projection_revision": snapshot.revision,
            })),
        });
        let inbox = view.endpoints[1].connection.inbox.clone();
        let mut pending = inbox.lock().unwrap();
        pending.activation = activation.clone();
        view.endpoints[1].live.activation = activation.clone();
        view.live.activation = activation;
        // Hold the reader's disconnect back to establish the gap before it lands.
        view.endpoints[1]
            .connection
            .handle
            .as_ref()
            .unwrap()
            .disconnect();
        view.poll_endpoints(cx);
        assert_eq!(
            view.selected_endpoint, 1,
            "a drop must not fall back to Local"
        );
        assert!(view.endpoints[1].connection.handle.is_none());
        assert!(view.live.snapshot.is_some(), "the gap under test");
        assert!(view.live.activation.deadline().unwrap() > Instant::now());
        // `live` still holds the lost connection's ready frame; painting it
        // must not undo the dimming.
        assert!(view.live.surface_ready());
        assert!(view.presentation.picture(&view.live).is_some());
        assert!(view.presentation.stale());
        assert!(view.reconnecting());
        drop(pending);
    });
}

#[test]
fn a_stopped_handle_reads_as_reconnecting_before_its_state_arrives() {
    let (mut endpoint, _server) = connected_endpoint("ssh:remote");
    let inbox = endpoint.connection.inbox.clone();
    let pending = inbox.lock().unwrap();
    // Hold the reader's disconnect back while the stopped handle is visible.
    endpoint.connection.handle.as_ref().unwrap().disconnect();
    endpoint.poll(Instant::now());
    assert!(endpoint.live.status.is_connected(), "the gap under test");
    assert_eq!(endpoint.outage(), Some("connection lost"));
    assert_eq!(endpoint.status(), "reconnecting");
    drop(pending);
}

/// The refusal can reach the inbox a poll after the stop it explains. It must
/// still hold the host to the longest delay, and name the reason.
#[test]
fn a_refusal_that_arrives_after_its_stop_still_waits_the_longest_delay() {
    let (mut endpoint, _server) = connected_endpoint("ssh:remote");
    endpoint.connection.handle.as_ref().unwrap().disconnect();
    let dropped = Instant::now();
    endpoint.poll(dropped);
    assert_eq!(endpoint.outage(), Some("connection lost"));
    assert_eq!(endpoint.retry_at(), dropped + endpoint.retry_delay());

    let error =
        spaceport_client::Error::Ssh(spaceport_client::ssh::Error::Refused(SshFailure::HostKey))
            .to_string();
    endpoint.connection.inbox.lock().unwrap().apply(
        ClientEvent::Disconnected {
            reason: error.clone(),
            ssh: Some(SshFailure::HostKey),
        },
        &crate::state::labels(),
    );
    let arrived = dropped + Duration::from_millis(16);
    endpoint.poll(arrived);
    assert_eq!(endpoint.outage(), Some(error.as_str()));
    assert_eq!(endpoint.retry_at(), arrived + MAX_RETRY_DELAY);
}

/// The replacement connection's snapshot ends the outage before its first
/// frame arrives. The old picture must stay dimmed, under the card, until then.
#[gpui::test]
fn the_card_stays_until_the_replacement_frame_lands(cx: &mut gpui::TestAppContext) {
    let (fixture, cx) = cx.add_window_view(|window, cx| {
        Fixture(cx.new(|cx| crate::sidebar::layout_tests::fixture_window(window, cx)))
    });
    let view = fixture.update(cx, |fixture, _| fixture.0.clone());
    let (endpoint, server) = connected_endpoint("ssh:remote");
    let first = view.update(cx, |view, cx| {
        prepare_mouse(view, endpoint, cx);
        view.presentation.picture(&view.live).unwrap().frame
    });
    // Close the real peer before rebinding this endpoint's private address.
    drop(server);
    view.update(cx, |view, cx| {
        wait_until(|| {
            let endpoint = &mut view.endpoints[1];
            endpoint.poll(Instant::now());
            endpoint.connection.handle.is_none() && endpoint.connection.retry_ready()
        });
        view.endpoints[1].set_retry_at_for_test(Instant::now() + Duration::from_secs(120));
        view.poll_endpoints(cx);
        assert!(view.presentation.stale());

        let mut replacement = start_endpoint(&mut view.endpoints[1]);
        // Project the new generation while its welcome is still withheld.
        view.poll_endpoints(cx);
        welcome_peer(&mut replacement, &snapshot(), None);
        project_until(view, cx, "the replacement snapshot", |view| {
            view.live.snapshot.is_some()
        });
        assert!(view.endpoints[1].outage().is_none());
        assert!(view.live.surface.is_none());
        let shown = view.presentation.picture(&view.live).unwrap().frame;
        assert!(Arc::ptr_eq(&shown, &first));
        assert!(view.presentation.stale());
        assert!(view.reconnecting(), "the card outlives the outage");

        let mut frame = (*surface(view.live.snapshot.as_ref().unwrap())).clone();
        frame.surface_revision = first.surface_revision + 1;
        frame.frame.cells.resize_with(
            usize::from(frame.frame.width) * usize::from(frame.frame.height),
            || CellData {
                symbol: " ".into(),
                fg: 0,
                bg: 0,
                modifier: 0,
                skip: false,
                hyperlink: None,
            },
        );
        write_message(
            &mut replacement.stream,
            &ServerMessage::PaneSurface(frame),
            MAX_GRAPHICS_FRAME_SIZE,
        )
        .unwrap();
        project_until(view, cx, "the replacement frame", |view| {
            view.live.surface_ready()
        });
        view.presentation.picture(&view.live);
        assert!(!view.presentation.stale());
        assert!(!view.reconnecting());
    });
}
