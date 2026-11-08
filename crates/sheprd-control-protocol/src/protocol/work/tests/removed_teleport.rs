use super::*;

#[test]
fn teleport_wire_op_is_rejected_without_changing_caller_identity() {
    let old = r#"{"method":"work","caller":{"workspace_id":"w4"},"op":{"op":"teleport","host":"other-host"}}"#;
    assert!(serde_json::from_str::<Request>(old).is_err());

    let retained = Request::Work(Call {
        caller: Caller {
            workspace_id: Some("w4".into()),
            ..Caller::default()
        },
        op: WorkOp::MuteStatus,
    });
    let wire = serde_json::to_string(&retained).unwrap();
    assert_eq!(serde_json::from_str::<Request>(&wire).unwrap(), retained);
}
