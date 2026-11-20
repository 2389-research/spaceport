// ABOUTME: Checks repeated host selection refusals through the real window handler.
// ABOUTME: Keeps each control reply and status notice aligned with its typed cause.
//! `host select` passes on why the window refused, each time it refuses.
use super::*;
use crate::i18n::Localize as _;
use crate::state::ConnectionStatus;

/// A window whose selected host is connected but cannot hand its surface
/// over, beside a saved host to switch to.
fn stuck_on_local(view: &mut SheprdWindow) {
    let mut devbox = crate::endpoint::Endpoint::new(
        "ssh:devbox".into(),
        "Devbox".into(),
        sheprd_client::ConnectTarget::Ssh {
            target: "me@devbox".into(),
            session: "default".into(),
        },
        false,
    );
    devbox.set_enabled_for_test(true);
    view.endpoints.push(devbox);
    view.selected_endpoint = 0;
    view.endpoints[0].live.status = ConnectionStatus::Connected;
    view.endpoints[0].live.supports_surface = false;
}

#[gpui::test]
fn the_same_refusal_twice_in_a_row_says_why_both_times(cx: &mut TestAppContext) {
    let (view, cx) = window(cx);
    view.update(cx, |view, cx| {
        stuck_on_local(view);
        let reason = crate::endpoint::Error::SurfaceSwitchingUnsupported.localize();
        let refused = Response::error(ErrorCode::Failed, reason.as_str());
        for attempt in 1..=2 {
            let answer = select(view, "ssh:devbox", |view, id| view.select_endpoint(id, cx));
            assert_eq!(answer, refused, "attempt {attempt}");
            assert_eq!(view.selected_endpoint, 0);
            // The status bar shows the refusal as before.
            assert_eq!(view.notices.text(), Some(reason.as_str()));
        }
    });
}
