// ABOUTME: Keeps excluded host-resource polling out of the private control wire.
// ABOUTME: Rejects old commands and replies while retaining exact host selectors.
use super::*;

#[test]
fn resource_dashboard_operations_and_replies_are_rejected() {
    assert!(
        serde_json::from_str::<Request>(r#"{"method":"host","caller":{},"op":{"op":"system"}}"#)
            .is_err()
    );
    assert!(
        serde_json::from_str::<Response>(
            r#"{"status":"host","reply":{"kind":"system","load":null}}"#
        )
        .is_err()
    );
}

#[test]
fn retained_host_selection_keeps_its_exact_target() {
    let request = Request::Host(Call {
        caller: Caller::default(),
        op: HostOp::Select {
            id: "ssh:fixture@box:2202".into(),
        },
    });
    let encoded = serde_json::to_string(&request).unwrap();
    assert_eq!(serde_json::from_str::<Request>(&encoded).unwrap(), request);
}
