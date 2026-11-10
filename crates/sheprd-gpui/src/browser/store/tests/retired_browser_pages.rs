use super::*;
use crate::browser::LocalFile;

#[test]
fn saved_browser_pages_do_not_restore_but_retained_views_keep_their_ids() {
    let mut store = Store::default();
    let scope = Scope::endpoint("fixture-host");
    let web = store
        .open(
            scope.clone(),
            "fixture-workspace",
            url("https://example.test/"),
            None,
        )
        .unwrap();
    let blank = store
        .open(scope.clone(), "fixture-workspace", None, None)
        .unwrap();
    let path = std::env::temp_dir()
        .join("spaceport-page")
        .join("preview.html");
    assert!(path.is_absolute());
    assert!(path.parent().unwrap().file_name().is_some());
    let local = store
        .open(
            scope.clone(),
            "fixture-workspace",
            Some(Location::Local {
                file: LocalFile::new(&path, None).unwrap(),
            }),
            None,
        )
        .unwrap();
    let devices = store
        .open(scope, "fixture-workspace", Some(Location::Devices), None)
        .unwrap();
    let core = CoreGroupKey::new("fixture-core", None, "spaceport", Uuid::from_u128(42)).unwrap();
    let terminal = store.open_core(&core, None).unwrap();
    let saved = serde_json::to_vec(&store.saved()).unwrap();

    let restored = Store::with_saved(parse_saved(&saved).unwrap(), None);

    assert!(restored.get(web).is_none());
    assert!(restored.get(blank).is_none());
    assert!(restored.get(local).is_none());
    assert_eq!(
        restored.get(devices).unwrap().location,
        Some(Location::Devices)
    );
    assert_eq!(restored.get(terminal).unwrap().location, None);
    assert!(restored.next > u64::from(terminal));
}
