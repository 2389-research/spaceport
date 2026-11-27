// ABOUTME: Exercises tab restoration using isolated files and real writers.
// ABOUTME: Checks that later saves preserve rejected bytes and reserve saved IDs.
use super::*;
use serde_json::{Value, json};
use std::{fs, path::Path};

const GOOD: &str =
    r#"{"id":2,"scope":"local","workspace_id":"w","location":{"kind":"devices"},"title":"Good"}"#;
const TABS_FILE: &str = "browser-tabs.json";
const LAYOUTS_FILE: &str = "editor-groups.json";

fn open_devices(store: &mut Store) -> Option<TabId> {
    store.open(Scope::endpoint("local"), "w", Some(Location::Devices), None)
}

fn finish_writes(store: &mut Store) -> bool {
    store.writer.take().is_some_and(|writer| {
        writer.finish();
        true
    })
}

fn backups(folder: &Path) -> Vec<PathBuf> {
    let prefix = format!("{TABS_FILE}.rejected-");
    fs::read_dir(folder)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| {
            path.file_name()
                .unwrap()
                .to_string_lossy()
                .strip_prefix(prefix.as_str())
                .is_some_and(|suffix| !suffix.is_empty())
        })
        .collect()
}

fn write_layout(folder: &Path, workspace: &str, group: Value) {
    let layout = json!({
        "layouts": [{
            "scope": "local",
            "workspace": workspace,
            "layout": {"groups": [group], "active": 0},
        }],
    });
    fs::write(
        folder.join(LAYOUTS_FILE),
        serde_json::to_vec(&layout).unwrap(),
    )
    .unwrap();
}

#[test]
fn recovery_keeps_valid_tabs_and_reserves_rejected_ids_and_saved_next() {
    for (bad, saved_next, expected) in [
        (
            r#"{"id":70,"scope":"local","workspace_id":"w","location":{"kind":"future"},"title":"Future"}"#,
            3,
            71,
        ),
        (
            r#"{"id":70,"scope":"local","workspace_id":"","location":{"kind":"devices"},"title":"Invalid"}"#,
            90,
            90,
        ),
        (
            r#"{"id":70,"scope":"local","workspace_id":"w","location":{"kind":"code","file":{"path":"/project/main.rs","line":-1}},"title":"Last edit"}"#,
            3,
            71,
        ),
        (
            r#"{"id":70,"scope":"local","workspace_id":"w","place":"px0","location":{"kind":"devices"},"origin":"w:p1","title":"Old editor"}"#,
            3,
            71,
        ),
    ] {
        let folder = tempfile::tempdir().unwrap();
        let path = folder.path().join(TABS_FILE);
        let original = format!(" \n{{\"tabs\":[{GOOD},{bad}],\"next\":{saved_next}}}\n");
        fs::write(&path, &original).unwrap();

        let mut store = Store::load(Some(path.clone()));
        let good = store.get(TabId::test(2)).cloned();
        let rejected = store.get(TabId::test(70)).is_some();
        let opened = open_devices(&mut store);
        assert!(finish_writes(&mut store));
        assert_eq!(good.as_ref().map(|tab| tab.title.as_str()), Some("Good"));
        assert!(!rejected);
        assert_eq!(opened, Some(TabId::test(expected)));

        let copies = backups(folder.path());
        assert_eq!(copies.len(), 1);
        assert_eq!(fs::read(&copies[0]).unwrap(), original.as_bytes());
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            assert_eq!(
                fs::metadata(&copies[0]).unwrap().permissions().mode() & 0o777,
                0o600
            );
        }
        let mut reloaded = Store::load(Some(path));
        assert!(finish_writes(&mut reloaded));
        assert_eq!(reloaded.get(TabId::test(2)), good.as_ref());
        assert!(reloaded.get(opened.unwrap()).is_some());
    }
}

#[test]
fn loading_valid_retired_tabs_leaves_the_original_as_the_only_copy() {
    let folder = tempfile::tempdir().unwrap();
    let path = folder.path().join(TABS_FILE);
    let retired = r#"{"id":7,"scope":"local","workspace_id":"w","location":{"kind":"web","url":"https://example.org"},"title":"Docs"}"#;
    let original = format!(r#"{{"tabs":[{GOOD},{retired}]}}"#);
    fs::write(&path, &original).unwrap();
    for _ in 0..2 {
        let mut store = Store::load(Some(path.clone()));
        assert!(store.get(TabId::test(2)).is_some());
        assert!(store.get(TabId::test(7)).is_none());
        assert!(finish_writes(&mut store));
    }
    assert_eq!(fs::read(&path).unwrap(), original.as_bytes());
    assert!(backups(folder.path()).is_empty());
}

#[test]
fn recovery_backs_up_torn_tabs_before_writing_a_new_tab() {
    let folder = tempfile::tempdir().unwrap();
    let path = folder.path().join(TABS_FILE);
    let original = format!(r#"{{"tabs":[{GOOD},{{"id":3,"sco"#);
    fs::write(&path, &original).unwrap();
    write_layout(
        folder.path(),
        "w",
        json!({"pick": {"kind": "page", "id": 41}, "share": 1}),
    );

    let mut store = Store::load(Some(path.clone()));
    let opened = open_devices(&mut store);
    assert!(finish_writes(&mut store));
    assert_eq!(opened, Some(TabId::test(42)));
    let copies = backups(folder.path());
    assert_eq!(copies.len(), 1);
    assert_eq!(fs::read(&copies[0]).unwrap(), original.as_bytes());
    let mut reloaded = Store::load(Some(path));
    assert!(finish_writes(&mut reloaded));
    assert!(reloaded.get(opened.unwrap()).is_some());
}

#[test]
fn recovery_keeps_unique_copies_of_rejected_files() {
    let folder = tempfile::tempdir().unwrap();
    let path = folder.path().join(TABS_FILE);
    let originals = [b"{".as_slice(), b"{\"tabs\":[".as_slice()];
    for original in originals {
        fs::write(&path, original).unwrap();
        let mut store = Store::load(Some(path.clone()));
        assert!(finish_writes(&mut store));
    }
    let mut preserved = backups(folder.path())
        .into_iter()
        .map(|path| fs::read(path).unwrap())
        .collect::<Vec<_>>();
    preserved.sort_unstable();
    assert_eq!(preserved, originals.map(<[u8]>::to_vec));
}

#[test]
fn recovery_reserves_page_ids_in_every_layout_reference_slot() {
    let page = json!({"kind": "page", "id": 77});
    for group in [
        json!({"pick": page, "share": 1}),
        json!({"tabs": [page], "share": 1}),
        json!({"hidden": [page], "share": 1}),
    ] {
        let folder = tempfile::tempdir().unwrap();
        write_layout(folder.path(), "w", group.clone());
        let mut store = Store::load(Some(folder.path().join(TABS_FILE)));
        let opened = open_devices(&mut store);
        assert!(finish_writes(&mut store));
        assert_eq!(opened, Some(TabId::test(78)), "{group}");
    }
}

#[test]
fn recovery_reserves_page_ids_from_invalid_layout_entries() {
    let page = json!({"kind": "page", "id": 300});
    for group in [
        json!({"pick": page, "share": 0}),
        json!({"pick": page, "tabs": [{"kind": "future", "id": 400}], "share": 1}),
    ] {
        let folder = tempfile::tempdir().unwrap();
        write_layout(folder.path(), "w", group.clone());
        let mut store = Store::load(Some(folder.path().join(TABS_FILE)));
        let opened = open_devices(&mut store);
        assert!(finish_writes(&mut store));
        assert_eq!(opened, Some(TabId::test(301)), "{group}");
    }
}

#[test]
fn recovery_refuses_allocation_at_the_maximum_saved_page_id() {
    let folder = tempfile::tempdir().unwrap();
    let path = folder.path().join(TABS_FILE);
    write_layout(
        folder.path(),
        "w",
        json!({"pick": {"kind": "page", "id": u64::MAX}, "share": 1}),
    );
    let mut store = Store::load(Some(path.clone()));
    let opened = open_devices(&mut store);
    finish_writes(&mut store);
    assert!(opened.is_none());
    assert!(!path.try_exists().unwrap());
}

#[test]
fn recovery_keeps_readable_tabs_when_the_layout_envelope_is_unsafe() {
    for unsafe_layout in [r#"{"layouts":["#, r#"{"layouts":{}}"#] {
        let folder = tempfile::tempdir().unwrap();
        let path = folder.path().join(TABS_FILE);
        let layouts = folder.path().join(LAYOUTS_FILE);
        let original = format!(r#"{{"tabs":[{GOOD}],"next":3}}"#);
        fs::write(&path, &original).unwrap();
        fs::write(&layouts, unsafe_layout).unwrap();

        let mut store = Store::load(Some(path.clone()));
        let restored = store.get(TabId::test(2)).is_some();
        let changed = store.visited(TabId::test(2), None, Some("Changed"));
        let opened = open_devices(&mut store);
        let had_writer = finish_writes(&mut store);
        assert!(restored);
        assert!(changed);
        assert!(opened.is_none(), "{unsafe_layout}");
        assert!(!had_writer);
        assert_eq!(fs::read(path).unwrap(), original.as_bytes());
        assert_eq!(fs::read(layouts).unwrap(), unsafe_layout.as_bytes());
    }
}

#[test]
fn recovery_refuses_to_write_after_a_tab_read_error() {
    let folder = tempfile::tempdir().unwrap();
    let path = folder.path().join(TABS_FILE);
    fs::create_dir(&path).unwrap();
    fs::write(path.join("keep"), b"original").unwrap();

    let mut store = Store::load(Some(path.clone()));
    let opened = open_devices(&mut store);
    let had_writer = finish_writes(&mut store);
    assert!(opened.is_none());
    assert!(!had_writer);
    assert_eq!(fs::read(path.join("keep")).unwrap(), b"original".as_slice());
    assert!(backups(folder.path()).is_empty());
}

#[test]
fn recovery_refuses_to_write_after_an_oversized_tab_file() {
    let folder = tempfile::tempdir().unwrap();
    let path = folder.path().join(TABS_FILE);
    let original = vec![b' '; MAX_FILE_BYTES as usize + 1];
    fs::write(&path, &original).unwrap();

    let mut store = Store::load(Some(path.clone()));
    let opened = open_devices(&mut store);
    let had_writer = finish_writes(&mut store);
    assert!(
        fs::read(path).unwrap() == original,
        "the oversized tab file was replaced"
    );
    assert!(opened.is_none());
    assert!(!had_writer);
}

#[cfg(unix)]
#[test]
fn recovery_refuses_to_write_when_the_tab_backup_fails() {
    use std::os::unix::fs::PermissionsExt;

    struct RestorePermissions<'a>(&'a Path, fs::Permissions);

    impl Drop for RestorePermissions<'_> {
        fn drop(&mut self) {
            let _ = fs::set_permissions(self.0, self.1.clone());
        }
    }

    let folder = tempfile::tempdir().unwrap();
    let path = folder.path().join(TABS_FILE);
    let original = b"{";
    fs::write(&path, original).unwrap();
    let mut store = {
        let _restore = RestorePermissions(
            folder.path(),
            fs::metadata(folder.path()).unwrap().permissions(),
        );
        fs::set_permissions(folder.path(), fs::Permissions::from_mode(0o500)).unwrap();
        assert_eq!(fs::read(&path).unwrap(), original.as_slice());
        assert_eq!(
            fs::File::create(folder.path().join("write-probe"))
                .unwrap_err()
                .kind(),
            std::io::ErrorKind::PermissionDenied
        );
        Store::load(Some(path.clone()))
    };

    let opened = open_devices(&mut store);
    let had_writer = finish_writes(&mut store);
    assert_eq!(fs::read(path).unwrap(), original.as_slice());
    assert!(opened.is_none());
    assert!(!had_writer);
    assert!(backups(folder.path()).is_empty());
}

#[test]
fn recovery_creates_a_tab_file_when_both_saved_files_are_missing() {
    let folder = tempfile::tempdir().unwrap();
    let path = folder.path().join(TABS_FILE);
    let mut store = Store::load(Some(path.clone()));
    let opened = open_devices(&mut store);
    assert!(finish_writes(&mut store));
    assert_eq!(opened, Some(TabId::test(0)));
    let mut reloaded = Store::load(Some(path));
    assert!(finish_writes(&mut reloaded));
    assert!(reloaded.get(opened.unwrap()).is_some());
    assert!(backups(folder.path()).is_empty());
}

#[test]
fn recovery_keeps_the_saved_layout_floor_and_tab_sequence_across_two_reloads() {
    let folder = tempfile::tempdir().unwrap();
    let path = folder.path().join(TABS_FILE);
    fs::write(&path, r#"{"tabs":[],"next":3}"#).unwrap();
    fs::write(
        folder.path().join(LAYOUTS_FILE),
        r#"{"layouts":[],"next_tab_id":301}"#,
    )
    .unwrap();

    let mut store = Store::load(Some(path.clone()));
    let opened = open_devices(&mut store);
    assert!(finish_writes(&mut store));
    assert_eq!(opened, Some(TabId::test(301)));

    let mut reloaded = Store::load(Some(path.clone()));
    let removed = reloaded.detach(opened.unwrap());
    assert!(finish_writes(&mut reloaded));
    assert_eq!(removed.map(|tab| tab.id), opened);

    let mut again = Store::load(Some(path));
    let next = open_devices(&mut again);
    assert!(finish_writes(&mut again));
    assert_eq!(next, Some(TabId::test(302)));
    assert!(again.get(opened.unwrap()).is_none());
}

#[test]
fn recovery_saves_existing_tabs_when_layout_references_exhaust_new_ids() {
    let folder = tempfile::tempdir().unwrap();
    let path = folder.path().join(TABS_FILE);
    fs::write(&path, format!(r#"{{"tabs":[{GOOD}]}}"#)).unwrap();
    write_layout(
        folder.path(),
        "w",
        json!({"pick": {"kind": "page", "id": u64::MAX}, "share": 1}),
    );
    let mut store = Store::load(Some(path.clone()));
    let admission = store.check_new_tab_id();
    let opened = open_devices(&mut store);
    let visited = store.visited(TabId::test(2), None, Some("Updated"));
    let joined = finish_writes(&mut store);
    assert!(matches!(admission, Err(Error::TabIdsExhausted)));
    assert!(opened.is_none());
    assert!(visited && joined);

    let mut reloaded = Store::load(Some(path));
    let title = reloaded.get(TabId::test(2)).map(|tab| tab.title.clone());
    let opened = open_devices(&mut reloaded);
    let joined = finish_writes(&mut reloaded);
    assert_eq!(title.as_deref(), Some("Updated"));
    assert!(opened.is_none());
    assert!(joined);
}

#[test]
fn recovery_rejects_duplicate_tab_fields_without_losing_their_id() {
    let folder = tempfile::tempdir().unwrap();
    let path = folder.path().join(TABS_FILE);
    let duplicate = r#"{"id":70,"scope":"local","workspace_id":"w","location":{"kind":"devices"},"title":"First","title":"Second"}"#;
    let original = format!(r#"{{"tabs":[{GOOD},{duplicate}]}}"#);
    fs::write(&path, &original).unwrap();

    let mut store = Store::load(Some(path));
    let good = store.get(TabId::test(2)).is_some();
    let duplicate = store.get(TabId::test(70)).is_some();
    let opened = open_devices(&mut store);
    let joined = finish_writes(&mut store);
    assert!(good);
    assert!(!duplicate);
    assert_eq!(opened, Some(TabId::test(71)));
    assert!(joined);
    let copies = backups(folder.path());
    assert_eq!(copies.len(), 1);
    assert_eq!(fs::read(&copies[0]).unwrap(), original.as_bytes());
}

#[test]
fn positional_tab_rows_reserve_their_id_when_an_entry_is_rejected() {
    for (kind, restored) in [("devices", true), ("future", false)] {
        let folder = tempfile::tempdir().unwrap();
        let path = folder.path().join(TABS_FILE);
        let row = json!([300, "local", "w", {"kind": kind}, "Positional", null, "group"]);
        let original = format!(r#"{{"tabs":[{GOOD},{row},false],"next":3}}"#);
        fs::write(&path, &original).unwrap();

        let mut store = Store::load(Some(path));
        let good = store.get(TabId::test(2)).is_some();
        let positional = store.get(TabId::test(300)).is_some();
        let opened = open_devices(&mut store);
        let joined = finish_writes(&mut store);
        assert!(good);
        assert_eq!(positional, restored);
        assert_eq!(opened, Some(TabId::test(301)));
        assert!(joined);
    }
}
