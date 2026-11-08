use super::*;

#[test]
fn window_new_refuses_named_legacy_session_on_the_wire() {
    let plain = r#"{"method":"window","caller":{"window":42},"op":{"op":"new"}}"#;
    let request: Request = serde_json::from_str(plain).unwrap();
    assert!(matches!(
        request,
        Request::Window(Call {
            caller: Caller {
                window: Some(42),
                ..
            },
            op: WindowOp::New {},
        })
    ));
    let named =
        r#"{"method":"window","caller":{"window":42},"op":{"op":"new","session":"legacy"}}"#;
    assert!(serde_json::from_str::<Request>(named).is_err());
}
