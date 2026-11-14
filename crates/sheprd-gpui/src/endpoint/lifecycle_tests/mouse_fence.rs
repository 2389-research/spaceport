// ABOUTME: Pins which connection changes a held terminal mouse gesture survives.
// ABOUTME: A gesture compares the window's own record of the selection, not the endpoint's.

use super::*;
use crate::{
    endpoint::fence::changes::{CHANGES, apply},
    terminal::InputTarget,
};

const AFTER: &str = "after";

fn press(view: &mut SheprdWindow, window: &mut gpui::Window, cx: &mut Context<SheprdWindow>) {
    assert!(view.terminal_mouse_down(
        &MouseDownEvent {
            position: mouse_position(view, 3.5, 4.5),
            button: MouseButton::Left,
            ..Default::default()
        },
        window,
        cx,
    ));
}

/// Sends a marker on the gesture's own transport, so what the gesture sent
/// before it can be told from nothing at all.
fn mark(view: &SheprdWindow, boot: &str) {
    ConnectionBridge::send_input(
        view.endpoints[1].connection.handle.as_ref().unwrap(),
        boot,
        &InputTarget::Pane("w1:p1".into()),
        ClientPaneInputEvent::TextCommit(AFTER.into()),
    )
    .unwrap();
}

/// Whether the application was sent the button's release before the marker.
fn released(server: &mut Server) -> bool {
    let pane_input = |events| ClientMessage::ClientShellPaneInput {
        pane_id: "w1:p1".into(),
        events,
    };
    let down = mouse_event(ClientMouseKind::Down(ClientMouseButton::Left), 2, 3);
    let up = mouse_event(ClientMouseKind::Up(ClientMouseButton::Left), 2, 3);
    let marker = pane_input(vec![ClientPaneInputEvent::TextCommit(AFTER.into())]);
    assert_eq!(server.receive(), pane_input(vec![down]));
    let next = server.receive();
    if next == marker {
        return false;
    }
    assert_eq!(next, pane_input(vec![up]));
    assert_eq!(server.receive(), marker);
    true
}

/// What each way of ending or continuing a held gesture does after `change`:
/// a release at the pressed cell, a cancel, and a drag during a frame gap.
#[gpui::test]
fn a_held_gesture_follows_the_windows_record_of_its_selection(cx: &mut gpui::TestAppContext) {
    let (fixture, cx) = cx.add_window_view(|window, cx| {
        Fixture(cx.new(|cx| crate::sidebar::layout_tests::fixture_window(window, cx)))
    });
    let view = fixture.update(cx, |fixture, _| fixture.0.clone());
    let seen = CHANGES.map(|change| {
        let mut row = [false; 3];
        for (way, sent) in row.iter_mut().enumerate() {
            let (endpoint, mut server) = connected_endpoint("ssh:mouse");
            let waits = cx.update(|window, cx| {
                view.update(cx, |view, cx| {
                    prepare_mouse(view, endpoint, cx);
                    let boot = view.live.snapshot.as_ref().unwrap().boot_id.clone();
                    press(view, window, cx);
                    if way == 2 {
                        Arc::make_mut(view.live.surface.as_mut().unwrap()).projection_revision -= 1;
                    }
                    apply(view, 1, change, cx);
                    let position = mouse_position(view, 3.5, 4.5);
                    match way {
                        0 => {
                            view.terminal_mouse_up(
                                &MouseUpEvent {
                                    position,
                                    button: MouseButton::Left,
                                    ..Default::default()
                                },
                                cx,
                            );
                        }
                        1 => view.cancel_terminal_mouse(cx),
                        _ => {
                            view.terminal_mouse_move(
                                &MouseMoveEvent {
                                    position: mouse_position(view, 5.5, 6.5),
                                    pressed_button: Some(MouseButton::Left),
                                    ..Default::default()
                                },
                                cx,
                            );
                        }
                    }
                    let waits = view.terminal_mouse.is_some();
                    mark(view, &boot);
                    waits
                })
            });
            // A drag in a gap either keeps waiting, or ends with a release.
            *sent = match way {
                2 => waits && !released(&mut server),
                _ => released(&mut server),
            };
        }
        row
    });
    // Columns: release sent, cancel sent, drag waits through a gap.
    // Rows: Reselect, SwitchAndReturn, ReconnectUnseen, Reconnect, Reboot,
    // Removed. A gesture has no endpoint id term and reads the window's
    // `selected_generation`, so it outlives a replaced connection the window
    // has not reset onto, and a slot that names another endpoint.
    assert_eq!(
        seen,
        [
            [true; 3], [false; 3], [true; 3], [false; 3], [false; 3], [true; 3],
        ]
    );
}
