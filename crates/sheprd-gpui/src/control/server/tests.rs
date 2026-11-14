#![allow(clippy::unwrap_used)]

use super::*;

mod core_status;
#[cfg(windows)]
mod pipe_status;

#[test]
fn a_huge_wait_is_clamped_instead_of_overflowing() {
    let now = Instant::now();
    assert_eq!(
        deadline(Duration::MAX, now),
        now + Duration::from_secs(MAX_WAIT_SECONDS)
    );
    assert_eq!(
        deadline(Duration::from_secs(5), now),
        now + Duration::from_secs(5)
    );
}

fn call<T>(op: T) -> crate::control::protocol::Call<T> {
    crate::control::protocol::Call {
        caller: Caller::default(),
        op,
    }
}

#[test]
fn only_requests_the_protocol_says_wait_may_park() {
    use crate::control::protocol::{
        app::Switch, events::EventsOp, hosts::HostOp, settings::SettingsOp, tabs::TabOp,
        work::WorkOp,
    };
    for parks in [
        Request::Tab(call(TabOp::Devices)),
        Request::Host(call(HostOp::OpenPort { port: 80 })),
        Request::Settings(call(SettingsOp::Fonts)),
        Request::Settings(call(SettingsOp::Caffeine {
            switch: Some(Switch::On),
            seconds: None,
        })),
        Request::Events(call(EventsOp::Poll {
            after: None,
            instance: None,
            wait_seconds: 30,
            kinds: Vec::new(),
        })),
    ] {
        assert!(may_park(&parks), "{parks:?}");
    }
    // A full list of waiting requests refuses only requests that would join
    // it: one answered at once is never turned away for them.
    for answers_at_once in [
        Request::Status(Caller::default()),
        Request::Work(call(WorkOp::Notes)),
        Request::Work(call(WorkOp::SetNote { text: "x".into() })),
        Request::Work(call(WorkOp::Mute { muted: true })),
        Request::Tab(call(TabOp::List {})),
        Request::Host(call(HostOp::Ports)),
        Request::Settings(call(SettingsOp::Bell)),
    ] {
        assert!(!may_park(&answers_at_once), "{answers_at_once:?}");
    }
}
