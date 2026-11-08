use super::*;
use sheprd_backend::RunState;
use sheprd_client::protocol::AgentStatus;

fn info(name: &str) -> TerminalInfo {
    TerminalInfo {
        agent: AgentStatus::Unknown,
        cwd: None,
        id: TerminalId::new(name).unwrap(),
        labels: Default::default(),
        name: None,
        state: RunState::Running,
        title: None,
    }
}

#[test]
fn a_started_terminal_is_known_at_once() {
    let mut infos = Infos::default();
    let started = infos.insert(info("t1"));
    assert_eq!(infos.get(&started.id), Some(&started));
}

#[test]
fn a_started_terminal_outlives_one_observation_that_missed_it() {
    let mut infos = Infos::default();
    infos.insert(info("t2"));
    // The initial subscribed snapshot may predate the Core run result.
    assert!(infos.sync(1, &[info("t1")], true));
    assert!(infos.get(&TerminalId::new("t2").unwrap()).is_some());
    assert!(infos.get(&TerminalId::new("t1").unwrap()).is_some());
    // The next observed revision is authoritative.
    assert!(infos.sync(2, &[info("t1")], true));
    assert!(infos.get(&TerminalId::new("t2").unwrap()).is_none());
}

#[test]
fn a_new_observation_replaces_old_facts() {
    let mut infos = Infos::default();
    assert!(infos.sync(1, &[info("t1")], true));
    assert!(!infos.sync(1, &[], true));
    assert!(infos.get(&TerminalId::new("t1").unwrap()).is_some());
    assert!(infos.sync(2, &[], true));
    assert!(infos.get(&TerminalId::new("t1").unwrap()).is_none());
}

#[test]
fn a_lost_observation_clears_a_recent_run() {
    let mut infos = Infos::default();
    infos.insert(info("t1"));
    assert!(infos.sync(1, &[], false));
    assert!(infos.get(&TerminalId::new("t1").unwrap()).is_none());
}
