//! A private Core client refetches a remote document by permanent Core ID.
//! Reconnect and missed events must not leave its old version authoritative.

use super::*;
use crate::CoreBackend;
use sheprd_backend::Backend as _;
use std::os::unix::fs::PermissionsExt as _;

#[test]
#[ignore = "needs HERDR_CORE_TEST_BINARY"]
fn remote_document_reloads_by_core_id_after_hub_and_machine_reconnect() {
    let binary = std::env::var_os("HERDR_CORE_TEST_BINARY").expect("HERDR_CORE_TEST_BINARY");
    let mut remote = Sandbox::start(binary.clone());
    let relay = remote._dir.path().join("fixture-ssh");
    std::fs::write(
        &relay,
        b"#!/bin/sh\n[ \"$1\" = fixture-host ] || exit 2\nexport HC_SOCKET=\"$TEST_REMOTE_SOCKET\"\nexec \"$TEST_REMOTE_BINARY\" connect\n",
    )
    .unwrap();
    std::fs::set_permissions(&relay, std::fs::Permissions::from_mode(0o700)).unwrap();
    let mut local = Sandbox::start_with(
        binary.clone(),
        vec![
            ("HC_SSH", relay.into_os_string()),
            ("HC_RECONNECT_MS", "100".into()),
            ("TEST_REMOTE_BINARY", binary),
            ("TEST_REMOTE_SOCKET", remote.var("HC_SOCKET").unwrap()),
        ],
    );
    let hub = local.connect();
    let added = hub
        .machine_add("fixture-host", Some("lab"), None, false)
        .unwrap();
    let machine_id = added.id;
    let connected = |core: &Core| {
        let deadline = Instant::now() + Duration::from_secs(10);
        loop {
            let catalog = core.machine_list().unwrap();
            if catalog
                .machines
                .iter()
                .any(|view| view.machine.id == machine_id && view.state == "connected")
            {
                return;
            }
            assert!(Instant::now() < deadline, "private machine did not connect");
            std::thread::sleep(Duration::from_millis(30));
        }
    };
    connected(&hub);
    let backend = CoreBackend::new(local.connect(), 1000);
    let (_, mut changes) = backend.watch().unwrap();
    let remote_core = remote.connect();
    let document = |step| serde_json::json!({"schema_version": 1, "workspaces": [], "step": step});
    assert_eq!(
        remote_core
            .doc_set("spaceport", &document(1), 0, None)
            .unwrap(),
        1
    );
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        let batch = changes.next_batch(Duration::from_millis(100)).unwrap();
        if batch.documents.iter().any(|notice| {
            notice.machine.as_deref() == Some("lab")
                && notice.name == "spaceport"
                && notice.version == 1
        }) {
            break;
        }
        assert!(
            Instant::now() < deadline,
            "remote document notice did not arrive"
        );
    }
    assert_eq!(
        backend
            .core()
            .doc_get("spaceport", Some(&machine_id))
            .unwrap()
            .version,
        1
    );

    local.stop().unwrap();
    assert_eq!(
        remote_core
            .doc_set("spaceport", &document(2), 1, None)
            .unwrap(),
        2
    );
    local.restart();
    let resumed = local.connect();
    connected(&resumed);
    let renamed = resumed.machine_rename("lab", "work").unwrap();
    assert_eq!(renamed.id, machine_id);
    let backend = CoreBackend::new(local.connect(), 1000);
    let (_, mut changes) = backend.watch().unwrap();
    let current = backend
        .core()
        .doc_get("spaceport", Some(&machine_id))
        .unwrap();
    assert_eq!(current.version, 2);
    assert_eq!(current.value, document(2));

    remote.stop().unwrap();
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        let catalog = resumed.machine_list().unwrap();
        if catalog
            .machines
            .iter()
            .any(|view| view.machine.id == machine_id && view.state != "connected")
        {
            break;
        }
        assert!(
            Instant::now() < deadline,
            "hub did not observe the machine loss"
        );
        std::thread::sleep(Duration::from_millis(30));
    }
    remote.restart();
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        let batch = changes.next_batch(Duration::from_millis(100)).unwrap();
        if batch.missed {
            break;
        }
        assert!(
            Instant::now() < deadline,
            "remote reconnect did not mark documents missed"
        );
    }
    connected(&resumed);
    assert_eq!(
        backend
            .core()
            .doc_get("spaceport", Some(&machine_id))
            .unwrap()
            .version,
        2
    );
    assert_eq!(
        remote
            .connect()
            .doc_set("spaceport", &document(3), 2, None)
            .unwrap(),
        3
    );
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        let batch = changes.next_batch(Duration::from_millis(100)).unwrap();
        if batch.documents.iter().any(|notice| {
            notice.machine.as_deref() == Some("work")
                && notice.name == "spaceport"
                && notice.version == 3
        }) {
            break;
        }
        assert!(
            Instant::now() < deadline,
            "renamed machine document notice did not arrive"
        );
    }
    let current = backend
        .core()
        .doc_get("spaceport", Some(&machine_id))
        .unwrap();
    assert_eq!(current.version, 3);
    assert_eq!(current.value, document(3));
    assert_eq!(
        backend.core().doc_get("spaceport", None).unwrap().version,
        0
    );
    local.stop().unwrap();
    remote.stop().unwrap();
}
