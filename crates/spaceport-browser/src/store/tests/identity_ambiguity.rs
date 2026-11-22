// ABOUTME: Checks identity reservations against duplicate fields in original JSON.
// ABOUTME: Requires a safe floor or a blocked writer without hiding valid siblings.
use super::*;
use std::fs;

const GOOD: &str =
    r#"{"id":2,"scope":"local","workspace_id":"w","location":{"kind":"devices"},"title":"Good"}"#;

fn finish(store: &mut Store) -> bool {
    store.writer.take().is_some_and(|writer| {
        writer.finish();
        true
    })
}

fn assert_safe_floor(original: &str, readable_title: Option<&str>) {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("browser-tabs.json");
    fs::write(&path, original).unwrap();
    let mut store = Store::load(Some(path.clone()));
    let good = store.get(TabId::test(2)).map(|tab| tab.title.clone());
    let opened = store.open(Scope::endpoint("local"), "w", Some(Location::Devices), None);
    let visited = store.visited(TabId::test(2), None, Some("Updated"));
    let had_writer = finish(&mut store);

    assert!(
        opened.is_none_or(|id| u64::from(id) >= 301),
        "an earlier identity occurrence was reused: {opened:?}"
    );
    if let Some(expected) = readable_title {
        assert_eq!(good.as_deref(), Some(expected));
        assert!(visited);
    }
    if opened.is_none() {
        assert!(!had_writer);
        assert_eq!(fs::read(path).unwrap(), original.as_bytes());
    }
}

#[test]
fn duplicate_tab_ids_never_erase_the_earlier_reservation() {
    let bad = r#"{"id":300,"id":3,"scope":"local","workspace_id":"w","location":{"kind":"devices"},"title":"Ambiguous"}"#;
    assert_safe_floor(
        &format!(r#"{{"tabs":[{GOOD},{bad}],"next":3}}"#),
        Some("Good"),
    );
}

#[test]
fn duplicate_saved_counters_never_erase_the_earlier_reservation() {
    assert_safe_floor(
        &format!(r#"{{"tabs":[{GOOD}],"next":301,"next":3}}"#),
        Some("Good"),
    );
}

#[test]
fn duplicate_tab_lists_never_erase_the_earlier_reservation() {
    let high = GOOD.replace("\"id\":2", "\"id\":300");
    assert_safe_floor(
        &format!(r#"{{"tabs":[{high}],"tabs":[{GOOD}],"next":3}}"#),
        None,
    );
}

#[test]
fn nonobject_rejected_tabs_leave_valid_siblings_writable() {
    for bad in ["null", "false", "42", "[]", "\"unknown\""] {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("browser-tabs.json");
        let original = format!(r#"{{"tabs":[{GOOD},{bad}],"next":3}}"#);
        fs::write(&path, original).unwrap();
        let mut store = Store::load(Some(path));
        let good = store.get(TabId::test(2)).cloned();
        let opened = store.open(Scope::endpoint("local"), "w", Some(Location::Devices), None);
        let had_writer = finish(&mut store);

        assert_eq!(good.as_ref().map(|tab| tab.title.as_str()), Some("Good"));
        assert_eq!(opened, Some(TabId::test(3)), "{bad}");
        assert!(had_writer);
    }
}
