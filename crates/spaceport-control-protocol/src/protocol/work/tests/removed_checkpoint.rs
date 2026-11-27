// ABOUTME: Rejects retired checkpoint restore requests on the work wire.
// ABOUTME: Checks retained work requests keep the caller's exact identity.
use super::*;

#[test]
fn restore_wire_op_is_rejected_without_changing_caller_identity() {
    for old in [
        r#"{"method":"work","caller":{"workspace_id":"w4"},"op":{"op":"restore"}}"#,
        r#"{"method":"work","caller":{"workspace_id":"w4"},"op":{"op":"restore","id":"123"}}"#,
    ] {
        assert!(serde_json::from_str::<Request>(old).is_err());
    }

    let retained = Request::Work(Call {
        caller: Caller {
            workspace_id: Some("w4".into()),
            window: Some(7),
            ..Caller::default()
        },
        op: WorkOp::Notes,
    });
    let wire = serde_json::to_string(&retained).unwrap();
    assert_eq!(serde_json::from_str::<Request>(&wire).unwrap(), retained);
}
