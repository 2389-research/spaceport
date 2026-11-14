// ABOUTME: Verifies Ask Spaceport uses the normal activity-panel wire contract.
// ABOUTME: Keeps the existing Overview independently addressable.
use super::*;

#[test]
fn ask_and_overview_are_separate_activity_panels() {
    assert_eq!(ActivityPanel::parse("ask"), Some(ActivityPanel::Ask));
    assert_eq!(
        ActivityPanel::parse("overview"),
        Some(ActivityPanel::Overview)
    );
    let request = Request::Ui(Call {
        caller: Caller::default(),
        op: UiOp::Panel {
            panel: ActivityPanel::Ask,
        },
    });
    let encoded = serde_json::to_value(&request).unwrap();
    assert_eq!(encoded["op"]["panel"], "ask");
    assert_eq!(serde_json::from_value::<Request>(encoded).unwrap(), request);
}

#[test]
fn status_roundtrip_retains_read_only_ask_facts_and_existing_fields() {
    let reply: crate::protocol::Response = serde_json::from_value(serde_json::json!({
        "status":"status","version":"0.1","protocol":crate::protocol::PROTOCOL_VERSION,
        "pid":1,"windows":[],"core_catalog":{"epoch":null,"state":"not_started","machine_count":null,"error":null},
        "ask":{"configured":true,"loading":false,"answering":true,"history_present":true,"agent":"Pi","error":"Observed failure"}
    })).unwrap();
    let encoded = serde_json::to_value(&reply).unwrap();
    assert_eq!(encoded["ask"]["answering"], true);
    assert_eq!(encoded["ask"]["agent"], "Pi");
    assert_eq!(encoded["ask"]["error"], "Observed failure");
    assert_eq!(encoded["pid"], 1);
    assert_eq!(encoded["core_catalog"]["state"], "not_started");
    assert_eq!(
        serde_json::from_value::<crate::protocol::Response>(encoded).unwrap(),
        reply
    );
}
