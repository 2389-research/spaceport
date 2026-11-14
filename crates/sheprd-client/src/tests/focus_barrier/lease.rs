//! The request lease while an activation waits behind focus: responses in
//! either order, the bound both assemblies share, and what a refusal or a
//! cancellation leaves behind.

use super::*;

fn chunk(id: &str, final_chunk: bool, data: Vec<u8>) -> ServerMessage {
    ServerMessage::ClientShellEndpointResponseChunk {
        boot_id: "boot-v1".into(),
        request_id: id.into(),
        final_chunk,
        data,
    }
}

fn pending(id: &str, method: Method) -> Pending {
    Pending {
        id: id.into(),
        method,
        bytes: Vec::new(),
        started: Instant::now(),
    }
}

/// A session with `focus` on the wire and `activation` written behind it.
fn pipelined_session() -> Session {
    let mut session = ready_session();
    session.pipeline_barriers = true;
    session.pending = Some(pending("focus", Method::PaneFocus));
    session.barrier = Some(session::Barrier::new(
        pending("activation", Method::ClientShellSurfaceSet),
        Vec::new(),
    ));
    session
}

fn padded(json: &str, len: usize) -> Vec<u8> {
    let mut bytes = json.as_bytes().to_vec();
    bytes.resize(len, b' ');
    bytes
}

fn responses(events: &[ClientEvent]) -> Vec<(&str, &Value)> {
    events
        .iter()
        .map(|event| match event {
            ClientEvent::Response {
                request_id,
                response,
            } => (request_id.as_str(), response),
            other => panic!("unexpected {other:?}"),
        })
        .collect()
}

fn busy() -> Value {
    json!({"error": {"code": "endpoint_busy", "message": "busy"}})
}

#[test]
fn an_activation_answered_after_focus_is_delivered_alone_and_keeps_the_lease_until_then() {
    let (client, mut server, worker) = connected();
    let focus = client.handle.focus_pane("boot-v1", "w1:p2").unwrap();
    let activation = client.handle.set_surface_active("boot-v1", true).unwrap();
    let next = client.handle.focus_pane("boot-v1", "w1:p3").unwrap();
    assert_eq!(request(&mut server)["id"], focus);
    assert_eq!(request(&mut server)["id"], activation);
    respond(
        &mut server,
        "boot-v1",
        &focus,
        json!({"result": {"focused": true}}),
    );
    assert_eq!(
        response(&client),
        (
            focus.clone(),
            json!({"id": focus, "result": {"focused": true}})
        )
    );
    // The unanswered activation still holds the lease.
    silent(&mut server);
    assert!(client.events.try_recv().is_err());
    respond(&mut server, "boot-v1", &activation, activated(9));
    let mut expected = json!({"id": activation});
    expected["result"] = activated(9)["result"].clone();
    assert_eq!(response(&client), (activation, expected));
    assert_eq!(request(&mut server)["id"], next);
    assert!(client.events.try_recv().is_err());
    client.handle.disconnect();
    worker.join().unwrap().unwrap();
}

#[test]
fn both_unfinished_responses_share_the_bound_and_each_is_delivered_when_it_ends() {
    for excess in [0, 1] {
        let mut session = pipelined_session();
        let no_event = |_| -> Result<()> { panic!("neither response is complete") };
        let half = MAX_RESPONSE_BYTES / 2;
        session
            .handle_message(
                chunk(
                    "activation",
                    false,
                    padded(r#"{"id":"activation","result":7}"#, half),
                ),
                no_event,
            )
            .unwrap();
        session
            .handle_message(
                chunk(
                    "focus",
                    false,
                    padded(r#"{"id":"focus","result":{}}"#, half - 8),
                ),
                no_event,
            )
            .unwrap();
        let mut events = Vec::new();
        let result =
            session.handle_message(chunk("focus", true, vec![b' '; 8 + excess]), |event| {
                events.push(event);
                Ok(())
            });
        if excess == 1 {
            assert!(matches!(result, Err(Error::ResponseLimit)));
            assert!(events.is_empty());
            assert_eq!(session.pending.as_ref().unwrap().bytes.len(), half - 8);
            assert_eq!(session.barrier.as_ref().unwrap().pending.bytes.len(), half);
            continue;
        }
        result.unwrap();
        assert_eq!(
            responses(&events),
            [("focus", &json!({"id": "focus", "result": {}}))]
        );
        assert!(session.pending.is_none());
        assert_eq!(session.barrier.as_ref().unwrap().pending.bytes.len(), half);
        events.clear();
        session
            .handle_message(chunk("activation", true, Vec::new()), |event| {
                events.push(event);
                Ok(())
            })
            .unwrap();
        assert_eq!(
            responses(&events),
            [("activation", &json!({"id": "activation", "result": 7}))]
        );
        assert!(session.barrier.is_none());
    }
}

#[test]
fn a_refusal_releases_its_bytes_and_a_held_response_is_not_answered_twice() {
    let mut session = pipelined_session();
    let no_event = |_| -> Result<()> { panic!("a refusal is not reported") };
    let refusal = r#"{"id":"activation","error":{"code":"endpoint_busy","message":"busy"}}"#;
    session
        .handle_message(
            chunk("activation", true, padded(refusal, MAX_RESPONSE_BYTES / 2)),
            no_event,
        )
        .unwrap();
    assert!(!session.pipeline_barriers);
    assert!(!session.resend_ready());
    // The activation is no longer awaited: a second answer has no request.
    assert!(matches!(
        session.handle_message(chunk("activation", true, b"{}".to_vec()), no_event),
        Err(Error::UnsolicitedResponse)
    ));
    let mut events = Vec::new();
    session
        .handle_message(
            chunk(
                "focus",
                true,
                padded(r#"{"id":"focus","result":{}}"#, MAX_RESPONSE_BYTES),
            ),
            |event| {
                events.push(event);
                Ok(())
            },
        )
        .unwrap();
    assert_eq!(
        responses(&events),
        [("focus", &json!({"id": "focus", "result": {}}))]
    );
    assert!(session.resend_ready());
    assert!(session.take_refused_barrier().is_some());
    assert!(session.barrier.is_none() && !session.resend_ready());

    let mut session = pipelined_session();
    session
        .handle_message(
            chunk(
                "activation",
                true,
                br#"{"id":"activation","result":{}}"#.to_vec(),
            ),
            |_| -> Result<()> { panic!("held until focus completes") },
        )
        .unwrap();
    assert!(session.pipeline_barriers);
    assert!(!session.resend_ready() && session.take_refused_barrier().is_none());
    assert!(matches!(
        session.handle_message(
            chunk("activation", true, b"{}".to_vec()),
            |_| -> Result<()> { panic!("already answered") }
        ),
        Err(Error::UnsolicitedResponse)
    ));
}

#[test]
fn a_refused_activation_is_resent_once_and_a_second_refusal_is_delivered() {
    for refused_first in [true, false] {
        let (client, mut server, worker) = connected();
        let focus = client.handle.focus_pane("boot-v1", "w1:p2").unwrap();
        let activation = client.handle.set_surface_active("boot-v1", true).unwrap();
        request(&mut server);
        let original = request(&mut server);
        if refused_first {
            respond(&mut server, "boot-v1", &activation, busy());
            silent(&mut server);
            assert!(client.events.try_recv().is_err());
            respond(&mut server, "boot-v1", &focus, json!({"result": {}}));
            assert_eq!(response(&client).0, focus);
        } else {
            respond(&mut server, "boot-v1", &focus, json!({"result": {}}));
            assert_eq!(response(&client).0, focus);
            respond(&mut server, "boot-v1", &activation, busy());
        }
        assert_eq!(request(&mut server), original, "{refused_first}");
        respond(&mut server, "boot-v1", &activation, busy());
        let (id, refusal) = response(&client);
        assert_eq!(id, activation);
        assert_eq!(refusal["error"]["code"], "endpoint_busy");
        // Neither request is written again.
        silent(&mut server);
        assert!(client.events.try_recv().is_err());
        client.handle.disconnect();
        worker.join().unwrap().unwrap();
        assert_eq!(server.read(&mut [0]).unwrap(), 0);
    }
}

#[test]
fn cancellation_drops_a_held_activation_response_and_what_waits_behind_it() {
    let (client, mut server, worker) = connected();
    client.handle.focus_pane("boot-v1", "w1:p2").unwrap();
    let activation = client.handle.set_surface_active("boot-v1", true).unwrap();
    client.handle.focus_pane("boot-v1", "w1:p3").unwrap();
    request(&mut server);
    request(&mut server);
    respond(&mut server, "boot-v1", &activation, activated(9));
    // Held behind focus: neither delivered nor releasing the next request.
    silent(&mut server);
    assert!(client.events.try_recv().is_err());
    client.handle.disconnect();
    worker.join().unwrap().unwrap();
    assert_eq!(server.read(&mut [0]).unwrap(), 0);
    assert!(client.events.try_recv().is_err());
}

#[test]
fn a_held_activation_follows_a_chunked_focus_response_with_both_bodies_intact() {
    let mut session = pipelined_session();
    let no_event = |_| -> Result<()> { panic!("held until focus completes") };
    let activation = br#"{"id":"activation","result":{"type":"client_shell_surface_set","active":true,"projection_revision":9}}"#;
    session
        .handle_message(chunk("activation", true, activation.to_vec()), no_event)
        .unwrap();
    assert!(session.pending.is_some() && session.barrier.is_some());
    session
        .handle_message(
            chunk(
                "focus",
                false,
                br#"{"id":"focus","result":{"pane_"#.to_vec(),
            ),
            no_event,
        )
        .unwrap();
    let mut events = Vec::new();
    session
        .handle_message(
            chunk("focus", true, br#"id":"w1:p2","focused":true}}"#.to_vec()),
            |event| {
                events.push(event);
                Ok(())
            },
        )
        .unwrap();
    assert_eq!(
        responses(&events),
        [
            (
                "focus",
                &json!({"id": "focus", "result": {"pane_id": "w1:p2", "focused": true}})
            ),
            (
                "activation",
                &json!({"id": "activation", "result": {
                    "type": "client_shell_surface_set", "active": true, "projection_revision": 9
                }})
            ),
        ]
    );
    assert!(session.pending.is_none() && session.barrier.is_none());
    assert!(session.pipeline_barriers && !session.resend_ready());
}
