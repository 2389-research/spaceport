// ABOUTME: Verifies rejected layouts survive writes to real state files.
// ABOUTME: Checks valid entries remain available when another entry fails validation.
#![allow(clippy::unwrap_used)]
use super::*;

#[test]
fn recovery_keeps_rejected_ids_after_a_layout_only_save_and_restart() {
    let folder = tempfile::tempdir().unwrap();
    let path = folder.path().join("editor-groups.json");
    let tabs = folder.path().join("browser-tabs.json");
    std::fs::write(&tabs, r#"{"tabs":[],"next":3}"#).unwrap();
    std::fs::write(&path, r#"{"layouts":[{"scope":"local","workspace":"bad","layout":{"groups":[{"pick":{"kind":"page","id":300},"share":0}],"active":9}}]}"#).unwrap();
    drop(crate::browser::Store::load_from(Some(tabs.clone())));
    let mut layouts = Layouts::load_from(Some(path.clone()));
    layouts.save();
    layouts.writer.take().unwrap().finish();
    assert_eq!(reserved_next(&path).unwrap(), 301);
    let mut store = crate::browser::Store::load_from(Some(tabs.clone()));
    let opened = store
        .open(
            Scope::endpoint("local"),
            "w",
            Some(crate::browser::Location::Devices),
            None,
        )
        .unwrap();
    assert_eq!(u64::from(opened), 301);
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
    loop {
        let file: serde_json::Value =
            serde_json::from_slice(&std::fs::read(&tabs).unwrap()).unwrap();
        if file["next"] == 302 {
            break;
        }
        assert!(std::time::Instant::now() < deadline);
        std::thread::yield_now();
    }
}

#[test]
fn recovery_preserves_a_saturated_floor_without_live_layouts() {
    let folder = tempfile::tempdir().unwrap();
    let path = folder.path().join("editor-groups.json");
    std::fs::write(
        &path,
        format!(r#"{{"layouts":[],"next_tab_id":{}}}"#, u64::MAX),
    )
    .unwrap();
    let mut layouts = Layouts::load_from(Some(path.clone()));
    layouts.save();
    layouts.writer.take().unwrap().finish();
    assert_eq!(reserved_next(&path).unwrap(), u64::MAX);
}

#[cfg(unix)]
#[test]
fn recovery_preserves_original_references_when_the_floor_write_fails() {
    use std::os::unix::fs::PermissionsExt;
    let folder = tempfile::tempdir().unwrap();
    let path = folder.path().join("editor-groups.json");
    let original = r#"{"layouts":[{"scope":"local","workspace":"w","layout":{"groups":[{"pick":{"kind":"page","id":41},"hidden":[{"kind":"page","id":300}],"share":1}],"active":0}}]}"#;
    std::fs::write(&path, original).unwrap();
    let mut layouts = Layouts::load_from(Some(path.clone()));
    std::fs::set_permissions(folder.path(), std::fs::Permissions::from_mode(0o500)).unwrap();
    layouts.save();
    layouts.writer.take().unwrap().finish();
    std::fs::set_permissions(folder.path(), std::fs::Permissions::from_mode(0o700)).unwrap();
    assert_eq!(std::fs::read_to_string(&path).unwrap(), original);
    assert_eq!(reserved_next(&path).unwrap(), 301);
}

#[test]
fn recovery_keeps_valid_layouts_and_preserves_every_rejected_byte() {
    let good = r#"{"scope":"local","workspace":"w","layout":{"groups":[{"pick":{"kind":"page","id":41},"share":1}],"active":0}}"#;
    for bad in [
        r#"{"scope":"local","workspace":"bad","layout":{"groups":[{"pick":{"kind":"future","id":70},"share":1}],"active":0}}"#,
        r#"{"scope":"local","workspace":"bad","layout":{"groups":[],"active":0}}"#,
    ] {
        let folder = tempfile::tempdir().unwrap();
        let path = folder.path().join("editor-groups.json");
        let original = format!(r#"{{"layouts":[{good},{bad}]}}"#);
        std::fs::write(&path, &original).unwrap();
        let mut layouts = Layouts::load_from(Some(path.clone()));
        let key = (Scope::endpoint("local"), "w".to_owned());
        assert!(layouts.layouts.contains_key(&key));
        layouts.save();
        layouts.writer.take().unwrap().finish();
        let backups = std::fs::read_dir(folder.path())
            .unwrap()
            .map(|entry| entry.unwrap().path())
            .filter(|path| {
                path.file_name()
                    .unwrap()
                    .to_string_lossy()
                    .starts_with("editor-groups.json.rejected-")
            })
            .collect::<Vec<_>>();
        assert_eq!(backups.len(), 1);
        assert_eq!(std::fs::read(&backups[0]).unwrap(), original.as_bytes());
        assert!(
            parse(&std::fs::read(path).unwrap())
                .unwrap()
                .contains_key(&key)
        );
    }
}

#[test]
fn recovery_keeps_unreadable_reference_files_blocked_across_restarts() {
    let folder = tempfile::tempdir().unwrap();
    let path = folder.path().join("editor-groups.json");
    std::fs::write(&path, "{\"layouts\":[").unwrap();
    let layouts = Layouts::load_from(Some(path.clone()));
    layouts.save();
    assert!(layouts.writer.is_none());
    assert_eq!(std::fs::read_to_string(&path).unwrap(), "{\"layouts\":[");
    let again = Layouts::load_from(Some(path));
    assert!(again.writer.is_none());
    assert!(std::fs::read_dir(folder.path()).unwrap().any(|entry| {
        let path = entry.unwrap().path();
        path.file_name()
            .unwrap()
            .to_string_lossy()
            .starts_with("editor-groups.json.rejected-")
            && std::fs::read_to_string(path).unwrap() == "{\"layouts\":["
    }));
}

#[cfg(unix)]
#[test]
fn recovery_does_not_replace_layouts_when_the_backup_fails() {
    use std::os::unix::fs::PermissionsExt;
    let folder = tempfile::tempdir().unwrap();
    let path = folder.path().join("editor-groups.json");
    std::fs::write(&path, "{").unwrap();
    std::fs::set_permissions(folder.path(), std::fs::Permissions::from_mode(0o500)).unwrap();
    let layouts = Layouts::load_from(Some(path.clone()));
    std::fs::set_permissions(folder.path(), std::fs::Permissions::from_mode(0o700)).unwrap();
    assert!(layouts.writer.is_none());
    layouts.save();
    assert_eq!(std::fs::read_to_string(path).unwrap(), "{");
}
