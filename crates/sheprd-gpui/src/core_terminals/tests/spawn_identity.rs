use crate::browser::CoreGroupKey;
use crate::core_terminals::{Purpose, purpose_matches_core, workspace_document};

#[test]
fn queued_core_tab_spawn_keeps_its_original_hub_and_document() {
    let group = uuid::Uuid::from_u128(71);
    let key = CoreGroupKey::new("hub-a", None, workspace_document::NAME, group).unwrap();
    let (scope, workspace) = key.parts();
    let purpose = Purpose::Tab { scope, workspace };
    assert!(purpose_matches_core(&purpose, "hub-a"));
    assert!(!purpose_matches_core(&purpose, "hub-b"));
    let remote =
        CoreGroupKey::new("hub-a", Some("machine-a"), workspace_document::NAME, group).unwrap();
    let (scope, workspace) = remote.parts();
    assert!(!purpose_matches_core(
        &Purpose::Tab { scope, workspace },
        "hub-a"
    ));
    let other_doc = CoreGroupKey::new("hub-a", None, "other", group).unwrap();
    let (scope, workspace) = other_doc.parts();
    assert!(!purpose_matches_core(
        &Purpose::Tab { scope, workspace },
        "hub-a"
    ));
}
