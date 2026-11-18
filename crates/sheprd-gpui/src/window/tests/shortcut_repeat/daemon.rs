// ABOUTME: Checks held shortcuts against real framed daemon connections.
// ABOUTME: Uses ordered input markers to prove no repeated requests reached the wire.
use super::*;
use crate::window::MockPeer;
use gpui::{Entity, TestAppContext, VisualTestContext};
use sheprd_client::protocol::{
    ClientMessage, ClientPaneInputEvent, ClientShellCommand, ClientShellCommandAction,
};

mod platform_shortcuts;

fn marker(peer: &mut MockPeer, view: &Entity<SheprdWindow>, cx: &VisualTestContext) {
    view.read_with(cx, |view, _| {
        peer.client.handle.send_input(
            &view.live.snapshot.as_ref().unwrap().boot_id,
            "w1:p1",
            [ClientPaneInputEvent::TextCommit("shortcut-marker".into())],
        )
    })
    .unwrap();
    loop {
        match peer.receive() {
            ClientMessage::ClientShellPaneInput { events, .. }
                if events == [ClientPaneInputEvent::TextCommit("shortcut-marker".into())] =>
            {
                return;
            }
            ClientMessage::ClientShellEndpointRequest { request, .. } => {
                panic!("held shortcut sent another request: {request}")
            }
            _ => {}
        }
    }
}

fn ready(view: &Entity<SheprdWindow>, cx: &mut VisualTestContext) {
    cx.update(|window, cx| {
        window.draw(cx).clear(cx);
        view.update(cx, |view, cx| {
            // Acknowledge the rendered viewport before dispatch can draw another frame.
            let size = view.options.surface_size;
            let surface = Arc::make_mut(view.live.surface.as_mut().unwrap());
            surface.frame.width = size.cols;
            surface.frame.height = size.rows;
            for pane in &mut surface.panes {
                pane.rect.width = size.cols;
                pane.rect.height = size.rows;
                pane.inner_rect = pane.rect;
            }
            assert!(view.endpoints[0].surface_requested());
            assert!(
                view.input_ready(),
                "the fixture must accept real terminal actions"
            );
            assert!(view.menu.page.is_none());
            assert!(view.flush_pending_input(cx));
        });
    });
}

#[gpui::test]
fn tab_split_and_custom_shortcuts_ignore_repeats_after_daemon_acknowledgement(
    cx: &mut TestAppContext,
) {
    for command in [Some(Command::Tab), Some(Command::SplitRight), None] {
        cx.update(crate::actions::bind_keys);
        let (view, cx) = cx.add_window_view(fixture_window);
        let mut peer = MockPeer::advertising(&["tab.create", "pane.split", "command.invoke"]);
        let key = cx.update(|window, cx| {
            view.update(cx, |view, cx| {
                peer.prepare_local_tab(view, cx);
                view.endpoints[0].attach_shown(peer.client.handle.clone());
                view.live.supports_surface = false;
                view.settings.shared = Some(
                    crate::herdr_settings::Settings::parse_text(
                        "[ui]\nprompt_new_tab_name = false\n",
                    )
                    .unwrap(),
                );
                let key = match command {
                    Some(command) => view.keymap().primary(command).to_owned(),
                    None => {
                        Arc::make_mut(view.live.snapshot.as_mut().unwrap()).commands =
                            vec![ClientShellCommand {
                                command_id: "custom.repeat".into(),
                                description: None,
                                action: ClientShellCommandAction::Shell,
                                binding_label: "alt+f12".into(),
                                binding_labels: vec!["alt+f12".into()],
                            }];
                        "alt-f12".into()
                    }
                };
                view.endpoints[0].live = view.live.clone();
                window.focus(&view.focus.clone(), cx);
                key
            })
        });
        cx.update(|window, cx| window.draw(cx).clear(cx));
        ready(&view, cx);
        assert!(
            press(cx, &key, false),
            "shortcut was not handled: {command:?} {key}"
        );
        view.read_with(cx, |view, _| {
            assert!(
                view.notices.is_none(),
                "{command:?}: {:?}",
                view.notices.text()
            )
        });
        let request = peer.request();
        let method = match command {
            Some(Command::Tab) => "tab.create",
            Some(_) => "pane.split",
            None => "command.invoke",
        };
        assert_eq!(request["method"], method);
        let boot = view.read_with(cx, |view, _| {
            view.live.snapshot.as_ref().unwrap().boot_id.clone()
        });
        peer.respond(
            &boot,
            request["id"].as_str().unwrap(),
            &serde_json::json!({"id":request["id"], "result":{}}),
        );
        ready(&view, cx);
        press(cx, &key, true);
        marker(&mut peer, &view, cx);
        ready(&view, cx);
        press(cx, &key, false);
        assert_eq!(peer.request()["method"], method);
    }
}

#[gpui::test]
fn ordinary_terminal_keys_keep_their_repeat_events(cx: &mut TestAppContext) {
    let (view, cx) = cx.add_window_view(fixture_window);
    let mut peer = MockPeer::new();
    cx.update(|window, cx| {
        view.update(cx, |view, cx| {
            peer.prepare_local_tab(view, cx);
            view.endpoints[0].attach_shown(peer.client.handle.clone());
        });
        window.focus(&view.read(cx).focus.clone(), cx);
        window.draw(cx).clear(cx);
    });
    ready(&view, cx);
    press(cx, "left", false);
    ready(&view, cx);
    press(cx, "left", true);
    let mut inputs = Vec::new();
    while inputs.len() < 2 {
        if let ClientMessage::ClientShellPaneInput { events, .. } = peer.receive() {
            inputs.extend(events);
        }
    }
    assert!(inputs.iter().all(|input| matches!(
        input,
        ClientPaneInputEvent::Key {
            code: sheprd_client::protocol::ClientKeyCode::Left,
            ..
        }
    )));
}

#[gpui::test]
fn navigation_and_resize_commands_keep_repeating(cx: &mut TestAppContext) {
    for command in [Command::FocusRight, Command::ResizeRight] {
        let (view, cx) = cx.add_window_view(fixture_window);
        let mut peer = MockPeer::advertising(&[
            sheprd_client::Method::PaneFocusDirection.as_str(),
            "pane.resize",
        ]);
        cx.update(|window, cx| {
            view.update(cx, |view, cx| {
                peer.prepare_local_tab(view, cx);
                view.endpoints[0].attach_shown(peer.client.handle.clone());
                view.live.supports_surface = false;
                let name = match command {
                    Command::FocusRight => "focus_right",
                    _ => "resize_pane_right",
                };
                view.config.keybindings = crate::keymap::Keymap::with_overrides(
                    &[(name.into(), crate::keymap::Binding::One("alt-f11".into()))].into(),
                    &Default::default(),
                    &crate::keymap::DaemonKeys::default(),
                )
                .unwrap();
                view.endpoints[0].live = view.live.clone();
                window.focus(&view.focus.clone(), cx);
            });
            window.draw(cx).clear(cx);
        });
        for held in [false, true] {
            ready(&view, cx);
            press(cx, "alt-f11", held);
            let request = peer.request();
            assert_eq!(
                request["method"],
                if command == Command::FocusRight {
                    sheprd_client::Method::PaneFocusDirection.as_str()
                } else {
                    "pane.resize"
                }
            );
            let boot = view.read_with(cx, |view, _| {
                view.live.snapshot.as_ref().unwrap().boot_id.clone()
            });
            peer.respond(
                &boot,
                request["id"].as_str().unwrap(),
                &serde_json::json!({"id":request["id"], "result":{}}),
            );
        }
    }
}
