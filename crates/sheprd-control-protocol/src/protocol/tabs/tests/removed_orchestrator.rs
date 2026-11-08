use super::*;

#[test]
fn orchestrator_wire_is_rejected_and_device_target_round_trips() {
    assert!(serde_json::from_str::<TabOp>(r#"{"op":"orchestrator","repo":"/src/app"}"#).is_err());
    assert!(
        serde_json::from_str::<Request>(
            r#"{"method":"tab","caller":{"workspace_id":"w4"},"op":{"op":"orchestrator"}}"#
        )
        .is_err()
    );

    let request = Request::Tab(Call {
        caller: Caller {
            daemon_socket: Some("/private/core.sock".into()),
            window: Some(7),
            workspace_id: Some("w4".into()),
            ..Caller::default()
        },
        op: TabOp::Devices,
    });
    let wire = serde_json::to_string(&request).unwrap();
    assert_eq!(serde_json::from_str::<Request>(&wire).unwrap(), request);
}
