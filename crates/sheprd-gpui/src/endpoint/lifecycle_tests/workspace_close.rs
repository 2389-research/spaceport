// ABOUTME: Exercises workspace consent against an isolated framed Herdr TUI peer.
// ABOUTME: Verifies cancellation, held keys, stale targets, and exactly one queued request.
use super::*;

fn key(value: &'static str, held: bool) -> gpui::KeyDownEvent {
    gpui::KeyDownEvent {
        keystroke: gpui::Keystroke::parse(value).unwrap(),
        is_held: held,
        prefer_character_input: false,
    }
}

fn marker(view: &SheprdWindow, server: &mut Server) {
    view.endpoints[view.selected_endpoint]
        .connection
        .handle
        .as_ref()
        .unwrap()
        .send_input(
            &snapshot().boot_id,
            "w1:p1",
            [ClientPaneInputEvent::TextCommit(
                "workspace-close-marker".into(),
            )],
        )
        .unwrap();
    loop {
        match server.receive() {
            ClientMessage::ClientShellPaneInput { events, .. }
                if events
                    == [ClientPaneInputEvent::TextCommit(
                        "workspace-close-marker".into(),
                    )] =>
            {
                return;
            }
            ClientMessage::ClientShellEndpointRequest { request, .. } => {
                panic!("unconfirmed workspace request: {request}")
            }
            _ => {}
        }
    }
}

fn select(view: &mut SheprdWindow, endpoint: Endpoint, cx: &mut Context<SheprdWindow>) {
    view.endpoints.truncate(1);
    view.endpoints.push(endpoint);
    view.selected_endpoint = 1;
    view.options = ConnectOptions::default();
    view.reset_selected(cx);
    view.live
        .activation
        .transition(crate::state::activation::Transition::Reset(None));
    view.config.confirm_close_tab = false;
    view.config.confirm_close_pane = false;
    assert!(view.input_ready());
}

#[gpui::test]
fn workspace_close_waits_for_fresh_consent_and_sends_one_exact_request(
    cx: &mut gpui::TestAppContext,
) {
    let (fixture, cx) = cx.add_window_view(|window, cx| {
        Fixture(cx.new(|cx| crate::sidebar::layout_tests::fixture_window(window, cx)))
    });
    let view = fixture.update(cx, |fixture, _| fixture.0.clone());
    let (endpoint, mut server) = connected_endpoint("ssh:workspace-close");
    cx.update(|window, cx| {
        view.update(cx, |view, cx| {
            select(view, endpoint, cx);
            assert!(view.open_workspace_close("w1", window, cx));
            marker(view, &mut server);
            view.close_confirmation_key(&key("escape", false), window, cx);
            assert!(view.menu.page.is_none());
            marker(view, &mut server);
            assert!(view.open_workspace_close("w1", window, cx));
            view.close_confirmation_key(&key("tab", false), window, cx);
            view.close_confirmation_key(&key("enter", true), window, cx);
            assert_eq!(view.menu.page, Some(crate::menu::Page::ConfirmClose));
            marker(view, &mut server);
            view.close_confirmation_key(&key("enter", false), window, cx);
            assert!(view.menu.page.is_none());
        })
    });
    let ClientMessage::ClientShellEndpointRequest { request, .. } = server.receive() else {
        panic!("missing workspace close");
    };
    let request: serde_json::Value = serde_json::from_str(&request).unwrap();
    assert_eq!(request["method"], Method::WorkspaceClose.as_str());
    assert_eq!(
        request["params"],
        serde_json::json!({"workspace_id": "w1", "close_group": false})
    );
    // Release the real worker's single API lease before the ordered marker.
    server.respond(&request);
    let ClientMessage::ClientShellEndpointRequest { request, .. } = server.receive() else {
        panic!("missing workspace close surface barrier");
    };
    let barrier: serde_json::Value = serde_json::from_str(&request).unwrap();
    assert_eq!(barrier["method"], Method::ClientShellSurfaceSet.as_str());
    assert_eq!(barrier["params"]["active"], true);
    background_commands::acknowledge_activation(&mut server, &barrier);
    view.read_with(cx, |view, _| marker(view, &mut server));
}

#[gpui::test]
fn workspace_close_refuses_changed_sessions_and_targets(cx: &mut gpui::TestAppContext) {
    let (fixture, cx) = cx.add_window_view(|window, cx| {
        Fixture(cx.new(|cx| crate::sidebar::layout_tests::fixture_window(window, cx)))
    });
    let view = fixture.update(cx, |fixture, _| fixture.0.clone());
    for change in ["generation", "selection", "boot", "pane", "label", "locked"] {
        let (endpoint, mut server) = connected_endpoint("ssh:stale-workspace");
        cx.update(|window, cx| {
            view.update(cx, |view, cx| {
                view.dismiss_menu(window, cx);
                select(view, endpoint, cx);
                assert!(view.open_workspace_close("w1", window, cx));
                match change {
                    "generation" => view.endpoints[1].generation += 1,
                    "selection" => view.selection_epoch += 1,
                    "locked" => {}
                    _ => {
                        let snapshot = Arc::make_mut(view.live.snapshot.as_mut().unwrap());
                        match change {
                            "boot" => snapshot.boot_id.push('x'),
                            "pane" => snapshot.panes[0].pane_id.push('x'),
                            _ => snapshot.workspaces[0].label.push('x'),
                        }
                    }
                }
                let inbox = view.endpoints[1].connection.inbox.clone();
                let locked = (change == "locked").then(|| inbox.lock().unwrap());
                view.send_close(window, cx);
                assert_eq!(view.menu.page, Some(crate::menu::Page::ConfirmClose));
                assert!(view.pending_navigation.is_none());
                drop(locked);
                marker(view, &mut server);
            })
        });
    }
}

#[gpui::test]
fn workspace_close_checks_newer_published_members_before_the_next_ui_poll(
    cx: &mut gpui::TestAppContext,
) {
    let (fixture, cx) = cx.add_window_view(|window, cx| {
        Fixture(cx.new(|cx| crate::sidebar::layout_tests::fixture_window(window, cx)))
    });
    let view = fixture.update(cx, |fixture, _| fixture.0.clone());
    for pending in [false, true] {
        let (endpoint, mut server) = connected_endpoint("ssh:published-workspace-close");
        cx.update(|window, cx| {
            view.update(cx, |view, cx| {
                view.dismiss_menu(window, cx);
                select(view, endpoint, cx);
                assert!(view.open_workspace_close("w1", window, cx));
                let mut newer = (**view.live.snapshot.as_ref().unwrap()).clone();
                newer.revision += 1;
                let mut pane = newer.panes[0].clone();
                pane.pane_id = "w1:new-pane".into();
                if pending {
                    let mut tab = newer.tabs[0].clone();
                    tab.tab_id = "w1:other-tab".into();
                    tab.focused = false;
                    pane.tab_id = tab.tab_id.clone();
                    newer.tabs.push(tab);
                }
                newer.panes.push(pane);
                let mut inbox = view.endpoints[1].connection.inbox.lock().unwrap();
                inbox.apply(ClientEvent::Snapshot(Arc::new(newer)));
                assert_eq!(inbox.pending_snapshot.is_some(), pending);
                if pending {
                    assert_eq!(
                        inbox.snapshot.as_ref().unwrap().revision,
                        view.live.snapshot.as_ref().unwrap().revision
                    );
                }
                drop(inbox);
                view.send_close(window, cx);
                assert_eq!(view.menu.page, Some(crate::menu::Page::ConfirmClose));
                assert!(view.pending_navigation.is_none());
                marker(view, &mut server);
            })
        });
    }
}
