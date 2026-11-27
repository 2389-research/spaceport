use super::*;

#[test]
fn cloud_wire_routes_are_rejected_while_host_target_stays_exact() {
    for old in [
        r#"{"method":"cloud","caller":{},"op":{"op":"batches"}}"#,
        r#"{"method":"host","caller":{},"op":{"op":"cloud"}}"#,
        r#"{"method":"host","caller":{},"op":{"op":"cloud_setup","id":"box"}}"#,
        r#"{"method":"host","caller":{},"op":{"op":"cloud_images","provider":null}}"#,
    ] {
        assert!(serde_json::from_str::<Request>(old).is_err(), "{old}");
    }
    assert!(
        "cloud_batch"
            .parse::<crate::protocol::events::EventKind>()
            .is_err()
    );

    let retained = Request::Host(Call {
        caller: Caller {
            daemon_socket: Some("/private/core.sock".into()),
            window: Some(7),
            workspace_id: Some("w4".into()),
            ..Caller::default()
        },
        op: HostOp::Forward { port: 3000 },
    });
    let wire = serde_json::to_string(&retained).unwrap();
    assert_eq!(serde_json::from_str::<Request>(&wire).unwrap(), retained);
}
