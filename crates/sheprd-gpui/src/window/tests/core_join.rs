use super::*;
use crate::core_terminals::{Error, Start, State};
use sheprd_client::ConnectTarget;

fn private_join(start: Start) -> Result<sheprd_core_client::CoreBackend, Error> {
    Err(match start {
        Start::Join => Error::NotInstalled,
        Start::IfMissing => Error::Worker(std::io::Error::other("unexpected Core start")),
    })
}

#[gpui::test]
fn plain_local_window_joins_existing_core_without_starting_it(cx: &mut gpui::TestAppContext) {
    let (view, cx) = cx.add_window_view(fixture_window);
    view.update_in(cx, |view, window, cx| {
        view.endpoints[0].connection.target = ConnectTarget::Local;
        view.core_terminals.hub.set_join_for_test(private_join);
        view.poll_core_terminals(window, cx);
    });
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(1);
    loop {
        let state = view.update(cx, |view, _| {
            view.core_terminals.hub.poll();
            match view.core_terminals.hub.state() {
                State::Idle => "idle",
                State::Connecting(_) => "connecting",
                State::Unavailable(error) if matches!(error.as_ref(), Error::NotInstalled) => {
                    "joined"
                }
                State::Unavailable(_) => "started",
                State::Ready(_) => "ready",
            }
        });
        if state != "connecting" {
            assert_eq!(state, "joined");
            break;
        }
        assert!(
            std::time::Instant::now() < deadline,
            "Core join worker did not answer"
        );
        std::thread::yield_now();
    }
}

#[gpui::test]
fn explicit_socket_window_does_not_join_core(cx: &mut gpui::TestAppContext) {
    let (view, cx) = cx.add_window_view(fixture_window);
    view.update_in(cx, |view, window, cx| {
        view.core_terminals.hub.set_join_for_test(private_join);
        view.poll_core_terminals(window, cx);
        assert!(matches!(view.core_terminals.hub.state(), State::Idle));
    });
}
