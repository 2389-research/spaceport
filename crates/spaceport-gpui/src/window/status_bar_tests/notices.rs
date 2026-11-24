// ABOUTME: Exercises transient local and live notice expiry through the window tick owner.
// ABOUTME: Keeps disconnected diagnostics authoritative through late operation failures.
use super::*;
use crate::{notice::Cause, state::ConnectionStatus};
use std::time::{Duration, Instant};

#[gpui::test]
fn local_and_live_operation_notices_expire_without_a_host_switch(cx: &mut TestAppContext) {
    let (view, cx) = cx.add_window_view(fixture_window);
    view.update(cx, |view, _| {
        view.live.status = ConnectionStatus::Connected;
        view.notices.show(Cause::Transfer, "files busy".into());
        view.live
            .notices
            .show(Cause::Daemon, "request refused".into());
        *view.endpoints[0].live_mut() = view.live.clone();
        assert!(
            crate::state::status_text(&view.live, view.notices.text(), Instant::now())
                .contains("files busy")
        );
        assert!(view.expire_status_notices(Instant::now() + Duration::from_secs(11)));
        assert_eq!(
            crate::state::status_text(&view.live, view.notices.text(), Instant::now()),
            "Connected"
        );
        assert!(view.notices.is_none());
        assert!(view.live.notices.is_none());
        assert!(view.endpoints[0].live().notices.is_none());
    });
}

#[gpui::test]
fn disconnection_reason_survives_expiry_and_hides_local_failures(cx: &mut TestAppContext) {
    let (view, cx) = cx.add_window_view(fixture_window);
    view.update(cx, |view, _| {
        view.live.apply(
            spaceport_client::ClientEvent::Disconnected {
                reason: "socket closed".into(),
                ssh: None,
            },
            &crate::state::labels(),
        );
        view.notices.show(Cause::Input, "old input error".into());
        view.live.apply(
            spaceport_client::ClientEvent::CommandRejected {
                request_id: None,
                reason: spaceport_client::Error::Disconnected,
            },
            &crate::state::labels(),
        );
        assert_eq!(
            crate::state::status_text(&view.live, view.notices.text(), Instant::now()),
            "\u{2068}Disconnected\u{2069}: \u{2068}socket closed\u{2069}"
        );
        view.expire_status_notices(Instant::now() + Duration::from_secs(11));
        assert_eq!(view.live.notices.connection_reason(), Some("socket closed"));
        assert_eq!(
            crate::state::status_text(&view.live, view.notices.text(), Instant::now()),
            "\u{2068}Disconnected\u{2069}: \u{2068}socket closed\u{2069}"
        );
    });
}
