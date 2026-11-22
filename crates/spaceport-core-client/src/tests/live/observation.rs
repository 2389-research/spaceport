//! A private Core reports terminal facts without inferring agent activity.

use super::*;
use crate::{
    event::{Notice, TerminalChange},
    wire::StatusState,
};

#[test]
#[ignore = "needs HERDR_CORE_TEST_BINARY"]
fn reported_status_and_unreported_process_are_distinct() {
    let binary = std::env::var_os("HERDR_CORE_TEST_BINARY").expect("HERDR_CORE_TEST_BINARY");
    let mut sandbox = Sandbox::start(binary);
    let core = sandbox.connect();
    let mut events = core.subscribe().unwrap();

    let quiet = core
        .run(&Run {
            argv: &["sh".into(), "-c".into(), "sleep 30".into()],
            ..Run::default()
        })
        .unwrap();
    assert_eq!(core.terminal_get(&quiet.address).unwrap().status, None);

    let reported = core
        .run(&Run {
            argv: &[
                "sh".into(),
                "-c".into(),
                "printf '\\033]7501;state=blocked:kind=permission:app=fixture\\033\\\\'; sleep 30"
                    .into(),
            ],
            ..Run::default()
        })
        .unwrap();
    let deadline = Instant::now() + Duration::from_secs(10);
    let mut saw_status_event = false;
    while !saw_status_event {
        assert!(
            Instant::now() < deadline,
            "Core did not report the fixture status"
        );
        saw_status_event = events.next(Duration::from_millis(100)).unwrap().into_iter().any(|notice| {
            matches!(notice, Notice::Event(event)
                if event.terminal == reported.address
                    && matches!(event.change, TerminalChange::Status { reported: true, status: Some(ref status) }
                        if status.state == StatusState::Blocked && status.kind.as_deref() == Some("permission")))
        });
    }
    let listed = core.list(false).unwrap();
    assert!(
        listed
            .terminals
            .iter()
            .any(|terminal| terminal.address == quiet.address && terminal.status.is_none())
    );
    assert!(listed.terminals.iter().any(|terminal| {
        terminal.address == reported.address
            && terminal.status.as_ref().is_some_and(|status| {
                status.state == StatusState::Blocked && status.kind.as_deref() == Some("permission")
            })
    }));

    core.kill(&quiet.address).unwrap();
    core.kill(&reported.address).unwrap();
    sandbox.stop().unwrap();
}
