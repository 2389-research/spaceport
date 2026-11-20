// ABOUTME: Checks that canceled session forms do not lend scroll state to later forms.
// ABOUTME: Uses connected display facts without discovery, submission, or process startup.
use super::*;
use gpui::{ScrollDelta, ScrollEvent};
use sheprd_client::{RemoteHost, RemoteSession};

#[gpui::test]
fn canceling_a_scrolled_form_does_not_scroll_the_next_create_or_delete(cx: &mut TestAppContext) {
    for deleting in [false, true] {
        let (view, cx) = cx.add_window_view(|window, cx| {
            let mut view = fixture_window(window, cx);
            view.endpoints[0].connection.target =
                ConnectTarget::Socket("/synthetic/no-daemon.sock".into());
            let mut endpoint = Endpoint::new(
                "prepared-remote".into(),
                "Prepared".into(),
                ConnectTarget::Ssh {
                    target: "prepared.invalid".into(),
                    session: "profile-session".into(),
                },
                true,
            );
            endpoint.live.status = crate::state::ConnectionStatus::Connected;
            view.sessions.devices.answers.insert(
                endpoint.id.clone(),
                DeviceScan::Sessions(vec![RemoteSession {
                    name: "fixture-session".into(),
                    running: true,
                }]),
            );
            view.endpoints.push(endpoint);
            view.menu.page = Some(Page::Devices);
            view.open_session_create(target(), window, cx);
            assert!(matches!(view.menu.session_edit, Some(Edit::Create { .. })));
            view.menu.error = Some(error());
            view
        });
        cx.simulate_resize(size(px(320.), px(240.)));
        cx.update(|window, cx| full_draw(window, cx).clear(cx));
        let top = cx.debug_bounds("session-modal").unwrap().top();
        cx.simulate_event(ScrollEvent {
            position: point(px(160.), px(120.)),
            delta: ScrollDelta::Pixels(point(px(0.), px(-80.))),
            ..Default::default()
        });
        cx.update(|window, cx| window.draw(cx).clear(cx));
        assert_eq!(
            cx.debug_bounds("session-modal").unwrap().top(),
            top - px(80.)
        );
        cx.simulate_keystrokes("escape");
        view.read_with(cx, |view, _| {
            assert!(view.menu.session_edit.is_none());
            assert_eq!(view.menu.page, Some(Page::Devices));
        });
        cx.update(|window, cx| {
            view.update(cx, |view, cx| {
                if deleting {
                    let row = Row::Device {
                        id: "prepared-remote".into(),
                        session: "fixture-session".into(),
                    };
                    assert!(view.session_delete_reason(&row).is_none());
                    view.confirm_session_delete(row, Some(target()), window, cx);
                    assert!(matches!(view.menu.session_edit, Some(Edit::Delete { .. })));
                } else {
                    view.open_session_create(target(), window, cx);
                    assert!(matches!(view.menu.session_edit, Some(Edit::Create { .. })));
                }
                view.menu.error = Some(error());
                assert!(view.sessions.mutation.is_none());
            });
            window.draw(cx).clear(cx);
        });
        assert_eq!(
            cx.debug_bounds("session-modal").unwrap().top(),
            top,
            "deleting={deleting}"
        );
        view.read_with(cx, |view, _| {
            assert_eq!(view.menu.session_edit_scroll.offset().y, px(0.))
        });
    }
}

fn target() -> Target {
    Target::Device {
        id: "prepared-remote".into(),
        host: RemoteHost::Ssh("prepared.invalid".into()),
    }
}

fn error() -> String {
    "synthetic bounded failure with long words 🙂 ".repeat(30)
}
