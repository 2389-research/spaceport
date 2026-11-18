// ABOUTME: Checks frozen session answers against a replacement invalid Create form.
// ABOUTME: Contains leaked callbacks before any process or daemon mutation can run.
use super::*;
use crate::menu::sessions::{Edit, Row, Target};
use gpui::Modifiers;

#[gpui::test]
fn frozen_picker_rows_chips_and_card_cannot_change_replacement_state(cx: &mut TestAppContext) {
    let (root, cx) = cx.add_window_view(|window, cx| Preview {
        source: cx.new(|cx| {
            let mut view = fixture(window, cx);
            let mut remote = crate::endpoint::Endpoint::new(
                "removed-remote".into(),
                "Prepared remote".into(),
                ConnectTarget::Ssh {
                    target: "unreachable.invalid".into(),
                    session: "fixture-remote-session".into(),
                },
                true,
            );
            remote.live.status = crate::state::ConnectionStatus::Connected;
            view.menu.unfolded_device = Some(remote.id.clone());
            view.sessions.devices.answers.insert(
                remote.id.clone(),
                crate::sessions::DeviceScan::Sessions(vec![sheprd_client::RemoteSession {
                    name: "fixture-remote-session".into(),
                    running: true,
                }]),
            );
            view.endpoints.push(remote);
            view
        }),
        paint: None,
        modal: false,
    });
    cx.update(|window, cx| window.draw(cx).clear(cx));
    cx.update(|window, cx| {
        root.update(cx, |root, cx| {
            root.paint = root.source.update(cx, |view, cx| {
                let paint = view.devices_paint(cx).unwrap();
                view.dismiss_menu(window, cx);
                // Remove the exact remote identity before exercising any stale session callback.
                view.endpoints.truncate(1);
                view.sessions.devices.answers.clear();
                view.menu.page = Some(Page::Devices);
                view.menu.selected = None;
                // A leaked local chip may only fold this prepared group; it must never start a scan.
                view.menu.unfolded_device = Some(view.endpoints[0].id.clone());
                view.menu.session_edit = Some(Edit::Create {
                    input: cx.new(|cx| {
                        let mut input = SearchInput::new(cx);
                        input.set_text_selected("invalid/name", cx);
                        input
                    }),
                    target: Target::Local,
                });
                window.focus(&view.menu.focus, cx);
                Some(paint)
            });
        });
        window.refresh();
        window.draw(cx).clear(cx);
    });
    for selector in [
        "sessions-delete-0",
        "sessions-row-0",
        "sessions-row-1",
        "device-sessions-1",
        "device-row-1",
        "device-row-0",
        "device-activity",
    ] {
        let bounds = cx.debug_bounds(selector).unwrap();
        cx.simulate_mouse_move(bounds.center(), None, Modifiers::default());
        cx.simulate_click(bounds.center(), Modifiers::default());
        root.read_with(cx, |root, cx| {
            let view = root.source.read(cx);
            assert_eq!(view.menu.page, Some(Page::Devices), "{selector}");
            assert!(
                matches!(view.menu.session_edit, Some(Edit::Create { .. })),
                "{selector}"
            );
            assert_eq!(view.menu.selected, None, "{selector}");
            assert_eq!(
                view.menu.unfolded_device.as_ref(),
                Some(&view.endpoints[0].id),
                "{selector}"
            );
            assert!(view.menu.error.is_none(), "{selector}");
            assert!(view.sessions.mutation.is_none(), "{selector}");
            assert!(matches!(
                view.endpoints[0].connection.target,
                ConnectTarget::Socket(_)
            ));
        });
    }
}

#[gpui::test]
fn frozen_form_buttons_cannot_submit_or_cancel_replacement_input(cx: &mut TestAppContext) {
    for deleting in [false, true] {
        let (root, cx) = cx.add_window_view(|window, cx| {
            let source = cx.new(|cx| fixture(window, cx));
            let paint = source.update(cx, |view, cx| {
                view.menu.session_edit = if deleting {
                    Some(Edit::Delete {
                        row: Row::Local("fixture-session-0".into()),
                        target: Target::Local,
                    })
                } else {
                    Some(Edit::Create {
                        input: cx.new(SearchInput::new),
                        target: Target::Local,
                    })
                };
                let paint = view.devices_paint(cx).unwrap();
                view.dismiss_menu(window, cx);
                // A regressed Submit must fail validation before it can create or dial anything.
                view.menu.page = Some(Page::Devices);
                view.menu.session_edit = Some(Edit::Create {
                    input: cx.new(|cx| {
                        let mut input = SearchInput::new(cx);
                        input.set_text_selected("invalid/name", cx);
                        input
                    }),
                    target: Target::Local,
                });
                window.focus(&view.menu.focus, cx);
                paint
            });
            Preview {
                source,
                paint: Some(paint),
                modal: true,
            }
        });
        cx.update(|window, cx| window.draw(cx).clear(cx));
        let identity = root.read_with(cx, |root, cx| {
            match &root.source.read(cx).menu.session_edit {
                Some(Edit::Create { input, .. }) => input.entity_id(),
                _ => panic!("replacement Create exists"),
            }
        });
        for selector in ["session-edit-submit", "session-edit-cancel"] {
            let bounds = cx.debug_bounds(selector).unwrap();
            cx.simulate_click(bounds.center(), Modifiers::default());
        }
        cx.update(|window, cx| {
            root.read_with(cx, |root, cx| {
                let view = root.source.read(cx);
                assert_eq!(view.menu.page, Some(Page::Devices));
                let Some(Edit::Create {
                    input,
                    target: Target::Local,
                }) = &view.menu.session_edit
                else {
                    panic!("frozen controls changed replacement form");
                };
                assert_eq!(input.entity_id(), identity);
                assert_eq!(input.read(cx).text(), "invalid/name");
                assert!(view.menu.error.is_none());
                assert!(view.menu.focus.is_focused(window));
                assert!(view.sessions.mutation.is_none());
                assert!(matches!(
                    view.endpoints[0].connection.target,
                    ConnectTarget::Socket(_)
                ));
            })
        });
    }
}
