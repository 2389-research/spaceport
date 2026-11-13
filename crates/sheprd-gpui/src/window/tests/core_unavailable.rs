// ABOUTME: Checks core unavailable behavior with isolated application fixtures.
// ABOUTME: Makes Core opt-in explicit while preserving retained behavior checks.
use crate::{
    i18n::leak_tests::{draw, under_en_xa},
    state::ConnectionStatus,
};

#[test]
fn missing_legacy_daemon_does_not_show_herdr_install_in_status() {
    under_en_xa("Core fallback status", |cx, recorder| {
        let (view, cx) = cx.add_window_view(crate::sidebar::layout_tests::fixture_core_window);
        view.update(cx, |view, cx| {
            view.live.status = ConnectionStatus::Disconnected;
            view.live.missing_installation = true;
            view.live.error = Some(
                "Could not start herdr server: missing. Install Herdr and use Terminal > Reconnect."
                    .into(),
            );
            cx.notify();
        });
        draw(cx);
        let shown = recorder.take();
        assert!(
            !shown.iter().any(|line| line.contains("Install Herdr")),
            "legacy install instruction reached the window: {shown:?}"
        );
    });
}

#[test]
fn connecting_and_unavailable_core_show_the_same_guidance() {
    for unavailable in [false, true] {
        under_en_xa("Core not connected status", |cx, recorder| {
            let (view, cx) = cx.add_window_view(crate::sidebar::layout_tests::fixture_core_window);
            view.update(cx, |view, cx| {
                view.live.status = ConnectionStatus::Disconnected;
                view.live.missing_installation = true;
                if unavailable {
                    view.core_terminals
                        .hub
                        .set_unavailable_for_test(crate::core_terminals::Error::NotInstalled);
                } else {
                    view.core_terminals.hub.set_connecting_for_test();
                }
                cx.notify();
            });
            draw(cx);
            let expected = crate::i18n::fl!("status-bar-core-not-connected");
            let shown = recorder.take();
            assert!(
                shown.iter().any(|line| line.contains(&expected)),
                "Core guidance missing from the window: {shown:?}"
            );
        });
    }
}

#[test]
fn ready_core_status_names_only_the_terminal_fallback() {
    assert_eq!(
        super::super::status_bar::core_fallback_status(true),
        "Herdr Core connected (terminal fallback)"
    );
}
