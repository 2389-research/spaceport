// ABOUTME: Checks wait admission and observations through real private control requests.
// ABOUTME: Refuses unknown tabs and unshown sessions before a watch can park.
use super::*;
use crate::{
    control::protocol::{Caller, ErrorCode, Request},
    sidebar::layout_tests::{fixture_window, snapshot},
};

fn request(condition: Condition, caller: Caller) -> Request {
    Request::Events(Call {
        caller,
        op: EventsOp::Wait {
            condition,
            wait_seconds: 1,
        },
    })
}

fn answer(request: Request, cx: &mut App) -> (Response, usize) {
    let address = socket::test_address();
    let server = socket::Server::bind(&address.path).unwrap();
    let path = address.path.clone();
    let waiting = std::thread::spawn(move || socket::call(&path, &request));
    let incoming = server.next().unwrap();
    let mut state = crate::control::server::State::default();
    crate::control::server::serve(incoming, &mut state, cx);
    let parked = state.watches.0.len();
    // End any incorrectly parked wait without delaying this regression test.
    for watch in state.watches.0.drain(..) {
        watch
            .incoming
            .respond(Response::error(ErrorCode::Timeout, "unexpectedly parked"));
    }
    (waiting.join().unwrap().unwrap(), parked)
}

#[gpui::test]
fn an_unknown_tab_is_not_already_closed(cx: &mut gpui::TestAppContext) {
    cx.update(|cx| {
        let (response, parked) = answer(
            request(Condition::TabClosed { id: 991 }, Caller::default()),
            cx,
        );
        assert!(
            matches!(
                response,
                Response::Error {
                    code: ErrorCode::NotFound,
                    ..
                }
            ),
            "{response:?}"
        );
        assert_eq!(parked, 0);
    });
}

#[gpui::test]
fn online_waits_refuse_an_unshown_session_before_parking(cx: &mut gpui::TestAppContext) {
    let (_view, cx) = cx.add_window_view(|window, cx| {
        let mut view = fixture_window(window, cx);
        view.live.snapshot = Some(std::sync::Arc::new(snapshot(2)));
        view
    });
    cx.cx.update(|cx| {
        let caller = Caller {
            workspace_id: Some("w0".into()),
            daemon_socket: Some("/tmp/unshown-session.sock".into()),
            ..Caller::default()
        };
        let (response, parked) = answer(request(Condition::Online, caller), cx);
        assert!(
            matches!(
                response,
                Response::Error {
                    code: ErrorCode::WorkspaceNotFound | ErrorCode::NotFound,
                    ..
                }
            ),
            "{response:?}"
        );
        assert_eq!(parked, 0);
    });
}

#[gpui::test]
fn an_online_wait_observes_its_endpoint_instead_of_another_connected_host(
    cx: &mut gpui::TestAppContext,
) {
    let (view, cx) = cx.add_window_view(|window, cx| {
        let mut view = fixture_window(window, cx);
        view.live.snapshot = Some(std::sync::Arc::new(snapshot(2)));
        view.live.status = crate::state::ConnectionStatus::Connected;
        let mut other = crate::endpoint::Endpoint::new(
            "other".into(),
            "Other".into(),
            sheprd_client::ConnectTarget::Socket("/tmp/wait-other.sock".into()),
            true,
        );
        other.live.snapshot = Some(std::sync::Arc::new(snapshot(2)));
        view.endpoints.push(other);
        view
    });
    cx.cx.update(|cx| {
        let caller = Caller {
            workspace_id: Some("w0".into()),
            daemon_socket: Some("/tmp/wait-other.sock".into()),
            ..Caller::default()
        };
        let mut check = condition_check(Condition::Online, caller, cx).unwrap();
        assert!(check(cx).is_none());
        view.update(cx, |view, _| {
            view.endpoints[1].live.status = crate::state::ConnectionStatus::Connected
        });
        assert!(matches!(
            check(cx),
            Some(Response::Events {
                reply: Reply::Waited { met: true }
            })
        ));
    });
}

#[gpui::test]
fn a_known_tab_wait_completes_only_after_the_tab_closes(cx: &mut gpui::TestAppContext) {
    let (view, cx) = cx.add_window_view(|window, cx| {
        let mut view = fixture_window(window, cx);
        view.live.snapshot = Some(std::sync::Arc::new(snapshot(2)));
        view
    });
    cx.cx.update(|cx| {
        let scope = crate::browser::scope(&view.read(cx).endpoints[0]);
        let tab = Store::update(cx, |store| {
            store.open(scope, "w0", Some(crate::browser::Location::Devices), None)
        })
        .unwrap();
        let mut check = condition_check(
            Condition::TabClosed { id: tab.into() },
            Caller::default(),
            cx,
        )
        .unwrap();
        assert!(check(cx).is_none());
        Store::update(cx, |store| store.close(tab));
        assert!(matches!(
            check(cx),
            Some(Response::Events {
                reply: Reply::Waited { met: true }
            })
        ));
    });
}

#[gpui::test]
fn closure_waits_refuse_tabs_outside_the_callers_workspace(cx: &mut gpui::TestAppContext) {
    let (view, cx) = cx.add_window_view(|window, cx| {
        let mut view = fixture_window(window, cx);
        view.live.snapshot = Some(std::sync::Arc::new(snapshot(2)));
        view
    });
    cx.cx.update(|cx| {
        let scope = crate::browser::scope(&view.read(cx).endpoints[0]);
        let tab = Store::update(cx, |store| {
            store.open(scope, "w1", Some(crate::browser::Location::Devices), None)
        })
        .unwrap();
        let caller = Caller {
            workspace_id: Some("w0".into()),
            ..Caller::default()
        };
        let (response, parked) =
            answer(request(Condition::TabClosed { id: tab.into() }, caller), cx);
        assert!(matches!(
            response,
            Response::Error {
                code: ErrorCode::NotFound,
                ..
            }
        ));
        assert_eq!(parked, 0);
        assert!(cx.global::<Store>().get(tab).is_some());
    });
}

#[gpui::test]
fn online_waits_do_not_follow_another_session_selected_in_the_same_window(
    cx: &mut gpui::TestAppContext,
) {
    let (view, cx) = cx.add_window_view(|window, cx| {
        let mut view = fixture_window(window, cx);
        view.live.snapshot = Some(std::sync::Arc::new(snapshot(2)));
        view
    });
    cx.cx.update(|cx| {
        let mut check = condition_check(Condition::Online, Caller::default(), cx).unwrap();
        assert!(check(cx).is_none());
        view.update(cx, |view, _| {
            view.endpoints[0] = crate::endpoint::Endpoint::new(
                "replacement".into(),
                "Replacement".into(),
                sheprd_client::ConnectTarget::Socket("/tmp/replacement-wait.sock".into()),
                true,
            );
            view.live.status = crate::state::ConnectionStatus::Connected;
        });
        assert!(matches!(
            check(cx),
            Some(Response::Error {
                code: ErrorCode::NotFound,
                ..
            })
        ));
    });
}
