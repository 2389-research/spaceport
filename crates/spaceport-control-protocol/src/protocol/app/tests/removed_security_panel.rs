use super::*;

#[test]
fn security_panel_is_not_a_routable_activity_destination() {
    assert_eq!(ActivityPanel::parse("security"), None);
    assert!(serde_json::from_str::<ActivityPanel>("\"security\"").is_err());
    let retired =
        r#"{"method":"ui","caller":{"window":42},"op":{"op":"panel","panel":"security"}}"#;
    assert!(serde_json::from_str::<Request>(retired).is_err());
    assert_eq!(
        ActivityPanel::parse("workspaces"),
        Some(ActivityPanel::Workspaces)
    );
    assert_eq!(
        ActivityPanel::parse("overview"),
        Some(ActivityPanel::Overview)
    );
    let retained = Request::Ui(Call {
        caller: Caller {
            window: Some(42),
            ..Caller::default()
        },
        op: UiOp::Panel {
            panel: ActivityPanel::Overview,
        },
    });
    let wire = serde_json::to_string(&retained).unwrap();
    assert_eq!(serde_json::from_str::<Request>(&wire).unwrap(), retained);
}
