//! Two private Core processes exercise Core-owned machine aggregation.
//! A test-only pipe command carries bytes without real SSH or user state.
use super::*;
use crate::{CoreBackend, event::Notice, wire::Run};
use sheprd_backend::{Backend as _, Spawn};
use std::os::unix::fs::PermissionsExt as _;

#[test]
#[ignore = "needs HERDR_CORE_TEST_BINARY"]
fn a_real_core_aggregates_a_fixture_machine() {
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
    let core = local.connect();
    let added = core
        .machine_add("fixture-host", Some("lab"), None, false)
        .unwrap();
    assert_eq!(added.name, "lab");
    assert_ne!(added.id, core.hello().core.id);
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        let catalog = core.machine_list().unwrap();
        if catalog
            .machines
            .iter()
            .any(|view| view.machine.id == added.id && view.state == "connected")
        {
            break;
        }
        assert!(Instant::now() < deadline, "fixture machine did not connect");
        std::thread::sleep(Duration::from_millis(30));
    }
    let line = core.machine_connect("lab").unwrap();
    assert_eq!(line.machine.id, added.id);
    assert_eq!(line.state, "connected");
    let mut events = core.subscribe().unwrap();
    assert_eq!(core.doc_get("spaceport", Some("lab")).unwrap().version, 0);
    assert_eq!(
        core.doc_set(
            "spaceport",
            &serde_json::json!({"scope": "remote"}),
            0,
            Some("lab")
        )
        .unwrap(),
        1
    );
    let deadline = Instant::now() + Duration::from_secs(5);
    let document = loop {
        let notices = events.next(Duration::from_millis(100)).unwrap();
        if let Some(document) = notices
            .into_iter()
            .find(|notice| matches!(notice, Notice::Document { name, .. } if name == "spaceport"))
        {
            break document;
        }
        assert!(
            Instant::now() < deadline,
            "remote document event did not arrive"
        );
    };
    assert!(matches!(
        document,
        Notice::Document { machine: Some(machine), version: 1, .. } if machine == "lab"
    ));
    assert_eq!(
        remote.connect().doc_get("spaceport", None).unwrap().value,
        serde_json::json!({"scope": "remote"})
    );
    assert_eq!(core.doc_get("spaceport", None).unwrap().version, 0);
    assert_eq!(
        remote
            .connect()
            .doc_set(
                "spaceport",
                &serde_json::json!({"scope": "edited"}),
                1,
                None
            )
            .unwrap(),
        2
    );
    assert!(matches!(
        core.doc_set("spaceport", &serde_json::json!({"scope": "stale"}), 1, Some("lab")),
        Err(Error::Refused { kind: Some(kind), .. }) if kind == "conflict"
    ));
    assert_eq!(core.doc_get("spaceport", Some("lab")).unwrap().version, 2);
    let argv = [
        "sh".into(),
        "-c".into(),
        "printf remote-core; sleep 20".into(),
    ];
    let started = core
        .run(&Run {
            argv: &argv,
            on: Some("lab"),
            ..Run::default()
        })
        .unwrap();
    assert!(started.address.starts_with("lab/"), "{}", started.address);
    assert_eq!(
        core.terminal_get(&started.address).unwrap().address,
        started.address
    );
    assert_eq!(
        core.list(false)
            .unwrap()
            .terminals
            .iter()
            .filter(|terminal| terminal.address == started.address)
            .count(),
        1
    );
    core.kill(&started.address).unwrap();
    let backend = CoreBackend::new(local.connect(), 1000);
    let spawned = backend
        .spawn(&Spawn {
            argv: argv.to_vec(),
            machine: Some("lab".into()),
            ..Spawn::default()
        })
        .unwrap();
    assert!(spawned.id.as_str().starts_with("lab/"));
    backend.kill(&spawned.id).unwrap();
    remote.child.kill().unwrap();
    remote.child.wait().unwrap();
    remote.restart();
    let restarted = remote.connect();
    assert_eq!(restarted.hello().core.id, added.id);
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        let catalog = core.machine_list().unwrap();
        if catalog
            .machines
            .iter()
            .any(|view| view.machine.id == added.id && view.state == "connected")
        {
            break;
        }
        assert!(Instant::now() < deadline, "fixture machine did not recover");
        std::thread::sleep(Duration::from_millis(30));
    }
    let resumed = core
        .run(&Run {
            argv: &argv,
            on: Some("lab"),
            ..Run::default()
        })
        .unwrap();
    assert!(resumed.address.starts_with("lab/"), "{}", resumed.address);
    let bytes = [0, 255, 65];
    let staged = core
        .terminal_put(&resumed.address, "binary.png", &bytes)
        .unwrap();
    assert_eq!(staged.bytes, bytes.len());
    assert_eq!(std::fs::read(&staged.path).unwrap(), bytes);
    let local_address = resumed.address.strip_prefix("lab/").unwrap();
    let renamed = core.machine_rename("lab", "work").unwrap();
    assert_eq!(renamed.id, added.id);
    assert_eq!(renamed.name, "work");
    assert_eq!(
        core.terminal_get(&format!("work/{local_address}"))
            .unwrap()
            .address,
        format!("work/{local_address}")
    );
    assert_eq!(core.machine_rm("work").unwrap(), "work");
    assert!(core.machine_list().unwrap().machines.is_empty());
    let remote_core = remote.connect();
    assert!(
        remote_core
            .list(false)
            .unwrap()
            .terminals
            .iter()
            .any(|terminal| terminal.address == local_address)
    );
    remote_core.kill(local_address).unwrap();
    local.stop().unwrap();
    remote.stop().unwrap();
}

#[test]
#[ignore = "needs HERDR_CORE_TEST_BINARY"]
fn a_real_core_does_not_save_a_refused_machine() {
    let binary = std::env::var_os("HERDR_CORE_TEST_BINARY").expect("HERDR_CORE_TEST_BINARY");
    let parent = std::env::var_os("HERDR_TEST_TMPDIR").unwrap_or_else(|| "/tmp".into());
    let dir = tempfile::Builder::new()
        .prefix("hc-refuse")
        .tempdir_in(std::path::PathBuf::from(parent))
        .unwrap();
    let relay = dir.path().join("fixture-ssh");
    std::fs::write(
        &relay,
        b"#!/bin/sh\nprintf 'WARNING: REMOTE HOST IDENTIFICATION HAS CHANGED!\\n' >&2\nexit 255\n",
    )
    .unwrap();
    std::fs::set_permissions(&relay, std::fs::Permissions::from_mode(0o700)).unwrap();
    let mut local = Sandbox::start_with(binary, vec![("HC_SSH", relay.into_os_string())]);
    let core = local.connect();
    assert!(
        local.child.try_wait().unwrap().is_none(),
        "private Core detached"
    );
    match core.machine_add("fixture-host", Some("lab"), None, false) {
        Err(Error::Refused {
            kind,
            hint: Some(hint),
            ..
        }) => {
            assert_eq!(kind.as_deref(), Some("unreachable"));
            assert!(hint.as_str().contains("ssh fixture-host"));
        }
        other => panic!("expected Core refusal, got {other:?}"),
    }
    assert!(core.machine_list().unwrap().machines.is_empty());
    local.stop().unwrap();
}
