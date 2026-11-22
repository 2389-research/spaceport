// ABOUTME: Rejects the retired px0 editor operation on the private wire.
// ABOUTME: Keeps exact caller targeting for retained tab operations.
use super::*;

#[test]
fn px0_open_is_rejected_and_retained_tab_target_round_trips() {
    assert!(serde_json::from_str::<TabOp>(r#"{"op":"px0"}"#).is_err());
    assert!(
        serde_json::from_str::<Request>(
            r#"{"method":"tab","caller":{"workspace_id":"w4"},"op":{"op":"px0"}}"#
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
        op: TabOp::Focus { id: 42 },
    });
    let wire = serde_json::to_string(&request).unwrap();
    assert_eq!(serde_json::from_str::<Request>(&wire).unwrap(), request);
}
