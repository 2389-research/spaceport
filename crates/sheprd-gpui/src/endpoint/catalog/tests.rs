#![allow(clippy::unwrap_used)]
use super::*;
use crate::endpoint::tests::host;
use std::process::Command;

#[test]
fn explicit_socket_never_reads_shared_catalog() {
    let mut catalog = Catalog::new(&ConnectTarget::Socket("/unused.sock".into()));
    assert!(catalog.poll().is_none());
    assert!(catalog.pending.is_none());
    catalog.choose(LOCAL);
    assert!(catalog.queued_write.is_none());
    assert!(catalog.poll_write().is_none());
    assert!(catalog.writing.is_none());
    assert!(
        Catalog::new(&ConnectTarget::Session {
            name: "test".into(),
            development: true
        })
        .development
            == Some(true)
    );
}

#[test]
fn selection_writes_are_serialized_and_failure_keeps_the_ui_choice() {
    let mut catalog = Catalog::new(&ConnectTarget::Local);
    let (tx, rx) = mpsc::sync_channel(1);
    catalog.writing = Some(rx);
    catalog.choose("ssh:first");
    catalog.choose(LOCAL);
    catalog.choose("ssh:last");
    assert!(catalog.poll_write().is_none());
    assert!(catalog.writing.is_some());
    assert_eq!(catalog.queued_write, Some(Some("ssh:last".into())));
    // Simulate a failed worker without accessing the real user's state root.
    catalog.queued_write = None;
    tx.send(Err(std::io::Error::other("disk unavailable").into()))
        .unwrap();
    assert!(
        matches!(catalog.poll_write(), Some(Error::Io(error)) if error.to_string() == "disk unavailable")
    );
    assert!(catalog.writing.is_none());
    assert_eq!(catalog.desired.as_deref(), Some("ssh:last"));
    assert!(!catalog.restore_pending);
}

#[test]
fn desired_selection_is_client_local_and_catalog_changes_cancel_stale_restore() {
    let update = |enabled, selection| CatalogUpdate {
        hosts: vec![host("a", enabled)],
        wsl: Vec::new(),
        selection,
    };
    let mut first = Catalog::new(&ConnectTarget::Local);
    let mut second = Catalog::new(&ConnectTarget::Local);
    first.accept(&update(true, Some(Some("ssh:a".into()))));
    second.accept(&update(true, Some(Some("ssh:a".into()))));
    second.choose(LOCAL);
    first.accept(&update(true, Some(None)));
    assert_eq!(first.desired.as_deref(), Some("ssh:a"));
    assert!(first.restore_pending);
    assert_eq!(second.desired, None);
    first.accept(&update(false, None));
    assert_eq!(first.desired, None);
    assert!(!first.restore_pending);
    first.accept(&update(true, None));
    assert!(!first.restore_pending);
    let mut clicked = Catalog::new(&ConnectTarget::Local);
    clicked.choose(LOCAL);
    clicked.accept(&update(true, Some(Some("ssh:a".into()))));
    assert_eq!(
        clicked.desired, None,
        "late startup read cannot undo a click"
    );
    assert_eq!(clicked.queued_write, Some(None));
    second.choose("ssh:a");
    second.accept(&CatalogUpdate {
        hosts: vec![],
        wsl: Vec::new(),
        selection: None,
    });
    assert_eq!(second.desired, None);
}

#[gpui::test]
fn old_container_catalog_stays_foreign_to_startup_devices(cx: &mut gpui::TestAppContext) {
    let state = tempfile::tempdir().unwrap();
    let path = state.path().join("cloud-devices.json");
    let bytes =
        br#"{"version":1,"devices":[{"provider":"container","account":"local","id":"old","machine":"saved-machine","session":"saved-session","label":"Old Container","unknown":{"keep":true}}]}"#;
    std::fs::write(&path, bytes).unwrap();
    let (view, cx) = cx.add_window_view(crate::sidebar::layout_tests::fixture_window);
    view.update(cx, |view, cx| {
        let ids = |view: &SheprdWindow| {
            view.endpoints
                .iter()
                .map(|e| e.id.clone())
                .collect::<Vec<_>>()
        };
        view.reconcile_devices(vec![host("a", false)], Vec::new(), cx);
        assert_eq!(ids(view), [LOCAL, "ssh:a"]);
        assert_eq!(
            view.endpoints[1].connection.target,
            ConnectTarget::Ssh {
                target: "user@a".into(),
                session: "default".into()
            }
        );
        view.reconcile_devices(vec![], Vec::new(), cx);
        assert_eq!(ids(view), [LOCAL]);
    });
    assert_eq!(std::fs::read(path).unwrap(), bytes);
}

#[test]
fn startup_catalog_leaves_private_old_container_file_byte_for_byte() {
    if let Some(path) = std::env::var_os("SPACEPORT_OLD_CONTAINER_FIXTURE") {
        let update = load(false, true).unwrap();
        assert!(update.hosts.is_empty());
        assert!(update.wsl.is_empty());
        assert_eq!(update.selection, Some(None));
        assert_eq!(devices(update.hosts, update.wsl).len(), 0);
        assert!(std::fs::metadata(path).unwrap().is_file());
        return;
    }
    let home = tempfile::tempdir().unwrap();
    let state = home.path().join("state").join("spaceport");
    std::fs::create_dir_all(&state).unwrap();
    let path = state.join("cloud-devices.json");
    let bytes =
        br#"{"version":1,"devices":[{"provider":"container","account":"local","id":"old","machine":"saved-machine","session":"saved-session","label":"Old Container","unknown":{"keep":true}}]}"#;
    std::fs::write(&path, bytes).unwrap();
    let test = format!(
        "{}::startup_catalog_leaves_private_old_container_file_byte_for_byte",
        module_path!()
    );
    let output = Command::new(std::env::current_exe().unwrap())
        .args(["--exact", &test, "--nocapture"])
        .env("HOME", home.path())
        .env("XDG_CONFIG_HOME", home.path().join("config"))
        .env("XDG_STATE_HOME", home.path().join("state"))
        .env("SPACEPORT_OLD_CONTAINER_FIXTURE", &path)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(std::fs::read(path).unwrap(), bytes);
}

mod storage_warning;
mod wsl;
