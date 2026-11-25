// ABOUTME: Preserves a new selection's budget while earlier release receipts arrive.
// ABOUTME: Exercises expired inactive acknowledgements and exact source-release ownership.
use super::*;
use crate::state::activation::{Fence, Transition};

#[gpui::test]
fn reselection_keeps_its_budget_across_an_old_inactive_receipt(cx: &mut gpui::TestAppContext) {
    let (fixture, cx) = cx.add_window_view(|window, cx| {
        Fixture(cx.new(|cx| crate::sidebar::layout_tests::fixture_window(window, cx)))
    });
    let view = fixture.update(cx, |fixture, _| fixture.0.clone());
    let (source, mut source_server) = connected_endpoint("ssh:source");
    let (mut target, mut target_server) = connected_endpoint("ssh:target");
    let previous = target.live().snapshot.as_ref().unwrap();
    let mut old = Fence::Open;
    old.transition(Transition::Queued {
        result: Ok("previous-release".into()),
        boot: previous.boot_id.clone(),
        focus: None,
        active: false,
        now: Instant::now() - Duration::from_secs(60),
    });
    old.transition(Transition::Response {
        request: "previous-release",
        result: Some(&serde_json::json!({
            "type": "client_shell_surface_set", "active": false, "projection_revision": previous.revision,
        })),
    });
    target.connection.inbox.lock().unwrap().activation = old.clone();
    target.live_mut().activation = old;
    target.initial_surface = false;
    let inbox = target.connection.inbox.clone();
    view.update(cx, |view, cx| {
        prepare_mouse(view, source, cx);
        view.endpoints.push(target);
        view.select_endpoint("ssh:target", cx).unwrap();
    });
    assert!(matches!(
        source_server.receive(),
        ClientMessage::ClientShellFocus { focused: false }
    ));
    let ClientMessage::ClientShellEndpointRequest { request, .. } = source_server.receive() else {
        panic!("missing exact source release");
    };
    let release: serde_json::Value = serde_json::from_str(&request).unwrap();
    view.update(cx, |view, cx| {
        inbox.lock().unwrap().dirty = true;
        view.poll_endpoints(cx);
        assert_eq!(view.endpoints[view.selected_endpoint].id, "ssh:target");
        assert!(view.live.activation.deadline().unwrap() > Instant::now());
        assert!(!view.input_ready());
        assert_eq!(view.pending_releases.len(), 1);
        assert!(!view.endpoints[2].initial_surface);
    });
    let id = release["id"].as_str().unwrap();
    write_message(&mut source_server.stream, &ServerMessage::ClientShellEndpointResponseChunk {
        boot_id: snapshot().boot_id,
        request_id: id.into(),
        final_chunk: true,
        data: serde_json::to_vec(&serde_json::json!({"id": id, "result": {
            "type": "client_shell_surface_set", "active": false, "projection_revision": snapshot().revision,
        }})).unwrap(),
    }, MAX_GRAPHICS_FRAME_SIZE).unwrap();
    view.update(cx, |view, cx| {
        poll_until(view, cx, |view, cx| {
            view.poll_endpoints(cx);
            assert_eq!(view.endpoints[view.selected_endpoint].id, "ssh:target");
            view.endpoints[2].initial_surface
        });
    });
    assert!(matches!(
        target_server.receive(),
        ClientMessage::ClientShellResize { .. }
    ));
    let ClientMessage::ClientShellEndpointRequest { request, .. } = target_server.receive() else {
        panic!("missing destination activation");
    };
    let activation: serde_json::Value = serde_json::from_str(&request).unwrap();
    assert_eq!(activation["method"], Method::ClientShellSurfaceSet.as_str());
    assert_eq!(activation["params"]["active"], true);
}
