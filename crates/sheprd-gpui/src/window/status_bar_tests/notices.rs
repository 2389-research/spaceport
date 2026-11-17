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
        view.endpoints[0].live = view.live.clone();
        assert!(
            view.live
                .status_text(view.notices.text())
                .contains("files busy")
        );
        assert!(view.expire_status_notices(Instant::now() + Duration::from_secs(11)));
        assert_eq!(view.live.status_text(view.notices.text()), "Connected");
        assert!(view.notices.is_none());
        assert!(view.live.notices.is_none());
        assert!(view.endpoints[0].live.notices.is_none());
    });
}

#[gpui::test]
fn disconnection_reason_survives_expiry_and_hides_local_failures(cx: &mut TestAppContext) {
    let (view, cx) = cx.add_window_view(fixture_window);
    view.update(cx, |view, _| {
        view.live.apply(sheprd_client::ClientEvent::Disconnected {
            reason: "socket closed".into(),
            ssh: None,
        });
        view.notices.show(Cause::Input, "old input error".into());
        view.live
            .apply(sheprd_client::ClientEvent::CommandRejected {
                request_id: None,
                reason: sheprd_client::Error::Disconnected,
            });
        assert_eq!(
            view.live.status_text(view.notices.text()),
            "\u{2068}Disconnected\u{2069}: \u{2068}socket closed\u{2069}"
        );
        view.expire_status_notices(Instant::now() + Duration::from_secs(11));
        assert_eq!(view.live.notices.connection_reason(), Some("socket closed"));
        assert_eq!(
            view.live.status_text(view.notices.text()),
            "\u{2068}Disconnected\u{2069}: \u{2068}socket closed\u{2069}"
        );
    });
}
