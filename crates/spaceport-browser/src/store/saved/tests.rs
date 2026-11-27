#![allow(clippy::unwrap_used)]
use super::*;

#[test]
fn every_retired_kind_reads_as_retired_and_an_unknown_kind_is_refused() {
    for kind in ["web", "local", "review", "orchestrator", "code"] {
        let location: SavedLocation =
            serde_json::from_str(&format!(r#"{{"kind":"{kind}","payload":7}}"#)).unwrap();
        assert!(matches!(location, SavedLocation::Retired {}), "{kind}");
    }
    assert!(serde_json::from_str::<SavedLocation>(r#"{"kind":"hologram"}"#).is_err());
}
