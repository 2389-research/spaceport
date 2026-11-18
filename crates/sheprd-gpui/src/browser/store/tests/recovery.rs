// ABOUTME: Exercises recovery against real saved tab and layout files.
// ABOUTME: Protects rejected bytes and tab identities across subsequent saves.
use super::*;

const GOOD: &str =
    r#"{"id":2,"scope":"local","workspace_id":"w","location":{"kind":"devices"},"title":"Good"}"#;

fn copies(path: &std::path::Path) -> Vec<PathBuf> {
    let prefix = format!("{}.rejected-", path.file_name().unwrap().to_string_lossy());
    std::fs::read_dir(path.parent().unwrap())
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| {
            path.file_name()
                .unwrap()
                .to_string_lossy()
                .starts_with(&prefix)
        })
        .collect()
}

#[test]
fn recovery_preserves_unknown_and_invalid_tabs_before_saving_valid_tabs() {
    for bad in [
        r#"{"id":70,"scope":"local","workspace_id":"w","location":{"kind":"hologram"},"title":"Future"}"#,
        r#"{"id":70,"scope":"local","workspace_id":"","location":{"kind":"devices"},"title":"Invalid"}"#,
    ] {
        let folder = tempfile::tempdir().unwrap();
        let path = folder.path().join("browser-tabs.json");
        let original = format!(r#"{{"tabs":[{GOOD},{bad}],"next":3}}"#);
        std::fs::write(&path, &original).unwrap();
        let mut store = Store::load_from(Some(path.clone()));
        assert!(store.get(TabId::test(2)).is_some());
        let opened = store
            .open(Scope::endpoint("local"), "w", Some(Location::Devices), None)
            .unwrap();
        assert_eq!(u64::from(opened), 71);
        store.writer.take().unwrap().finish();
        let backups = copies(&path);
        assert_eq!(backups.len(), 1);
        assert_eq!(std::fs::read(&backups[0]).unwrap(), original.as_bytes());
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            assert_eq!(
                std::fs::metadata(&backups[0]).unwrap().permissions().mode() & 0o777,
                0o600
            );
        }
        let again = Store::restored_for_test(&std::fs::read(&path).unwrap()).unwrap();
        assert!(again.get(TabId::test(2)).is_some());
        assert!(again.get(opened).is_some());
    }
}

#[test]
fn recovery_reserves_layout_picks_owned_tabs_and_hidden_tabs_after_torn_tab_file() {
    let folder = tempfile::tempdir().unwrap();
    let path = folder.path().join("browser-tabs.json");
    let torn = format!(r#"{{"tabs":[{GOOD},{{"id":3,"sco"#);
    std::fs::write(&path, &torn).unwrap();
    std::fs::write(folder.path().join("editor-groups.json"), r#"{"layouts":[{"scope":"local","workspace":"w","layout":{"groups":[{"pick":{"kind":"page","id":41},"share":1,"tabs":[{"kind":"page","id":60}],"hidden":[{"kind":"page","id":77}]}],"active":0}}]}"#).unwrap();
    let mut store = Store::load_from(Some(path.clone()));
    let opened = store
        .open(Scope::endpoint("local"), "w", Some(Location::Devices), None)
        .unwrap();
    assert_eq!(u64::from(opened), 78);
    store.writer.take().unwrap().finish();
    assert_eq!(std::fs::read(&copies(&path)[0]).unwrap(), torn.as_bytes());
    assert!(
        Store::restored_for_test(&std::fs::read(path).unwrap())
            .unwrap()
            .get(opened)
            .is_some()
    );
}

#[test]
fn recovery_blocks_allocation_and_writes_when_layout_references_cannot_be_read() {
    let folder = tempfile::tempdir().unwrap();
    let path = folder.path().join("browser-tabs.json");
    let original = format!(r#"{{"tabs":[{GOOD}],"next":3}}"#);
    std::fs::write(&path, &original).unwrap();
    std::fs::write(folder.path().join("editor-groups.json"), "{\"layouts\":[").unwrap();
    let mut store = Store::load_from(Some(path.clone()));
    assert!(store.get(TabId::test(2)).is_some());
    assert!(store.restoration_blocked());
    assert!(
        store
            .open(Scope::endpoint("local"), "w", Some(Location::Devices), None)
            .is_none()
    );
    assert!(store.writer.is_none());
    assert!(store.visited(TabId::test(2), None, Some("Changed")));
    assert_eq!(std::fs::read(path).unwrap(), original.as_bytes());
}

#[test]
fn recovery_reserves_references_in_invalid_layouts_when_tabs_are_missing() {
    let folder = tempfile::tempdir().unwrap();
    let path = folder.path().join("browser-tabs.json");
    std::fs::write(folder.path().join("editor-groups.json"), r#"{"layouts":[{"scope":"local","workspace":"","layout":{"groups":[{"pick":{"kind":"page","id":300},"share":0}],"active":9}}]}"#).unwrap();
    let mut store = Store::load_from(Some(path.clone()));
    assert!(!store.restoration_blocked());
    let opened = store
        .open(Scope::endpoint("local"), "w", Some(Location::Devices), None)
        .unwrap();
    assert_eq!(u64::from(opened), 301);
    store.writer.take().unwrap().finish();
    let mut again = Store::load_from(Some(path));
    assert!(again.next > 301);
    again.writer.take().unwrap().finish();
}

#[test]
fn recovery_never_wraps_a_maximum_layout_reference() {
    let folder = tempfile::tempdir().unwrap();
    let path = folder.path().join("browser-tabs.json");
    std::fs::write(
        folder.path().join("editor-groups.json"),
        format!(
            r#"{{"layouts":[{{"layout":{{"groups":[{{"pick":{{"kind":"page","id":{}}}}}]}}}}]}}"#,
            u64::MAX
        ),
    )
    .unwrap();
    let mut store = Store::load_from(Some(path));
    assert_eq!(store.next, u64::MAX);
    assert!(
        store
            .open(Scope::endpoint("local"), "w", Some(Location::Devices), None)
            .is_none()
    );
    store.writer.take().unwrap().finish();
}

#[test]
fn recovery_does_not_replace_an_oversized_source() {
    let folder = tempfile::tempdir().unwrap();
    let path = folder.path().join("browser-tabs.json");
    let bytes = vec![b' '; MAX_FILE_BYTES as usize + 1];
    std::fs::write(&path, &bytes).unwrap();
    let mut store = Store::load_from(Some(path.clone()));
    assert!(store.restoration_blocked());
    assert!(store.writer.is_none());
    assert!(
        store
            .open(Scope::endpoint("local"), "w", Some(Location::Devices), None)
            .is_none()
    );
    assert_eq!(std::fs::read(path).unwrap(), bytes);
}

#[test]
fn recovery_does_not_replace_a_source_that_cannot_be_read() {
    let folder = tempfile::tempdir().unwrap();
    let path = folder.path().join("browser-tabs.json");
    std::fs::create_dir(&path).unwrap();
    let mut store = Store::load_from(Some(path.clone()));
    assert!(store.restoration_blocked());
    assert!(store.writer.is_none());
    assert!(
        store
            .open(Scope::endpoint("local"), "w", Some(Location::Devices), None)
            .is_none()
    );
    assert!(path.is_dir());
    assert!(copies(&path).is_empty());
}

#[cfg(unix)]
#[test]
fn recovery_does_not_write_when_the_rejected_source_cannot_be_preserved() {
    use std::os::unix::fs::PermissionsExt;
    let folder = tempfile::tempdir().unwrap();
    let path = folder.path().join("browser-tabs.json");
    std::fs::write(&path, "{").unwrap();
    std::fs::set_permissions(folder.path(), std::fs::Permissions::from_mode(0o500)).unwrap();
    let mut store = Store::load_from(Some(path.clone()));
    std::fs::set_permissions(folder.path(), std::fs::Permissions::from_mode(0o700)).unwrap();
    assert!(store.restoration_blocked());
    assert!(store.writer.is_none());
    assert!(
        store
            .open(Scope::endpoint("local"), "w", Some(Location::Devices), None)
            .is_none()
    );
    assert_eq!(std::fs::read_to_string(path).unwrap(), "{");
    assert!(copies(&folder.path().join("browser-tabs.json")).is_empty());
}

#[test]
fn recovery_never_overwrites_rejected_copies_from_other_loads() {
    let folder = tempfile::tempdir().unwrap();
    let path = folder.path().join("browser-tabs.json");
    for original in ["{", "{\"tabs\":["] {
        std::fs::write(&path, original).unwrap();
        let mut store = Store::load_from(Some(path.clone()));
        store.writer.take().unwrap().finish();
    }
    let originals = copies(&path)
        .iter()
        .map(|path| std::fs::read_to_string(path).unwrap())
        .collect::<Vec<_>>();
    assert_eq!(originals.len(), 2);
    assert!(originals.contains(&"{".to_owned()));
    assert!(originals.contains(&"{\"tabs\":[".to_owned()));
}
