use super::*;
use uuid::Uuid;

#[test]
fn core_tabs_keep_permanent_machine_and_group_identity_through_save() {
    let group = Uuid::from_u128(71);
    let first = CoreGroupKey::new("hub-a", Some("machine-a"), "spaceport", group).unwrap();
    let second = CoreGroupKey::new("hub-a", Some("machine-b"), "spaceport", group).unwrap();
    let other_document = CoreGroupKey::new("hub-a", Some("machine-a"), "other", group).unwrap();
    let mut store = Store::default();
    let a = store.open_core(&first, None).unwrap();
    let b = store.open_core(&second, None).unwrap();
    let c = store.open_core(&other_document, None).unwrap();
    assert_ne!(a, b);
    assert_ne!(a, c);
    assert_eq!(
        store
            .in_core_group(&first)
            .map(|tab| tab.id)
            .collect::<Vec<_>>(),
        [a]
    );
    assert_eq!(
        store
            .in_core_group(&second)
            .map(|tab| tab.id)
            .collect::<Vec<_>>(),
        [b]
    );
    assert_eq!(
        store
            .in_core_group(&other_document)
            .map(|tab| tab.id)
            .collect::<Vec<_>>(),
        [c]
    );

    let bytes = serde_json::to_vec(&store.saved()).unwrap();
    let restored = Store::with_saved(parse_saved(&bytes).unwrap(), None);
    assert_eq!(
        restored
            .in_core_group(&first)
            .map(|tab| tab.id)
            .collect::<Vec<_>>(),
        [a]
    );
    assert_eq!(
        restored
            .in_core_group(&second)
            .map(|tab| tab.id)
            .collect::<Vec<_>>(),
        [b]
    );
    assert_eq!(
        restored
            .in_core_group(&other_document)
            .map(|tab| tab.id)
            .collect::<Vec<_>>(),
        [c]
    );
    assert!(restored.next > u64::from(c));
}

#[test]
fn old_flat_tabs_remain_legacy_even_when_their_ids_match_a_core_group() {
    let group = Uuid::from_u128(71);
    let legacy = format!(
        r#"{{"tabs":[{{"id":4,"scope":"local:/tmp/herdr.sock","workspace_id":"{group}","location":null,"title":"Old","origin":null}}],"next":5}}"#
    );
    let restored = Store::with_saved(parse_saved(legacy.as_bytes()).unwrap(), None);
    let legacy_scope = Scope::local("/tmp/herdr.sock".as_ref());
    let core_key = CoreGroupKey::new("hub-a", None, "spaceport", group).unwrap();
    assert_eq!(
        restored
            .in_workspace(&legacy_scope, &group.to_string())
            .count(),
        1
    );
    assert_eq!(restored.in_core_group(&core_key).count(), 0);
    assert!(restored.get(TabId::test(4)).unwrap().core_group().is_none());
}

#[test]
fn core_saved_group_ids_must_use_one_canonical_shape() {
    let group = Uuid::from_u128(0xabc);
    let key = CoreGroupKey::new("hub-a", None, "spaceport", group).unwrap();
    let (scope, canonical) = key.parts();
    assert!(scope.valid_workspace(&canonical));
    assert!(!scope.valid_workspace(&canonical.to_uppercase()));
    let saved = format!(
        r#"{{"tabs":[{{"id":1,"scope":{},"workspace_id":"{}","location":null,"title":"Core"}}]}}"#,
        serde_json::to_string(&scope).unwrap(),
        canonical.to_uppercase()
    );
    assert!(parse_saved(saved.as_bytes()).is_err());
}
