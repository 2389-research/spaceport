//! `wait online` waits on the selected host of one window, so a caller in a
//! session no window shows is refused at once instead of waiting out the
//! timeout on another session's window.
use super::*;

#[gpui::test]
fn a_wait_for_a_host_of_another_session_is_refused_at_once(cx: &mut gpui::TestAppContext) {
    let (_view, cx) = window(cx);
    let refused = cx
        .cx
        .update(|cx| WaitFor::resolve(Condition::Online, &caller(Some("w_other_session")), cx));
    // The window shows w0 and w3, never w_other_session: a pane of a session
    // the app does not show gets exit 5, as every other command does, rather
    // than "the wait was not met" after the timeout.
    assert_eq!(
        error_code(&refused.unwrap_err()),
        Some(ErrorCode::WorkspaceNotFound)
    );
    cx.cx.update(|cx| {
        // The caller's own workspace, or none at all, resolves to a window.
        assert!(WaitFor::resolve(Condition::Online, &caller(Some("w3")), cx).is_ok());
        assert!(WaitFor::resolve(Condition::Online, &caller(None), cx).is_ok());
    });
}

#[gpui::test]
fn a_parked_online_wait_stays_with_the_original_window(cx: &mut gpui::TestAppContext) {
    use crate::control::protocol::Request;
    use crate::state::ConnectionStatus;
    let (original, cx) = window(cx);
    original.update(cx, |view, _| {
        view.live.status = ConnectionStatus::Connecting
    });
    let address = socket::test_address();
    let server = socket::Server::bind(&address.path).unwrap();
    let call = Call {
        caller: Caller::default(),
        op: EventsOp::Wait {
            condition: Condition::Online,
            wait_seconds: 30,
        },
    };
    let request = Request::Events(call.clone());
    let path = address.path.clone();
    let waiting = std::thread::spawn(move || socket::call(&path, &request));
    let incoming = server.next().unwrap();
    let mut watches = Watches::default();
    cx.cx.update(|cx| serve(call, incoming, &mut watches, cx));
    let other = cx.add_window(crate::sidebar::layout_tests::fixture_window);
    other
        .update(cx, |view, window, _| {
            view.live.status = ConnectionStatus::Connected;
            window.activate_window();
        })
        .unwrap();
    cx.cx.update(|cx| {
        assert_eq!(window_for(&Caller::default(), cx).unwrap(), other);
        watches.serve(cx);
        assert_eq!(watches.0.len(), 1);
    });
    original.update(cx, |view, _| view.live.status = ConnectionStatus::Connected);
    cx.cx.update(|cx| watches.serve(cx));
    assert!(matches!(
        waiting.join().unwrap().unwrap(),
        Response::Events {
            reply: Reply::Waited { met: true }
        }
    ));
}

#[gpui::test]
fn a_closed_online_target_stays_unmet_while_another_window_is_online(
    cx: &mut gpui::TestAppContext,
) {
    let (_, cx) = window(cx);
    let target = cx
        .cx
        .update(|cx| WaitFor::resolve(Condition::Online, &Caller::default(), cx).unwrap());
    let other = cx.add_window(crate::sidebar::layout_tests::fixture_window);
    other
        .update(cx, |view, _, _| {
            view.live.status = crate::state::ConnectionStatus::Connected
        })
        .unwrap();
    cx.update(|window, _| window.remove_window());
    cx.cx.update(|cx| assert!(!target.met(cx)));
}
