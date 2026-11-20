// ABOUTME: Exercises ordered response assembly and session revision fences.
// ABOUTME: Checks exact bounds and typed timeout or correlation failures.
use super::*;

#[test]
fn session_response_slot_correlates_chunks_and_clears_only_on_completion() {
    let mut session = ready_session();
    let chunk =
        |id: &str, final_chunk, data: &[u8]| ServerMessage::ClientShellEndpointResponseChunk {
            boot_id: "boot-v1".into(),
            request_id: id.into(),
            final_chunk,
            data: data.into(),
        };
    let no_event = |_| -> Result<()> { panic!("unexpected event") };
    assert_eq!(
        session
            .handle_message(chunk("one", true, b"{}"), no_event)
            .unwrap_err()
            .to_string(),
        "Herdr sent a reply without a matching request. Choose Terminal → Reconnect."
    );
    session.pending = Some(Pending {
        id: "one".into(),
        method: Method::PaneFocus,
        bytes: Vec::new(),
        started: Instant::now(),
    });
    session
        .handle_message(chunk("one", false, br#"{"id":"one","result":"#), no_event)
        .unwrap();
    let partial = session.pending.as_ref().unwrap().bytes.clone();
    assert_eq!(
        session
            .handle_message(chunk("other", true, b"null}"), no_event)
            .unwrap_err()
            .to_string(),
        "Herdr sent a reply without a matching request. Choose Terminal → Reconnect."
    );
    assert_eq!(session.pending.as_ref().unwrap().bytes, partial);
    let mut events = Vec::new();
    session
        .handle_message(chunk("one", true, b"null}"), |event| {
            events.push(event);
            Ok(())
        })
        .unwrap();
    assert!(session.pending.is_none());
    assert!(
        matches!(events.as_slice(), [ClientEvent::Response { request_id, response }]
        if request_id == "one" && *response == json!({"id": "one", "result": null}))
    );
    assert_eq!(
        session
            .handle_message(chunk("one", true, b"{}"), no_event)
            .unwrap_err()
            .to_string(),
        "Herdr sent a reply without a matching request. Choose Terminal → Reconnect."
    );
}

#[test]
fn session_response_limit_counts_previous_chunks_and_allows_exact_limit() {
    for overflow in [false, true] {
        let mut session = ready_session();
        let response = br#"{"id":"one"}"#;
        session.pending = Some(Pending {
            id: "one".into(),
            method: Method::PaneFocus,
            bytes: vec![b' '; MAX_RESPONSE_BYTES - response.len()],
            started: Instant::now(),
        });
        let mut data = response.to_vec();
        if overflow {
            data.push(b' ');
        }
        let mut events = Vec::new();
        let result = session.handle_message(
            ServerMessage::ClientShellEndpointResponseChunk {
                boot_id: "boot-v1".into(),
                request_id: "one".into(),
                final_chunk: true,
                data,
            },
            |event| {
                events.push(event);
                Ok(())
            },
        );
        if overflow {
            assert!(matches!(result, Err(Error::ResponseLimit)));
            assert!(events.is_empty());
            assert_eq!(
                session.pending.unwrap().bytes.len(),
                MAX_RESPONSE_BYTES - response.len()
            );
        } else {
            result.unwrap();
            assert!(session.pending.is_none());
            assert!(matches!(events.as_slice(), [ClientEvent::Response { .. }]));
        }
    }
}

#[test]
fn session_deadlines_and_snapshot_revision_fence() {
    let mut session = Session::new(true, false);
    session.started = Instant::now() - TIMEOUT - POLL;
    assert!(matches!(
        session.check_timeouts(),
        Err(Error::HandshakeTimeout)
    ));
    session
        .handle_message(
            ServerMessage::EndpointControl {
                kind: ENDPOINT_WELCOME_KIND.into(),
                data: WELCOME.into(),
            },
            |_| Ok(()),
        )
        .unwrap();
    assert!(session.check_timeouts().is_err()); // Welcome alone is not ready.

    let mut session = ready_session();
    session.started = Instant::now() - TIMEOUT - POLL;
    session.check_timeouts().unwrap();
    session.pending = Some(Pending {
        id: "one".into(),
        method: Method::PaneFocus,
        bytes: Vec::new(),
        started: Instant::now() - COMMAND_TIMEOUT - POLL,
    });
    assert!(matches!(
        session.check_timeouts(),
        Err(Error::RequestTimeout)
    ));

    let mut snapshot: Value = serde_json::from_str(SNAPSHOT).unwrap();
    snapshot["revision"] = json!(6);
    let error = session
        .handle_message(
            ServerMessage::EndpointControl {
                kind: ENDPOINT_SNAPSHOT_KIND.into(),
                data: snapshot.to_string(),
            },
            |_| panic!("regressed snapshot must not be published"),
        )
        .unwrap_err();
    assert!(matches!(error, Error::SnapshotIdentity));
    assert_eq!(session.snapshot.unwrap().revision, 7);
}

#[test]
fn response_boot_id_correlation_and_assembly_limits() {
    for case in ["boot", "id", "limit"] {
        let (client, mut server, worker) = test_client();
        handshake(&mut server);
        event(&client);
        event(&client);
        let id = client.handle.focus_pane("boot-v1", "w1:p1").unwrap();
        receive(&mut server);
        let data = match case {
            "limit" => vec![b' '; MAX_RESPONSE_BYTES + 1],
            "id" => br#"{"id":"wrong","result":{}}"#.to_vec(),
            _ => vec![],
        };
        send(
            &mut server,
            ServerMessage::ClientShellEndpointResponseChunk {
                boot_id: if case == "boot" { "stale" } else { "boot-v1" }.into(),
                request_id: id,
                final_chunk: true,
                data,
            },
        );
        let error = worker.join().unwrap().unwrap_err();
        assert!(
            matches!(
                (case, &error),
                ("boot", Error::ResponseBoot)
                    | ("id", Error::ResponseId)
                    | ("limit", Error::ResponseLimit)
            ),
            "{error}"
        );
    }
}
