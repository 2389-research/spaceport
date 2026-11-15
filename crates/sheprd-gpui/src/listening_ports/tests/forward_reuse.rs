//! A click on a remote port uses the forward the host menu or `spaceport
//! ports forward` already runs for it, rather than a second SSH child.

use super::*;
use crate::{port_forward::State, sidebar::layout_tests::fixture_window};
use gpui::TestAppContext;

/// A host that never resolves, should a tunnel be started after all.
const TARGET: &str = "me@devbox.invalid";

fn key(port: u16, address: Option<IpAddr>) -> Key {
    Key {
        target: TARGET.into(),
        address,
        port,
    }
}

/// Clicks `key`'s port in a window that already forwards `forwarded` from
/// `TARGET`, and returns the page opened, whether a tunnel was started, and
/// the flash shown.
fn click(
    cx: &mut TestAppContext,
    forwarded: Option<(u16, State)>,
    key: Key,
) -> (Option<String>, bool, Option<String>) {
    let (view, cx) = cx.add_window_view(fixture_window);
    let (started, flash) = cx.update(|window, cx| {
        view.update(cx, |view, cx| {
            if let Some((port, state)) = forwarded {
                view.port_forwards.fixture(TARGET, port, state);
            }
            let endpoint = view.endpoints[0].id.clone();
            view.open_port_link(&endpoint, "w0", &Link::Tunnel(key.clone()), window, cx);
            let started = view.tunnels.attempt(&key).is_some();
            // Whatever was started must not outlive the test.
            view.tunnels.close_all();
            let flash = view.flash.as_ref().map(|(flash, _)| flash.text.to_string());
            (started, flash)
        })
    });
    (cx.opened_url(), started, flash)
}

#[gpui::test]
fn a_port_click_opens_the_listening_forward_of_that_port(cx: &mut TestAppContext) {
    let listening = State::Listening { local_port: 40_001 };
    let (opened, started, flash) = click(cx, Some((5173, listening)), key(5173, None));
    assert_eq!(opened.as_deref(), Some("http://127.0.0.1:40001/"));
    assert!(!started, "a second SSH child was started for the port");
    assert_eq!(flash, None);
}

#[gpui::test]
fn a_port_click_waits_for_a_forward_that_is_still_connecting(cx: &mut TestAppContext) {
    let (opened, started, flash) = click(cx, Some((5173, State::Starting)), key(5173, None));
    assert_eq!(opened, None);
    assert!(!started, "a second SSH child was started for the port");
    assert_eq!(
        flash.as_deref(),
        Some("A tunnel to that port is already opening")
    );
}

/// Everything else still gets a tunnel of its own, as before.
#[test]
fn a_port_no_running_forward_reaches_is_still_tunnelled() {
    use super::super::render::forwarded;
    use crate::port_forward::{PortForwards, Reuse};
    let mut forwards = PortForwards::default();
    forwards.fixture(TARGET, 5173, State::Listening { local_port: 40_001 });
    forwards.fixture(TARGET, 8080, State::Ended("ssh exited".into()));
    assert_eq!(
        forwarded(&forwards, &key(5173, None)),
        Some(Reuse::Listening { local_port: 40_001 })
    );
    // A forward dials the remote's localhost, which a server bound to one
    // other address never answers on.
    let lan = IpAddr::V4(Ipv4Addr::new(10, 0, 0, 2));
    assert_eq!(forwarded(&forwards, &key(5173, Some(lan))), None);
    assert_eq!(forwarded(&forwards, &key(8080, None)), None);
    assert_eq!(forwarded(&forwards, &key(3000, None)), None);
    let elsewhere = Key {
        target: "me@other.invalid".into(),
        address: None,
        port: 5173,
    };
    assert_eq!(forwarded(&forwards, &elsewhere), None);
}
