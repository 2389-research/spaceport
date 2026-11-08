//! A private two-Core fixture proves exact-ID text reads after a machine rename.
//! It keeps file bytes and both Core state roots inside temporary directories.
use super::*;
use crate::wire::FileRead;
use std::os::unix::fs::PermissionsExt as _;

#[test]
#[ignore = "needs HERDR_CORE_TEST_BINARY"]
fn a_real_core_reads_only_text_from_the_renamed_machine_id() {
    let binary = std::env::var_os("HERDR_CORE_TEST_BINARY").expect("HERDR_CORE_TEST_BINARY");
    let mut remote = Sandbox::start(binary.clone());
    let text = remote._dir.path().join("fixture.txt");
    let binary_file = remote._dir.path().join("fixture.bin");
    let long_line = remote._dir.path().join("long-line.txt");
    std::fs::write(&text, b"first\nsecond\nthird\n").unwrap();
    std::fs::write(&binary_file, [0, 255, 65]).unwrap();
    std::fs::write(&long_line, vec![b'x'; (8 << 20) + 1]).unwrap();

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
    let machine = core
        .machine_add("fixture-host", Some("lab"), None, false)
        .unwrap();
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        if core
            .machine_list()
            .unwrap()
            .machines
            .iter()
            .any(|view| view.machine.id == machine.id && view.state == "connected")
        {
            break;
        }
        assert!(Instant::now() < deadline, "fixture machine did not connect");
        std::thread::sleep(Duration::from_millis(30));
    }
    assert_eq!(core.machine_rename("lab", "work").unwrap().id, machine.id);

    let text_path = text.to_str().unwrap();
    let read = core
        .file_read(&FileRead {
            path: text_path,
            on: Some(&machine.id),
            cwd: None,
            offset: Some(2),
            limit: Some(1),
        })
        .unwrap();
    assert_eq!(
        (read.text.as_str(), read.first, read.last, read.lines),
        ("second\n", 2, 2, 3)
    );
    assert!(matches!(
        core.file_read(&FileRead {
            path: binary_file.to_str().unwrap(),
            on: Some(&machine.id),
            cwd: None,
            offset: None,
            limit: None,
        }),
        Err(Error::Refused { kind: Some(kind), .. }) if kind == "not_text"
    ));
    assert!(matches!(
        core.file_read(&FileRead {
            path: long_line.to_str().unwrap(),
            on: Some(&machine.id),
            cwd: None,
            offset: None,
            limit: Some(1),
        }),
        Err(Error::Refused { kind: Some(kind), .. }) if kind == "invalid_params"
    ));
    local.stop().unwrap();
    remote.stop().unwrap();
}
