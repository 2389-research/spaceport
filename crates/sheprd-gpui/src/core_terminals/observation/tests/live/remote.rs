//! The app observer reads one remote document through a private Core hub.
//! Its machine route stays the permanent ID through notices and reconnects.

use super::*;
use std::os::unix::fs::PermissionsExt as _;

#[test]
#[ignore = "needs HERDR_CORE_TEST_BINARY"]
fn selected_remote_document_refetches_after_notice_and_reconnect() {
    let binary = std::env::var_os("HERDR_CORE_TEST_BINARY").expect("explicit private Core binary");
    let mut remote = Sandbox::start(binary.clone());
    let relay = remote._dir.path().join("fixture-ssh");
    std::fs::write(
        &relay,
        b"#!/bin/sh\n[ \"$1\" = fixture-host ] || exit 2\nexport HC_SOCKET=\"$TEST_REMOTE_SOCKET\"\nexec \"$TEST_REMOTE_BINARY\" connect\n",
    )
    .unwrap();
    std::fs::set_permissions(&relay, std::fs::Permissions::from_mode(0o700)).unwrap();
    let mut hub = Sandbox::start_with(
        binary.clone(),
        vec![
            ("HC_SSH", relay.into_os_string()),
            ("HC_RECONNECT_MS", "100".into()),
            ("TEST_REMOTE_BINARY", binary),
            ("TEST_REMOTE_SOCKET", remote.socket.clone().into_os_string()),
        ],
    );
    let core = hub.connect();
    let machine_id = core
        .machine_add("fixture-host", Some("lab"), None, false)
        .unwrap()
        .id;
    let deadline = Instant::now() + Duration::from_secs(10);
    while !core
        .machine_list()
        .unwrap()
        .machines
        .iter()
        .any(|view| view.machine.id == machine_id && view.state == "connected")
    {
        assert!(Instant::now() < deadline, "private machine did not connect");
        std::thread::sleep(Duration::from_millis(30));
    }
    let backend = CoreBackend::new(hub.connect(), 1000);
    let mailbox = Arc::new(Mutex::new(None));
    let stop = Arc::new(AtomicBool::new(false));
    let running = Arc::new(AtomicBool::new(true));
    let selection = Arc::new(Mutex::new(RemoteSelection {
        generation: 1,
        machine_id: Some(machine_id.clone()),
    }));
    let worker = {
        let mailbox = Arc::clone(&mailbox);
        let stop = Arc::clone(&stop);
        let running = Arc::clone(&running);
        std::thread::spawn(move || run(backend, 1, stop, running, mailbox, selection))
    };
    let wait_for = |version: u64, stale: bool| {
        let deadline = Instant::now() + Duration::from_secs(10);
        loop {
            if let Some(latest) = take_latest(&mailbox)
                && latest.remote.matches(1, 1, &machine_id)
            {
                if stale && matches!(latest.remote.status(), Some(RemoteStatus::Stale)) {
                    return;
                }
                if !stale
                    && matches!(latest.remote.status(), Some(RemoteStatus::Ready(doc)) if doc.version == version)
                {
                    return;
                }
            }
            assert!(
                Instant::now() < deadline,
                "remote document did not reach version {version} stale={stale}"
            );
            std::thread::sleep(Duration::from_millis(30));
        }
    };
    wait_for(0, false);
    let value = serde_json::json!({"schema_version": 1, "workspaces": []});
    assert_eq!(remote.connect().doc_set(NAME, &value, 0, None).unwrap(), 1);
    wait_for(1, false);
    remote.stop().unwrap();
    wait_for(1, true);
    remote.restart();
    remote.connect();
    wait_for(1, false);
    assert_eq!(remote.connect().doc_set(NAME, &value, 1, None).unwrap(), 2);
    wait_for(2, false);
    assert_eq!(core.doc_get(NAME, None).unwrap().version, 0);
    stop.store(true, Ordering::Release);
    worker.join().unwrap();
    hub.stop().unwrap();
    remote.stop().unwrap();
}
