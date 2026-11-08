//! Two private Core processes exercise Core-owned machine aggregation.
//! A test-only pipe command carries bytes without real SSH or user state.
use super::*;
use crate::wire::Run;
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
    let local = Sandbox::start_with(
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
    remote.child.kill().unwrap();
    remote.child.wait().unwrap();
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
    core.kill(&resumed.address).unwrap();
}
