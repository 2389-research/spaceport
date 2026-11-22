// ABOUTME: Pins Claude turn cancellation and completion correlation.
// ABOUTME: Keeps provider completion semantics explicit during state changes.
use super::*;

#[test]
fn a_natural_result_after_cancel_still_reports_cancelled() {
    let mut bridge = opened();
    prompt(&mut bridge, 12, "go");
    bridge.on_session(Inbound::Notification {
        method: "session/cancel".to_owned(),
        params: json!({}),
    });
    let outs = bridge.on_harness(&result("success", Some("end_turn")));
    assert_eq!(
        response(outs.last().unwrap(), 12).as_ref().unwrap()["stopReason"],
        "cancelled"
    );
    let duplicate = bridge.on_harness(&result("success", Some("end_turn")));
    assert!(
        duplicate
            .iter()
            .all(|out| !matches!(out, Out::Session(Inbound::Response { .. })))
    );
    assert!(matches!(
        prompt(&mut bridge, 13, "again").as_slice(),
        [Out::Harness(_)]
    ));
}
