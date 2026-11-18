// ABOUTME: Checks terminal wire delivery and accessible chrome prefix actions with real key dispatch.
// ABOUTME: Reuses the framed daemon fixture without requiring a native desktop keyboard.
use super::*;

#[gpui::test]
fn pane_function_and_tab_navigation_keys_reach_the_real_wire(cx: &mut TestAppContext) {
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
    use sheprd_client::protocol::ClientKeyCode;
    for (key, code, modifiers) in [
        ("f6", ClientKeyCode::F(6), 0),
        ("shift-f6", ClientKeyCode::F(6), 1),
        ("f11", ClientKeyCode::F(11), 0),
        ("ctrl-pageup", ClientKeyCode::PageUp, 2),
        ("ctrl-pagedown", ClientKeyCode::PageDown, 2),
    ] {
        ready(&view, cx);
        press(cx, key, false);
        loop {
            match peer.receive() {
                ClientMessage::ClientShellPaneInput { events, .. } => {
                    assert!(
                        matches!(events.as_slice(), [ClientPaneInputEvent::Key {
                        code: actual, modifiers: actual_modifiers, ..
                    }] if actual == &code && actual_modifiers == &modifiers),
                        "{key}: {events:?}"
                    );
                    break;
                }
                ClientMessage::ClientShellEndpointRequest { request, .. } => {
                    panic!("pane key invoked a GUI action: {key}: {request}");
                }
                _ => {}
            }
        }
    }
}

#[gpui::test]
fn explicit_prefix_region_commands_move_focus_through_real_dispatch(cx: &mut TestAppContext) {
    let (view, cx) = cx.add_window_view(fixture_window);
    cx.update(|window, _| window.activate_window());
    cx.update(|window, cx| {
        view.update(cx, |view, cx| {
            let table: toml::Table = "[keys]\nfocus_next_region = 'prefix+f6'\nfocus_previous_region = 'prefix+shift+f6'\n".parse().unwrap();
            let keys = crate::keymap::DaemonKeys::from_table(table["keys"].as_table());
            view.config.keybindings = crate::keymap::Keymap::with_overrides(
                &Default::default(), &Default::default(), &keys,
            ).unwrap();
            window.focus(&view.focus.clone(), cx);
        });
        window.draw(cx).clear(cx);
    });
    press(cx, "ctrl-b", false);
    press(cx, "f6", false);
    // Drawing alone does not deliver platform frame callbacks in headless GPUI.
    crate::sidebar::layout_tests::run_frames(cx, 2, std::time::Duration::ZERO);
    assert_eq!(
        cx.update(|window, cx| view.read(cx).focused_region(window)),
        Some(crate::window::chrome_focus::Region::TitleBar)
    );
    press(cx, "ctrl-b", false);
    press(cx, "shift-f6", false);
    assert_eq!(view.read_with(cx, |view, _| view.chrome_region), None);
}
