//! Explicit termination uses a Core address even when display labels match.

use super::*;
use crate::wire::TerminalState;

#[test]
#[ignore = "needs HERDR_CORE_TEST_BINARY"]
fn killing_one_address_keeps_same_labeled_terminal_and_other_views() {
    let binary = std::env::var_os("HERDR_CORE_TEST_BINARY").expect("HERDR_CORE_TEST_BINARY");
    let mut sandbox = Sandbox::start(binary);
    let first = sandbox.connect();
    let second = sandbox.connect();
    let argv = ["sh".to_owned(), "-c".to_owned(), "sleep 60".to_owned()];
    let run = Run {
        argv: &argv,
        labels: std::collections::BTreeMap::from([("fixture.display", "same-label")]),
        ..Run::default()
    };
    let victim = first.run(&run).unwrap();
    let survivor = first.run(&run).unwrap();
    assert_ne!(victim.address, survivor.address);
    assert_ne!(victim.pid, survivor.pid);
    assert_eq!(
        victim.labels.get("fixture.display"),
        survivor.labels.get("fixture.display")
    );

    let attach = |terminal| Attach {
        cols: 80,
        rows: 24,
        observe: true,
        terminal,
        ..Attach::default()
    };
    let victim_first_view = first.attach(&attach(&victim.address)).unwrap();
    let victim_second_view = second.attach(&attach(&victim.address)).unwrap();
    let survivor_view = second.attach(&attach(&survivor.address)).unwrap();
    assert_eq!(victim_first_view.terminal.pid, victim.pid);
    assert_eq!(victim_second_view.terminal.pid, victim.pid);
    assert_eq!(survivor_view.terminal.pid, survivor.pid);

    let ended = first.kill(&victim.address).unwrap();
    assert_eq!(ended.address, victim.address);
    assert_eq!(ended.state, TerminalState::Finished);
    let still_live = second.terminal_get(&survivor.address).unwrap();
    assert_eq!(still_live.address, survivor.address);
    assert_eq!(still_live.state, TerminalState::Live);
    assert_eq!(still_live.pid, survivor.pid);
    assert_eq!(
        survivor_view.read(Duration::from_millis(100)).unwrap().end,
        None
    );

    drop((victim_first_view, victim_second_view, survivor_view));
    second.kill(&survivor.address).unwrap();
    sandbox.stop().unwrap();
}
