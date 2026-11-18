// ABOUTME: Keeps Core split panes behind the shared close confirmation.
// ABOUTME: Verifies cancellation, target fencing, and held shortcut events.
use super::*;
use crate::{
    controls::Command,
    core_terminals::{CoreTerminalView, worker::Link},
    menu::Page,
};
use gpui::{AppContext, KeyDownEvent, Keystroke, PlatformInput};

#[cfg(unix)]
mod core_only;

fn split_window(cx: &mut TestAppContext) -> (WindowHandle<SheprdWindow>, TabId) {
    let (key, tab) = group_with_tab(cx);
    let handle = window_on(&key, cx);
    handle
        .update(cx, |view, window, cx| {
            view.poll_core_terminals(window, cx);
            view.place_split(tab, &id("t1"), Axis::Row, id("t2"), window, cx);
            view.place_split(tab, &id("t2"), Axis::Row, id("t3"), window, cx);
            for name in ["t1", "t2", "t3"] {
                let pane = cx.new(|cx| {
                    CoreTerminalView::new(
                        id(name),
                        Link::for_test(),
                        view.core_look(),
                        view.painter.clone(),
                        false,
                        cx,
                    )
                });
                view.core_terminals.views.insert((tab, id(name)), pane);
            }
            view.show_browser_tab(tab, window, cx);
            window.focus(
                &view.core_terminals.views[&(tab, id("t2"))]
                    .read(cx)
                    .focus_handle()
                    .clone(),
                cx,
            );
        })
        .unwrap();
    cx.update(|cx| {
        cx.update_window(handle.into(), |_, window, cx| window.draw(cx).clear(cx))
            .unwrap()
    });
    (handle, tab)
}

fn press(name: &'static str, held: bool) -> PlatformInput {
    PlatformInput::KeyDown(key(name, held))
}

fn key(name: &'static str, held: bool) -> KeyDownEvent {
    KeyDownEvent {
        keystroke: Keystroke::parse(name).unwrap(),
        is_held: held,
        prefer_character_input: false,
    }
}

#[gpui::test]
fn core_close_waits_for_confirmation_and_cancel_preserves_panes(cx: &mut TestAppContext) {
    let (handle, tab) = split_window(cx);
    handle
        .update(cx, |view, window, cx| {
            assert!(view.config.confirm_close_pane);
            view.command(Command::ClosePane, window, cx);
            assert_eq!(view.menu.page, Some(Page::ConfirmClose));
            view.dismiss_menu(window, cx);
            assert!(
                view.core_terminals.views[&(tab, id("t2"))]
                    .read(cx)
                    .focus_handle()
                    .is_focused(window)
            );
        })
        .unwrap();
    assert_eq!(saved_terminals(tab, cx), ["t1", "t2", "t3"]);
}

#[gpui::test]
fn cancel_does_not_focus_a_neighbor_when_the_captured_core_pane_disappears(
    cx: &mut TestAppContext,
) {
    let (handle, tab) = split_window(cx);
    handle
        .update(cx, |view, window, cx| {
            view.command(Command::ClosePane, window, cx);
            assert_eq!(view.menu.page, Some(Page::ConfirmClose));
            assert!(view.remove_core_pane(tab, &id("t2"), window, cx));
            view.dismiss_menu(window, cx);
            assert!(view.focus.is_focused(window));
            for terminal in ["t1", "t3"] {
                assert!(
                    !view.core_terminals.views[&(tab, id(terminal))]
                        .read(cx)
                        .focus_handle()
                        .is_focused(window)
                );
            }
        })
        .unwrap();
    assert_eq!(saved_terminals(tab, cx), ["t1", "t3"]);
}

#[gpui::test]
#[cfg(unix)]
fn a_core_close_cannot_follow_a_replaced_selection(cx: &mut TestAppContext) {
    let (handle, tab) = split_window(cx);
    let _peer = connected(&handle, cx);
    handle
        .update(cx, |view, window, cx| {
            view.command(Command::ClosePane, window, cx);
            assert_eq!(view.menu.page, Some(Page::ConfirmClose));
            view.selection_epoch += 1;
            assert_eq!(view.send_close(window, cx), None);
            assert_eq!(view.menu.page, Some(Page::ConfirmClose));
        })
        .unwrap();
    assert_eq!(saved_terminals(tab, cx), ["t1", "t2", "t3"]);
}

#[gpui::test]
fn close_shortcut_rebinding_preserves_menu_actions_and_other_commands(cx: &mut TestAppContext) {
    use crate::{
        actions::RunCommand,
        keymap::{Binding, DaemonKeys, Keymap},
    };
    let overrides = [
        ("close_pane".to_owned(), Binding::One("cmd-e".into())),
        ("toggle_sidebar".to_owned(), Binding::One("cmd-w".into())),
    ]
    .into();
    let keymap =
        Keymap::with_overrides(&overrides, &Default::default(), &DaemonKeys::default()).unwrap();
    let (handle, tab) = split_window(cx);
    handle
        .update(cx, |view, _, _| view.config.keybindings = keymap.clone())
        .unwrap();
    cx.update(|cx| {
        let mut appearance = crate::app::InitialAppearance::default();
        appearance.config.keybindings = keymap;
        cx.set_global(appearance);
        crate::actions::rebind_keys(cx);
        assert!(
            cx.key_bindings()
                .borrow()
                .bindings_for_action(&RunCommand {
                    command: Command::ClosePane
                })
                .next()
                .is_some()
        );
        cx.update_window(handle.into(), |_, window, cx| {
            window.draw(cx).clear(cx);
            window.dispatch_event(press("cmd-w", false), cx);
            window.dispatch_event(press("cmd-e", true), cx);
        })
        .unwrap();
    });
    handle
        .update(cx, |view, _, _| {
            assert!(!view.sidebar_visible);
            assert!(view.menu.page.is_none());
        })
        .unwrap();
    cx.update(|cx| {
        cx.update_window(handle.into(), |_, window, cx| {
            window.dispatch_event(press("cmd-e", false), cx);
        })
        .unwrap()
    });
    handle
        .update(cx, |view, window, cx| {
            assert_eq!(view.menu.page, Some(Page::ConfirmClose));
            view.dismiss_menu(window, cx);
        })
        .unwrap();
    cx.update(|cx| {
        cx.update_window(handle.into(), |_, window, cx| {
            window.draw(cx).clear(cx);
            window.dispatch_action(
                Box::new(RunCommand {
                    command: Command::ClosePane,
                }),
                cx,
            );
        })
        .unwrap()
    });
    cx.run_until_parked();
    handle
        .update(cx, |view, _, _| {
            assert_eq!(view.menu.page, Some(Page::ConfirmClose))
        })
        .unwrap();
    assert_eq!(saved_terminals(tab, cx), ["t1", "t2", "t3"]);
}

#[gpui::test]
#[cfg(unix)]
fn core_close_confirms_only_the_captured_pane(cx: &mut TestAppContext) {
    let (handle, tab) = split_window(cx);
    let mut peer = connected(&handle, cx);
    handle
        .update(cx, |view, window, cx| {
            view.command(Command::ClosePane, window, cx);
            assert_eq!(view.menu.page, Some(Page::ConfirmClose));
            view.close_confirmation_key(&key("right", false), window, cx);
            view.close_confirmation_key(&key("enter", true), window, cx);
            assert_eq!(view.menu.page, Some(Page::ConfirmClose));
            view.close_confirmation_key(&key("enter", false), window, cx);
            assert!(view.menu.page.is_none());
        })
        .unwrap();
    assert_eq!(saved_terminals(tab, cx), ["t1", "t3"]);
    receive_kill(&mut peer, "t2");
}

#[gpui::test]
#[cfg(unix)]
fn held_close_shortcut_cannot_walk_neighboring_core_panes(cx: &mut TestAppContext) {
    cx.update(crate::bind_keys);
    let (handle, tab) = split_window(cx);
    let mut peer = connected(&handle, cx);
    handle
        .update(cx, |view, _, _| view.config.confirm_close_pane = false)
        .unwrap();
    let shortcut = if crate::controls::CTRL_SHORTCUTS {
        "ctrl-shift-w"
    } else {
        "cmd-w"
    };
    cx.update(|cx| {
        cx.update_window(handle.into(), |_, window, cx| {
            window.dispatch_event(press(shortcut, false), cx);
            window.draw(cx).clear(cx);
            for _ in 0..3 {
                window.dispatch_event(press(shortcut, true), cx);
                window.draw(cx).clear(cx);
            }
        })
        .unwrap()
    });
    assert_eq!(saved_terminals(tab, cx), ["t1", "t3"]);
    receive_kill(&mut peer, "t2");
}

#[cfg(unix)]
fn connected(
    handle: &WindowHandle<SheprdWindow>,
    cx: &mut TestAppContext,
) -> std::os::unix::net::UnixStream {
    let (backend, peer) = core_terminals::connect_backend_for_test("hub-a");
    handle
        .update(cx, |view, _, _| {
            view.core_terminals.hub.set_backend_for_test(backend)
        })
        .unwrap();
    peer
}

#[cfg(unix)]
fn receive_kill(peer: &mut std::os::unix::net::UnixStream, terminal: &str) {
    use sheprd_core_client::frame::{self, Kind};
    use std::io::{Read, Write};
    loop {
        let mut header = [0; frame::HEADER];
        peer.read_exact(&mut header).unwrap();
        let len = u32::from_le_bytes(header[5..9].try_into().unwrap()) as usize;
        assert!(len <= frame::MAX_PAYLOAD);
        let mut payload = vec![0; len];
        peer.read_exact(&mut payload).unwrap();
        if header[..5] != [0; 5] {
            continue;
        }
        let request: serde_json::Value = serde_json::from_slice(&payload).unwrap();
        assert_eq!(request["method"], "terminal.kill");
        assert_eq!(request["params"]["terminal"], terminal);
        let reply = serde_json::json!({"jsonrpc":"2.0", "id":request["id"], "result":{"address":terminal,"state":"finished"}});
        peer.write_all(
            &frame::encode(0, Kind::Data, &serde_json::to_vec(&reply).unwrap()).unwrap(),
        )
        .unwrap();
        break;
    }
}
