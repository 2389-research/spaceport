//! The link machine's own rules: what each transition asks of the socket, what
//! phase it leaves, and when a wake or a network change recovers an attempt.
use super::*;
use spaceport_client::ConnectTarget;
use spaceport_live::{ConnectionStatus, LiveState};
use std::sync::Arc;
use std::time::{Duration, Instant};

const IDLE: Transport = Transport {
    handled: false,
    alive: false,
    can_restart: true,
};

const DIALING: Transport = Transport {
    handled: true,
    alive: true,
    can_restart: true,
};

fn ssh() -> ConnectTarget {
    ConnectTarget::Ssh {
        target: "me@homelab".into(),
        session: "default".into(),
    }
}

fn connected(host: &mut Host) {
    host.live_mut().status = ConnectionStatus::Connected;
    host.live_mut().snapshot = Some(Arc::new(snapshot()));
}

#[test]
fn only_a_disabled_phase_is_not_enabled() {
    // A row reads whether a host wants a connection off the phase it holds, so
    // every phase but `Disabled` must answer as the link does.
    let phases = [
        Phase::Detached,
        Phase::DaemonOutdated,
        Phase::ClientOutdated,
        Phase::HerdrMissing,
        Phase::HerdrStopped,
        Phase::Reconnecting,
        Phase::Connecting,
        Phase::Online,
    ];
    assert!(phases.iter().all(|phase| phase.enabled()));
    assert!(!Phase::Disabled.enabled());
}

#[test]
fn a_disabled_host_is_disabled_and_wants_nothing() {
    let host = Host::new(false);
    assert!(!host.enabled());
    assert_eq!(host.phase(), Phase::Disabled);
    assert_eq!(host.phase().as_str(), "disabled");
    assert!(!host.may_dial(true));
}

#[test]
fn an_enabled_host_waits_for_its_first_dial() {
    let host = Host::new(true);
    assert!(host.enabled());
    assert_eq!(host.phase(), Phase::Connecting);
    assert!(host.may_dial(true));
    // A dial waits for the inbox, which is where a worker's last events land.
    assert!(!host.may_dial(false));
}

#[test]
fn connecting_asks_for_a_dial_and_backs_off_further_each_time() {
    let mut host = Host::new(true);
    assert_eq!(host.enter(Transition::Connect), Action::None);
    assert_eq!(host.phase(), Phase::Connecting);
    // The first attempt is one second in: the curve counts from one attempt.
    assert_eq!(host.retry_delay(), Duration::from_secs(1));
    host.enter(Transition::Connect);
    assert_eq!(host.retry_delay(), Duration::from_secs(2));
    // A settled link starts its backoff over.
    assert_eq!(host.enter(Transition::ResetAttempts), Action::None);
    assert_eq!(host.retry_delay(), Duration::from_millis(500));
}

#[test]
fn stopping_retires_the_transport_and_waits_to_dial_again() {
    let mut host = Host::new(true);
    assert_eq!(host.enter(Transition::Stop), Action::Detach);
    assert_eq!(host.phase(), Phase::Connecting);
    assert!(host.may_dial(true));
}

#[test]
fn detaching_stops_dialing_until_asked() {
    let mut host = Host::new(true);
    assert_eq!(host.enter(Transition::Detach), Action::Detach);
    assert!(host.detached());
    assert_eq!(host.phase(), Phase::Detached);
    assert!(!host.may_dial(true));
}

#[test]
fn retargeting_hands_back_the_target_to_dial() {
    let mut host = Host::new(true);
    host.enter(Transition::Connect);
    host.enter(Transition::Connect);
    assert_eq!(host.retry_delay(), Duration::from_secs(2));
    assert_eq!(
        host.enter(Transition::Retarget {
            target: ssh(),
            enabled: true,
        }),
        Action::Retire { target: ssh() }
    );
    assert_eq!(host.retry_delay(), Duration::from_millis(500));
    assert_eq!(
        host.enter(Transition::Retarget {
            target: ssh(),
            enabled: false,
        }),
        Action::Retire { target: ssh() }
    );
    assert_eq!(host.phase(), Phase::Disabled);
}

#[test]
fn a_lost_transport_waits_its_backoff_and_keeps_the_words_it_has() {
    let mut host = Host::new(true);
    let now = Instant::now();
    assert_eq!(
        host.enter(Transition::Lost {
            now,
            lost: "connection lost".into(),
        }),
        Action::None
    );
    assert_eq!(host.phase(), Phase::Reconnecting);
    assert_eq!(host.outage(), Some("connection lost"));
    assert_eq!(host.retry_at(), now + host.retry_delay());
}

#[test]
fn a_serving_link_is_online_and_a_detached_one_says_why() {
    let mut host = Host::new(true);
    connected(&mut host);
    assert!(host.serving(DIALING));
    // A worker that ended, or one that has not delivered a snapshot, does not serve.
    assert!(!host.serving(IDLE));
    assert_eq!(
        host.enter(Transition::Observe {
            now: Instant::now(),
            transport: DIALING,
        }),
        Action::None
    );
    assert_eq!(host.phase(), Phase::Online);
    // An accepted handshake replaces any advice the outage carried.
    assert_eq!(host.outage(), None);
}

#[test]
fn a_missing_or_stopped_herdr_says_which_and_waits_for_the_user() {
    let mut host = Host::new(true);
    host.live_mut().missing_installation = true;
    host.enter(Transition::Observe {
        now: Instant::now(),
        transport: IDLE,
    });
    assert_eq!(host.phase(), Phase::HerdrMissing);
    assert!(host.herdr_missing());
    assert!(host.waits_for_user());
    // Even a thorough recovery leaves a refusal only the user can fix alone.
    let now = Instant::now();
    assert_eq!(
        host.recover(Trigger::Sleep, now, &ssh(), IDLE, || Some(false)),
        Some(Action::None)
    );
    // A thorough recovery leaves the retry where the refusal put it.
    assert!(host.retry_at() > now + Duration::from_secs(20));

    let mut stopped = Host::new(true);
    stopped.live_mut().server_stopped = true;
    stopped.enter(Transition::Observe {
        now: Instant::now(),
        transport: IDLE,
    });
    assert_eq!(stopped.phase(), Phase::HerdrStopped);
    assert!(stopped.herdr_stopped());
    assert!(!stopped.may_dial(true));
}

#[test]
fn a_target_that_cannot_be_restarted_does_not_blame_herdr() {
    let mut host = Host::new(true);
    host.live_mut().server_stopped = true;
    host.enter(Transition::Observe {
        now: Instant::now(),
        transport: Transport {
            can_restart: false,
            ..IDLE
        },
    });
    assert!(!host.herdr_stopped());
    assert_eq!(host.phase(), Phase::Connecting);
}

#[test]
fn recovering_a_live_link_asks_it_to_prove_itself() {
    let mut host = Host::new(true);
    assert_eq!(
        host.recover(
            Trigger::PathChanged,
            Instant::now(),
            &ssh(),
            DIALING,
            || { Some(true) }
        ),
        Some(Action::CheckLiveness)
    );
}

#[test]
fn a_report_the_inbox_cannot_give_waits_for_the_next_poll() {
    let mut host = Host::new(true);
    assert_eq!(
        host.recover(
            Trigger::NetworkRestored,
            Instant::now(),
            &ssh(),
            DIALING,
            || None
        ),
        None
    );
    // The report is not even asked for when the rules do not need it.
    let mut asked = false;
    assert_eq!(
        host.recover(
            Trigger::PathChanged,
            Instant::now(),
            &ssh(),
            DIALING,
            || {
                asked = true;
                Some(true)
            }
        ),
        Some(Action::CheckLiveness)
    );
    assert!(!asked);
}

#[test]
fn a_remote_attempt_the_worker_disowned_is_retired_and_redialed() {
    let mut host = Host::new(true);
    host.enter(Transition::Connect);
    assert_eq!(
        host.recover(
            Trigger::NetworkRestored,
            Instant::now(),
            &ssh(),
            DIALING,
            || Some(false)
        ),
        Some(Action::Detach)
    );
    // The stop leaves a waiting link, and a thorough recovery dials it at once.
    assert!(host.may_dial(true));
    assert_eq!(host.phase(), Phase::Connecting);
    assert!(host.retry_at() <= Instant::now());
}

#[test]
fn a_dropped_host_is_dialed_at_once_after_a_wake() {
    let mut host = Host::new(true);
    host.enter(Transition::Lost {
        now: Instant::now() + Duration::from_secs(30),
        lost: "connection lost".into(),
    });
    let now = Instant::now();
    assert_eq!(
        host.recover(Trigger::NetworkRestored, now, &ssh(), IDLE, || None),
        Some(Action::None)
    );
    assert_eq!(host.retry_at(), now);
    assert!(host.may_dial(true));
}

#[test]
fn the_selected_host_reads_the_windows_copy() {
    let mut hosts = [Host::new(true), Host::new(true)];
    hosts[1].live_mut().window_title = Some("remote".into());
    let window = LiveState {
        window_title: Some("window".into()),
        ..LiveState::default()
    };
    // Host 0 is selected: the window's copy answers, not the host's.
    assert_eq!(
        live_of(0, &window, 0, |index| &hosts[index])
            .window_title
            .as_deref(),
        Some("window")
    );
    assert_eq!(
        live_of(0, &window, 1, |index| &hosts[index])
            .window_title
            .as_deref(),
        Some("remote")
    );
}

#[test]
fn a_connection_reference_names_one_attempt_and_one_mailbox() {
    let inbox = Arc::new(std::sync::Mutex::new(LiveState::default()));
    let source = ConnectionRef::new("local".into(), 2, inbox.clone());
    assert!(source.names("local", 2, &inbox));
    assert!(!source.names("local", 3, &inbox));
    assert!(!source.names(
        "local",
        2,
        &Arc::new(std::sync::Mutex::new(LiveState::default()))
    ));
    let posted = source.downgrade();
    assert!(
        posted
            .upgrade()
            .is_some_and(|it| it.names("local", 2, &inbox))
    );
    drop(source);
    drop(inbox);
    assert!(posted.upgrade().is_none());
}
