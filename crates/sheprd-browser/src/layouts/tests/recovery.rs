// ABOUTME: Exercises layout restoration using isolated files and real writers.
// ABOUTME: Checks rejected file copies and the saved tab ID floor across restarts.
use super::*;
use serde_json::{Value, json};
use std::{fs, path::Path};

const LAYOUTS_FILE: &str = "editor-groups.json";
const TORN: &[u8] = br#"{"layouts":["#;

fn entry(workspace: &str, id: u64, share: u64) -> Value {
    json!({
        "scope": "local",
        "workspace": workspace,
        "layout": {"groups": [{"pick": {"kind": "page", "id": id}, "share": share}], "active": 0},
    })
}

fn finish_writes(layouts: &mut Layouts) -> bool {
    layouts.writer.take().is_some_and(|writer| {
        writer.finish();
        true
    })
}

fn backups(folder: &Path) -> Vec<PathBuf> {
    let prefix = format!("{LAYOUTS_FILE}.rejected-");
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

#[cfg(unix)]
struct ReadOnlyDirectory<'a> {
    path: &'a Path,
    original: fs::Permissions,
}

#[cfg(unix)]
impl<'a> ReadOnlyDirectory<'a> {
    fn new(path: &'a Path) -> Self {
        use std::os::unix::fs::PermissionsExt;

        let restore = Self {
            path,
            original: fs::metadata(path).unwrap().permissions(),
        };
        fs::set_permissions(path, fs::Permissions::from_mode(0o500)).unwrap();
        restore
    }
}

#[cfg(unix)]
impl Drop for ReadOnlyDirectory<'_> {
    fn drop(&mut self) {
        let _ = fs::set_permissions(self.path, self.original.clone());
    }
}

#[test]
fn recovery_keeps_valid_layouts_and_preserves_rejected_layout_bytes() {
    let good = entry("w", 41, 1);
    let expected: SavedLayout = serde_json::from_value(good["layout"].clone()).unwrap();
    let mut unknown_pick = entry("future", 70, 1);
    unknown_pick["layout"]["groups"][0]["pick"]["kind"] = json!("future");
    for bad in [unknown_pick, entry("invalid", 70, 0)] {
        let folder = tempfile::tempdir().unwrap();
        let path = folder.path().join(LAYOUTS_FILE);
        let original = serde_json::to_vec_pretty(&json!({"layouts": [good, bad]})).unwrap();
        fs::write(&path, &original).unwrap();

        let key = (Scope::endpoint("local"), "w".to_owned());
        let mut layouts = Layouts::load(Some(path.clone()));
        layouts.save();
        assert!(finish_writes(&mut layouts));
        assert_eq!(layouts.layouts.get(&key), Some(&expected));

        let copies = backups(folder.path());
        assert_eq!(copies.len(), 1);
        assert_eq!(fs::read(&copies[0]).unwrap(), original);
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            assert_eq!(
                fs::metadata(&copies[0]).unwrap().permissions().mode() & 0o777,
                0o600
            );
        }
        let saved = parse(&fs::read(path).unwrap()).unwrap();
        assert_eq!(saved.len(), 1);
        assert_eq!(saved.get(&key), Some(&expected));
    }
}

#[test]
fn recovery_keeps_the_id_floor_after_saving_and_removing_the_last_layout() {
    let folder = tempfile::tempdir().unwrap();
    let path = folder.path().join(LAYOUTS_FILE);
    let original = json!({"layouts": [entry("w", 41, 1), entry("invalid", 300, 0)]});
    fs::write(&path, serde_json::to_vec(&original).unwrap()).unwrap();
    let key = (Scope::endpoint("local"), "w".to_owned());

    let mut layouts = Layouts::load(Some(path.clone()));
    layouts.save();
    assert!(finish_writes(&mut layouts));
    let saved: Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    assert_eq!(saved["next_tab_id"], json!(301));

    let mut reloaded = Layouts::load(Some(path.clone()));
    let removed = reloaded.layouts.remove(&key);
    reloaded.save();
    assert!(finish_writes(&mut reloaded));
    assert!(removed.is_some());
    let saved: Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    assert_eq!(saved["layouts"], json!([]));
    assert_eq!(saved["next_tab_id"], json!(301));

    let mut again = Layouts::load(Some(path.clone()));
    again.save();
    assert!(finish_writes(&mut again));
    assert!(again.layouts.is_empty());
    let saved: Value = serde_json::from_slice(&fs::read(path).unwrap()).unwrap();
    assert_eq!(saved["layouts"], json!([]));
    assert_eq!(saved["next_tab_id"], json!(301));
}

#[gpui::test]
fn recovery_records_new_layout_ids_before_forgetting_the_last_workspace(
    cx: &mut gpui::TestAppContext,
) {
    for (slot, pick, tab, hidden) in [
        ("pick", 300, 41, 42),
        ("tabs", 41, 300, 42),
        ("hidden", 41, 42, 300),
    ] {
        let folder = tempfile::tempdir().unwrap();
        let path = folder.path().join(LAYOUTS_FILE);
        let key = (Scope::endpoint("local"), "w".to_owned());
        let layout = serde_json::from_value(json!({
            "groups": [{
                "pick": {"kind": "page", "id": pick},
                "tabs": [{"kind": "page", "id": tab}],
                "hidden": [{"kind": "page", "id": hidden}],
                "share": 1,
            }],
            "active": 0,
        }))
        .unwrap();

        let layouts = Layouts::load(Some(path.clone()));
        let writer = cx.update(|cx| {
            cx.set_global(layouts);
            Layouts::record(cx, &key, Some(layout));
            cx.global_mut::<Layouts>().writer.take()
        });
        writer.unwrap().finish();
        let recorded: Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
        assert_eq!(recorded["next_tab_id"], json!(301), "{slot}");

        let reloaded = Layouts::load(Some(path.clone()));
        let writer = cx.update(|cx| {
            cx.set_global(reloaded);
            Layouts::forget_workspaces(cx, &key.0, std::slice::from_ref(&key.1));
            cx.global_mut::<Layouts>().writer.take()
        });
        writer.unwrap().finish();
        let saved: Value = serde_json::from_slice(&fs::read(path).unwrap()).unwrap();
        assert_eq!(saved["layouts"], json!([]), "{slot}");
        assert_eq!(saved["next_tab_id"], json!(301), "{slot}");
    }
}

#[test]
fn recovery_keeps_an_unsafe_layout_envelope_blocked_on_reload() {
    let folder = tempfile::tempdir().unwrap();
    let path = folder.path().join(LAYOUTS_FILE);
    fs::write(&path, TORN).unwrap();
    for _ in 0..2 {
        let mut layouts = Layouts::load(Some(path.clone()));
        layouts.save();
        assert!(!finish_writes(&mut layouts));
        assert_eq!(fs::read(&path).unwrap(), TORN);
    }
    let copies = backups(folder.path());
    assert!(!copies.is_empty());
    for copy in copies {
        assert_eq!(fs::read(copy).unwrap(), TORN);
    }
}

#[test]
fn recovery_refuses_to_write_after_a_layout_read_error() {
    let folder = tempfile::tempdir().unwrap();
    let path = folder.path().join(LAYOUTS_FILE);
    fs::create_dir(&path).unwrap();
    fs::write(path.join("keep"), b"original").unwrap();

    let mut layouts = Layouts::load(Some(path.clone()));
    layouts.save();
    assert!(!finish_writes(&mut layouts));
    assert_eq!(fs::read(path.join("keep")).unwrap(), b"original".as_slice());
    assert!(backups(folder.path()).is_empty());
}

#[test]
fn recovery_refuses_to_write_after_an_oversized_layout_file() {
    let folder = tempfile::tempdir().unwrap();
    let path = folder.path().join(LAYOUTS_FILE);
    let original = vec![b' '; MAX_FILE_BYTES as usize + 1];
    fs::write(&path, &original).unwrap();

    let mut layouts = Layouts::load(Some(path.clone()));
    layouts.save();
    let had_writer = finish_writes(&mut layouts);
    assert!(
        fs::read(path).unwrap() == original,
        "the oversized layout file was replaced"
    );
    assert!(!had_writer);
}

#[cfg(unix)]
#[test]
fn recovery_refuses_to_write_when_the_layout_backup_fails() {
    let folder = tempfile::tempdir().unwrap();
    let path = folder.path().join(LAYOUTS_FILE);
    let good = entry("w", 41, 1);
    let original =
        serde_json::to_vec(&json!({"layouts": [good, entry("invalid", 300, 0)]})).unwrap();
    fs::write(&path, &original).unwrap();
    let mut layouts = {
        let _restore = ReadOnlyDirectory::new(folder.path());
        assert_eq!(fs::read(&path).unwrap(), original);
        assert_eq!(
            fs::File::create(folder.path().join("write-probe"))
                .unwrap_err()
                .kind(),
            std::io::ErrorKind::PermissionDenied
        );
        Layouts::load(Some(path.clone()))
    };

    layouts.save();
    let had_writer = finish_writes(&mut layouts);
    assert_eq!(fs::read(path).unwrap(), original);
    assert!(!had_writer);
    assert!(backups(folder.path()).is_empty());
}

#[cfg(unix)]
#[test]
fn recovery_keeps_legacy_hidden_ids_after_a_failed_layout_write() {
    let folder = tempfile::tempdir().unwrap();
    let path = folder.path().join(LAYOUTS_FILE);
    let mut legacy = entry("w", 41, 1);
    legacy["layout"]["groups"][0]["hidden"] = json!([{"kind": "page", "id": 300}]);
    let original = serde_json::to_vec_pretty(&json!({"layouts": [legacy]})).unwrap();
    fs::write(&path, &original).unwrap();

    let mut layouts = Layouts::load(Some(path.clone()));
    let (had_writer, write_error) = {
        let _restore = ReadOnlyDirectory::new(folder.path());
        let write_error = fs::File::create(folder.path().join("write-probe")).err();
        layouts.save();
        (finish_writes(&mut layouts), write_error)
    };
    assert!(had_writer);
    assert_eq!(
        write_error.map(|error| error.kind()),
        Some(std::io::ErrorKind::PermissionDenied)
    );
    assert_eq!(fs::read(&path).unwrap(), original);

    let mut reloaded = Layouts::load(Some(path.clone()));
    reloaded.save();
    assert!(finish_writes(&mut reloaded));
    let saved: Value = serde_json::from_slice(&fs::read(path).unwrap()).unwrap();
    assert_eq!(saved["next_tab_id"], json!(301));
}

#[test]
fn recovery_creates_a_layout_file_when_the_saved_file_is_missing() {
    let folder = tempfile::tempdir().unwrap();
    let path = folder.path().join(LAYOUTS_FILE);
    let key = (Scope::endpoint("local"), "w".to_owned());
    let layout: SavedLayout = serde_json::from_value(entry("w", 41, 1)["layout"].clone()).unwrap();

    let mut layouts = Layouts::load(Some(path.clone()));
    layouts.layouts.insert(key.clone(), layout.clone());
    layouts.save();
    assert!(finish_writes(&mut layouts));
    let mut reloaded = Layouts::load(Some(path));
    assert!(finish_writes(&mut reloaded));
    assert_eq!(reloaded.layouts.get(&key), Some(&layout));
    assert!(backups(folder.path()).is_empty());
}

#[test]
fn positional_layout_references_survive_key_replacement_and_entry_rejection() {
    let page = json!({"kind": "page", "id": 300});
    let unknown = json!({"kind": "future", "id": 7});
    for group in [json!([page, 1, [], []]), json!([unknown, 1, [], [page]])] {
        let folder = tempfile::tempdir().unwrap();
        let path = folder.path().join(LAYOUTS_FILE);
        let high = json!(["local", "w", {"groups": [group], "active": 0}]);
        let low = entry("w", 2, 1);
        let original = serde_json::to_vec(&json!({"layouts": [high, low]})).unwrap();
        fs::write(&path, &original).unwrap();

        assert_eq!(reserved_next(&path).unwrap(), 301);
        let mut layouts = Layouts::load(Some(path.clone()));
        layouts.save();
        assert!(finish_writes(&mut layouts));
        let bytes = fs::read(&path).unwrap();
        let saved: Value = serde_json::from_slice(&bytes).unwrap();
        let key = (Scope::endpoint("local"), "w".to_owned());
        let expected: SavedLayout = serde_json::from_value(low["layout"].clone()).unwrap();
        let restored = parse(&bytes).unwrap();
        assert_eq!(restored.len(), 1);
        assert_eq!(restored.get(&key), Some(&expected));
        assert_eq!(saved["next_tab_id"], json!(301));
        assert_eq!(reserved_next(&path).unwrap(), 301);
    }
}
