//! A private hub targets a remote Core session by permanent machine ID.
//! Run and attach must reach that exact Core after its display name changes.

use super::*;
use std::os::unix::fs::PermissionsExt as _;

#[test]
#[ignore = "needs HERDR_CORE_TEST_BINARY"]
fn run_and_attach_route_to_selected_remote_session_by_id() {
    let binary = std::env::var_os("HERDR_CORE_TEST_BINARY").expect("HERDR_CORE_TEST_BINARY");
    let mut remote = Sandbox::start_with(binary.clone(), vec![("HC_SESSION", "target".into())]);
    let relay = remote._dir.path().join("fixture-ssh");
    std::fs::write(
        &relay,
        b"#!/bin/sh\n[ \"$1\" = fixture-host ] || exit 2\ncase \" $* \" in *HC_SESSION=target*) ;; *) exit 3;; esac\nexport HC_SESSION=target HC_SOCKET=\"$TEST_REMOTE_SOCKET\"\nexec \"$TEST_REMOTE_BINARY\" connect\n",
    )
    .unwrap();
    std::fs::set_permissions(&relay, std::fs::Permissions::from_mode(0o700)).unwrap();
    let mut hub = Sandbox::start_with(
        binary.clone(),
        vec![
            ("HC_SSH", relay.into_os_string()),
            ("HC_RECONNECT_MS", "100".into()),
            ("TEST_REMOTE_BINARY", binary),
            ("TEST_REMOTE_SOCKET", remote.var("HC_SOCKET").unwrap()),
        ],
    );
    let core = hub.connect();
    let machine = core
        .machine_add("fixture-host", Some("lab"), Some("target"), false)
        .unwrap();
    assert_eq!(machine.id, remote.connect().hello().core.id);
    let deadline = Instant::now() + Duration::from_secs(10);
    while !core
        .machine_list()
        .unwrap()
        .machines
        .iter()
        .any(|view| view.machine.id == machine.id && view.state == "connected")
    {
        assert!(Instant::now() < deadline, "target session did not connect");
        std::thread::sleep(Duration::from_millis(30));
    }

    let argv = [
        "sh".into(),
        "-c".into(),
        "printf exact-session; sleep 30".into(),
    ];
    let started = core
        .run(&Run {
            argv: &argv,
            on: Some(&machine.id),
            ..Run::default()
        })
        .unwrap();
    let pid = started
        .pid
        .expect("private Core reports the started process");
    let local_id = started.address.strip_prefix("lab/").unwrap();
    assert!(!local_id.contains('/'));
    assert_eq!(
        remote.connect().terminal_get(local_id).unwrap().pid,
        Some(pid)
    );
    let deadline = Instant::now() + Duration::from_secs(10);
    while !core
        .list(false)
        .unwrap()
        .terminals
        .iter()
        .any(|terminal| terminal.address == started.address && terminal.pid == Some(pid))
    {
        assert!(
            Instant::now() < deadline,
            "remote terminal was not mirrored"
        );
        std::thread::sleep(Duration::from_millis(30));
    }
    let renamed = core.machine_rename("lab", "work").unwrap();
    assert_eq!(renamed.id, machine.id);
    let target = format!("{}/{local_id}", machine.id);
    let attached = core
        .attach(&Attach {
            cols: 80,
            rows: 24,
            terminal: &target,
            ..Attach::default()
        })
        .unwrap();
    assert_eq!(attached.terminal.pid, Some(pid));
    assert_eq!(attached.terminal.address, format!("work/{local_id}"));
    let mut seen = Vec::new();
    let deadline = Instant::now() + Duration::from_secs(10);
    while !seen.windows(13).any(|bytes| bytes == b"exact-session") {
        assert!(
            Instant::now() < deadline,
            "attached to wrong session: {seen:?}"
        );
        seen.extend(attached.read(Duration::from_millis(100)).unwrap().bytes);
    }
    assert_eq!(
        core.terminal_get(&target).unwrap().pid,
        remote.connect().terminal_get(local_id).unwrap().pid
    );
    assert_eq!(
        core.list(false)
            .unwrap()
            .terminals
            .iter()
            .filter(|terminal| terminal.pid == Some(pid))
            .count(),
        1
    );
    hub.stop().unwrap();
    remote.stop().unwrap();
}
