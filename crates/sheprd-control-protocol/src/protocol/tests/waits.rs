//! Which requests wait: the one classification the caller's read timeout and
//! the app's parking both derive from.
use super::*;
use crate::protocol::{
    app::Switch,
    events::{Condition, EventsOp},
    hosts::HostOp,
    settings::SettingsOp,
    tabs::TabOp,
    work::WorkOp,
};

fn call<T>(op: T) -> Call<T> {
    Call {
        caller: Caller::default(),
        op,
    }
}

/// Every operation the app parks while it waits for something to happen.
fn parking() -> Vec<Request> {
    let mut requests = vec![
        Request::Tab(call(TabOp::Devices)),
        Request::Host(call(HostOp::OpenPort { port: 80 })),
        Request::Settings(call(SettingsOp::Theme { name: "x".into() })),
        Request::Settings(call(SettingsOp::Fonts)),
        Request::Settings(call(SettingsOp::Font { name: None })),
        Request::Settings(call(SettingsOp::Integrations)),
        Request::Events(call(EventsOp::Poll {
            after: None,
            instance: None,
            wait_seconds: 1,
            kinds: Vec::new(),
        })),
        Request::Events(call(EventsOp::Wait {
            condition: Condition::Online,
            wait_seconds: 1,
        })),
    ];
    for switch in [Switch::On, Switch::Off, Switch::Toggle] {
        for seconds in [None, Some(60)] {
            requests.push(Request::Settings(call(SettingsOp::Caffeine {
                switch: Some(switch),
                seconds,
            })));
        }
    }
    requests
}

#[test]
fn every_operation_the_app_parks_reports_a_wait() {
    for request in parking() {
        assert!(request.wait_seconds() > 0, "{request:?}");
    }
}

#[test]
fn no_work_operation_waits() {
    for op in [
        WorkOp::Notes,
        WorkOp::Note,
        WorkOp::SetNote { text: "x".into() },
        WorkOp::MuteStatus,
        WorkOp::Mute { muted: true },
    ] {
        assert_eq!(Request::Work(call(op)).wait_seconds(), 0);
    }
}

#[test]
fn reading_caffeine_does_not_wait() {
    let report = Request::Settings(call(SettingsOp::Caffeine {
        switch: None,
        seconds: None,
    }));
    assert_eq!(report.wait_seconds(), 0);
}
