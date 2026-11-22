// ABOUTME: Rejects retired custom updater messages on the private control wire.
// ABOUTME: Both request and reply decoding refuse the removed variants.
use super::*;

#[test]
fn removed_updater_operations_and_replies_are_rejected() {
    for op in ["check_update", "update"] {
        assert!(serde_json::from_value::<SettingsOp>(serde_json::json!({"op": op})).is_err());
    }
    assert!(
        serde_json::from_value::<Reply>(
            serde_json::json!({"kind":"update", "state":"idle", "latest":null})
        )
        .is_err()
    );
}
