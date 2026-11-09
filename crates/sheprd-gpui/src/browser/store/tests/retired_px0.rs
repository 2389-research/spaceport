// ABOUTME: Checks that old px0 tabs retire without replacing retained tab IDs.
// ABOUTME: Keeps Core and Devices targets stable when a saved editor ID is dropped.
use super::*;
use uuid::Uuid;

#[test]
fn saved_px0_tab_is_discarded_without_reusing_its_id() {
    let mut old = Store::default();
    let scope = Scope::endpoint("local");
    let devices = old
        .open(scope.clone(), "w", Some(Location::Devices), None)
        .unwrap();
    let key = CoreGroupKey::new("hub-a", None, "spaceport", Uuid::from_u128(71)).unwrap();
    let core = old.open_core(&key, None).unwrap();
    let mut saved = serde_json::to_value(old.saved()).unwrap();
    let mut retired = saved["tabs"][0].clone();
    retired["id"] = 500.into();
    retired["place"] = "px0".into();
    retired["location"] = serde_json::Value::Null;
    saved["tabs"].as_array_mut().unwrap().push(retired);
    saved["next"] = 0.into();

    let mut restored = Store::with_saved(
        parse_saved(&serde_json::to_vec(&saved).unwrap()).unwrap(),
        None,
    );
    assert_eq!(
        restored.tabs().map(|tab| tab.id).collect::<Vec<_>>(),
        [devices, core]
    );
    assert!(restored.get(TabId::test(500)).is_none());
    assert_eq!(
        restored
            .in_core_group(&key)
            .map(|tab| tab.id)
            .collect::<Vec<_>>(),
        [core]
    );
    assert!(restored.get(devices).is_some());
    let next = restored.open(scope, "w", None, None).unwrap();
    assert!(u64::from(next) > 500);
    assert_eq!(restored.close(devices).map(|tab| tab.id), Some(devices));
    assert!(restored.get(core).is_some());
}
