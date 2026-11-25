// ABOUTME: Exercises terminal Find through the ordered Herdr client.
// ABOUTME: Checks server match coordinates, original text, and stale target rejection.
use super::*;

fn endpoint(peer: &mut Stream, method: &str) -> (String, Value) {
    let ClientMessage::ClientShellEndpointRequest { request, .. } = read(peer) else {
        panic!("expected an endpoint request")
    };
    let body: Value = serde_json::from_str(&request).unwrap();
    assert_eq!(body["method"], method);
    (body["id"].as_str().unwrap().into(), body["params"].clone())
}

#[test]
fn history_match_reads_original_text_and_context_at_one_revision() {
    let mut frame = surface();
    frame.frame = cells(&" ".repeat(160), 80);
    frame.panes[0].inner_rect.width = 79;
    frame.panes[0].rect.width = 79;
    let (session, mut peer) = peer_with_surface(frame);
    let target = session.poll(0).unwrap().input_target.unwrap();
    let task = std::thread::spawn(move || {
        session
            .history_search(target)?
            .find("needle".into(), false, None)
    });
    let (id, params) = endpoint(&mut peer, "pane.copy_search");
    assert_eq!(params["content_revision"], 2);
    assert_eq!(params["direction"], "backward");
    let span = json!({"start":{"row":7,"col":2},"end":{"row":7,"col":7}});
    reply(
        &mut peer,
        &id,
        json!({"type":"pane_copy_search","pane_id":"w1:p1","content_revision":2,
        "matches":[span],"total":2,"current":0,"current_global":1}),
    );
    for (start, end, text) in [
        (2, 7, "NEEDLE"),
        (0, 7, "界NEEDLE"),
        (2, 71, "NEEDLE second"),
    ] {
        if end == 71 {
            let (id, motion) = endpoint(&mut peer, "pane.copy_motion");
            assert_eq!(motion["motion"], "line_end");
            assert_eq!(motion["content_revision"], 2);
            reply(
                &mut peer,
                &id,
                json!({"type":"pane_copy_motion","pane_id":"w1:p1","content_revision":2,"cursor":{"row":7,"col":77}}),
            );
        }
        let (id, params) = endpoint(&mut peer, "pane.selection.read");
        assert_eq!(params["anchor"], json!({"row":7,"col":start}));
        assert_eq!(params["cursor"], json!({"row":7,"col":end}));
        assert_eq!(params["content_revision"], 2);
        reply(
            &mut peer,
            &id,
            json!({"type":"pane_selection","pane_id":"w1:p1","text":text}),
        );
    }
    let result = task.join().unwrap().unwrap().unwrap();
    assert_eq!((result.index, result.total), (2, 2));
    assert_eq!(
        (
            result.before.as_str(),
            result.matched.as_str(),
            result.after.as_str()
        ),
        ("界", "NEEDLE", " second")
    );
}

#[test]
fn history_search_rejects_target_from_another_connection() {
    let (session, _peer) = peer();
    let mut target = session.poll(0).unwrap().input_target.unwrap();
    target.connection += 1;
    assert!(matches!(
        session.history_search(target),
        Err(MobileError::StaleView)
    ));
}

#[test]
fn a_search_waits_for_a_settled_frame_instead_of_asking_about_a_write_in_progress() {
    let mut writing = surface();
    // An odd revision is a write in progress, which the daemon would refuse:
    // the window never sends such a request, and neither does the phone.
    writing.panes[0].content_revision = 3;
    let (session, mut peer) = peer_with_surface(writing);
    let target = session.poll(0).unwrap().input_target.unwrap();
    let task = std::thread::spawn(move || {
        session
            .history_search(target)?
            .find("needle".into(), false, None)
    });
    let mut settled = surface();
    settled.surface_revision += 1;
    settled.panes[0].content_revision = 4;
    send(&mut peer, ServerMessage::PaneSurface(settled));
    let (id, params) = endpoint(&mut peer, "pane.copy_search");
    assert_eq!(params["content_revision"], 4);
    reply(
        &mut peer,
        &id,
        json!({"type":"pane_copy_search","pane_id":"w1:p1","content_revision":4,"matches":[],"total":0}),
    );
    assert!(task.join().unwrap().unwrap().is_none());
}

#[test]
fn history_retries_moved_content_using_the_next_surface_revision() {
    let (session, mut peer) = peer();
    let target = session.poll(0).unwrap().input_target.unwrap();
    let task = std::thread::spawn(move || {
        session
            .history_search(target)?
            .find("needle".into(), false, None)
    });
    let (id, _) = endpoint(&mut peer, "pane.copy_search");
    let mut frame = surface();
    frame.surface_revision += 1;
    frame.panes[0].content_revision = 4;
    send(&mut peer, ServerMessage::PaneSurface(frame));
    send(
        &mut peer,
        ServerMessage::ClientShellEndpointResponseChunk {
            boot_id: "boot-v1".into(),
            request_id: id.clone(),
            final_chunk: true,
            data: serde_json::to_vec(
                &json!({"id":id, "error":{"code":"stale_content","message":"changed"}}),
            )
            .unwrap(),
        },
    );
    let (id, params) = endpoint(&mut peer, "pane.copy_search");
    assert_eq!(params["content_revision"], 4);
    assert!(params.get("previous").is_none());
    reply(
        &mut peer,
        &id,
        json!({"type":"pane_copy_search","pane_id":"w1:p1","content_revision":4,"matches":[],"total":0}),
    );
    assert!(task.join().unwrap().unwrap().is_none());
}

#[test]
fn superseding_history_drops_old_waiter_and_ignores_its_late_reply() {
    let (session, mut peer) = peer();
    let target = session.poll(0).unwrap().input_target.unwrap();
    let first_search = session.history_search(target.clone()).unwrap();
    let worker = first_search.clone();
    let first = std::thread::spawn(move || worker.find("one".into(), false, None));
    let (old, _) = endpoint(&mut peer, "pane.copy_search");
    let second_search = session.history_search(target).unwrap();
    assert!(matches!(first.join().unwrap(), Err(MobileError::StaleView)));
    // A delayed Swift cancellation handler for the first sheet cannot cancel the new one.
    first_search.cancel();
    let second = std::thread::spawn(move || second_search.find("two".into(), false, None));
    let span = json!({"start":{"row":2,"col":0},"end":{"row":2,"col":2}});
    reply(
        &mut peer,
        &old,
        json!({"type":"pane_copy_search","pane_id":"w1:p1",
        "content_revision":2,"matches":[span],"total":1,"current":0,"current_global":0}),
    );
    let (new, params) = endpoint(&mut peer, "pane.copy_search");
    assert_eq!(params["query"], "two");
    reply(
        &mut peer,
        &new,
        json!({"type":"pane_copy_search","pane_id":"w1:p1","content_revision":2,"matches":[],"total":0}),
    );
    assert!(second.join().unwrap().unwrap().is_none());
    assert!(session.poll(0).unwrap().error.is_none());
}

#[test]
fn timed_out_history_allows_a_new_query_and_absorbs_the_late_host_error() {
    let (session, mut peer) = peer();
    let target = session.poll(0).unwrap().input_target.unwrap();
    let search = session.history_search(target.clone()).unwrap();
    let first = std::thread::spawn(move || search.find("slow".into(), false, None));
    let (old, _) = endpoint(&mut peer, "pane.copy_search");
    assert!(matches!(
        first.join().unwrap(),
        Err(MobileError::HistoryTimeout)
    ));
    let search = session.history_search(target).unwrap();
    let second = std::thread::spawn(move || search.find("retry".into(), false, None));
    send(
        &mut peer,
        ServerMessage::ClientShellEndpointResponseChunk {
            boot_id: "boot-v1".into(),
            request_id: old.clone(),
            final_chunk: true,
            data: serde_json::to_vec(
                &json!({"id":old,"error":{"code":"stale_content","message":"late"}}),
            )
            .unwrap(),
        },
    );
    let (new, params) = endpoint(&mut peer, "pane.copy_search");
    assert_eq!(params["query"], "retry");
    reply(
        &mut peer,
        &new,
        json!({"type":"pane_copy_search","pane_id":"w1:p1","content_revision":2,"matches":[],"total":0}),
    );
    assert!(second.join().unwrap().unwrap().is_none());
    assert!(session.poll(0).unwrap().error.is_none());
}
