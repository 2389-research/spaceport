use super::*;

#[test]
fn github_host_operations_are_rejected_without_losing_exact_host_targets() {
    for op in [
        r#"{"op":"pr"}"#,
        r#"{"op":"pr_comments"}"#,
        r#"{"op":"pr_open"}"#,
        r#"{"op":"pr_quota"}"#,
        r#"{"op":"security"}"#,
        r#"{"op":"security_open","repository":"owner/repo","item":"42"}"#,
    ] {
        assert!(serde_json::from_str::<HostOp>(op).is_err(), "{op}");
    }

    let caller = Caller {
        daemon_socket: Some("/private/core.sock".into()),
        window: Some(7),
        workspace_id: Some("w4".into()),
        ..Caller::default()
    };
    for op in [HostOp::List, HostOp::Forward { port: 3000 }] {
        let request = Request::Host(Call {
            caller: caller.clone(),
            op,
        });
        let wire = serde_json::to_string(&request).unwrap();
        assert_eq!(serde_json::from_str::<Request>(&wire).unwrap(), request);
    }
}
