//! A private Core keeps a terminal alive after its viewer detaches.

use super::*;
use crate::wire::TerminalState;

#[test]
#[ignore = "needs HERDR_CORE_TEST_BINARY"]
fn a_terminal_survives_detach_and_another_client_reattaches() {
    let binary = std::env::var_os("HERDR_CORE_TEST_BINARY").expect("HERDR_CORE_TEST_BINARY");
    let mut sandbox = Sandbox::start(binary);
    let first = sandbox.connect();
    let argv = [
        "sh".to_owned(),
        "-c".to_owned(),
        "printf still-here; sleep 30".to_owned(),
    ];
    let started = first
        .run(&Run {
            argv: &argv,
            ..Run::default()
        })
        .unwrap();
    let pid = started.pid.expect("Core identifies the fixture process");
    let attached = first
        .attach(&Attach {
            cols: 80,
            rows: 24,
            terminal: &started.address,
            ..Attach::default()
        })
        .unwrap();
    assert_eq!(attached.terminal.pid, Some(pid));
    drop(attached);
    drop(first);

    let second = sandbox.connect();
    let listed = second.list(false).unwrap();
    let survivor = listed
        .terminals
        .iter()
        .find(|terminal| terminal.address == started.address)
        .expect("detaching the only viewer kept the terminal");
    assert_eq!(survivor.state, TerminalState::Live);
    assert_eq!(survivor.pid, Some(pid));

    let reattached = second
        .attach(&Attach {
            cols: 80,
            rows: 24,
            terminal: &started.address,
            ..Attach::default()
        })
        .unwrap();
    assert_eq!(reattached.terminal.pid, Some(pid));
    let mut bytes = Vec::new();
    let deadline = Instant::now() + Duration::from_secs(10);
    while !bytes
        .windows(b"still-here".len())
        .any(|part| part == b"still-here")
    {
        assert!(Instant::now() < deadline, "no first paint after reattach");
        let chunk = reattached.read(Duration::from_millis(100)).unwrap();
        assert_eq!(chunk.end, None);
        bytes.extend(chunk.bytes);
    }

    let before: Vec<_> = second
        .list(false)
        .unwrap()
        .terminals
        .into_iter()
        .map(|t| t.address)
        .collect();
    assert!(matches!(
        second.attach(&Attach {
            cols: 80,
            rows: 24,
            terminal: "t999999999",
            ..Attach::default()
        }),
        Err(Error::Refused { .. })
    ));
    let after: Vec<_> = second
        .list(false)
        .unwrap()
        .terminals
        .into_iter()
        .map(|t| t.address)
        .collect();
    assert_eq!(
        after, before,
        "attaching a missing ID must not start a program"
    );

    drop(reattached);
    second.kill(&started.address).unwrap();
    sandbox.stop().unwrap();
}
