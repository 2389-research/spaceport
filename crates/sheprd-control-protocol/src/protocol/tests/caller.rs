//! Every request family names its caller the same way.
use super::*;
use crate::protocol::{
    app::{AppOp, Switch},
    events::{Condition, EventsOp},
    hosts::HostOp,
    settings::SettingsOp,
    tabs::TabOp,
    work::WorkOp,
};

#[test]
fn every_request_family_names_its_caller() {
    let caller = Caller {
        window: Some(7),
        ..Caller::default()
    };
    fn call<T>(caller: &Caller, op: T) -> Call<T> {
        Call {
            caller: caller.clone(),
            op,
        }
    }
    // The families whose operations need no fixture beyond a value; the
    // match in `Request::caller` names every family, so a new one cannot be
    // left out there.
    for request in [
        Request::Status(caller.clone()),
        Request::App(call(&caller, AppOp::Focus)),
        Request::Tab(call(&caller, TabOp::Devices)),
        Request::Work(call(&caller, WorkOp::Notes)),
        Request::Host(call(&caller, HostOp::List)),
        Request::Settings(call(
            &caller,
            SettingsOp::Caffeine {
                switch: Some(Switch::On),
                seconds: None,
            },
        )),
        Request::Events(call(
            &caller,
            EventsOp::Wait {
                condition: Condition::Online,
                wait_seconds: 1,
            },
        )),
        Request::Link(call(
            &caller,
            LinkOp {
                url: "herdr-gpui://x".into(),
            },
        )),
    ] {
        assert_eq!(request.caller(), &caller, "{request:?}");
    }
    assert_eq!(Request::Status(Caller::default()).caller().window, None);
}
