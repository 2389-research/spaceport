// ABOUTME: Checks custom-command precedence against real Spaceport prefix defaults.
// ABOUTME: Preserves explicit settings and Herdr-native ownership through matching and wire dispatch.
use super::*;
use crate::keymap::{Binding, Keymap, custom::Reach};
#[cfg(feature = "integration-test")]
use gpui::{
    AppContext as _, Context, Entity, InteractiveElement as _, IntoElement, KeyDownEvent,
    PlatformInput, Render, TestAppContext, Window,
};
#[cfg(feature = "integration-test")]
use sheprd_client::protocol::{ClientMessage, ClientPaneInputEvent};
use sheprd_client::protocol::{ClientShellCommand, ClientShellCommandAction};

fn custom(label: &str) -> ClientShellCommand {
    ClientShellCommand {
        command_id: "custom.existing".into(),
        description: None,
        action: ClientShellCommandAction::Shell,
        binding_label: label.into(),
        binding_labels: vec![label.into()],
    }
}

fn resolved(text: &str) -> Keymap {
    let table: toml::Table = text.parse().unwrap();
    let keys = DaemonKeys::from_table_for_platform(
        table.get("keys").and_then(toml::Value::as_table),
        true,
    );
    Keymap::with_overrides(&Default::default(), &Default::default(), &keys).unwrap()
}

#[cfg(feature = "integration-test")]
struct ShortcutWindow(Entity<crate::window::SheprdWindow>);

#[cfg(feature = "integration-test")]
impl Render for ShortcutWindow {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.0.update(cx, |view, cx| {
            crate::window::SheprdWindow::shortcut_scope(cx).track_focus(&view.focus)
        })
    }
}

#[test]
fn spaceport_defaults_yield_but_explicit_view_and_herdr_chords_keep_ownership() {
    for (name, chord) in [
        ("focus_next_region", "f6"),
        ("focus_previous_region", "shift+f6"),
        ("toggle_full_screen", "f11"),
    ] {
        let command = custom(&format!("prefix+{chord}"));
        let defaults = resolved("");
        assert_eq!(
            defaults.custom_bindings([&command])[0][0].reach,
            Reach::Runs,
            "{name}"
        );
        let explicit = resolved(&format!("[keys]\n{name} = 'prefix+{chord}'\n"));
        assert_eq!(
            explicit.custom_bindings([&command])[0][0].reach,
            Reach::Shadowed,
            "{name}"
        );
    }
    for chord in ["?", "shift+d"] {
        let command = custom(&format!("prefix+{chord}"));
        assert_eq!(
            resolved("").custom_bindings([&command])[0][0].reach,
            Reach::Shadowed,
            "{chord}"
        );
    }
    let keys = DaemonKeys::from_table_for_platform(None, true);
    let gui = Keymap::with_overrides(
        &[("focus_next_region".into(), Binding::One("alt-f6".into()))].into(),
        &Default::default(),
        &keys,
    )
    .unwrap();
    assert_eq!(
        gui.custom_bindings([&custom("alt+f6")])[0][0].reach,
        Reach::Shadowed
    );
    assert_eq!(
        gui.custom_bindings([&custom("prefix+f6")])[0][0].reach,
        Reach::Runs
    );
}

#[cfg(feature = "integration-test")]
#[gpui::test]
fn existing_custom_chrome_chords_reach_the_real_wire(cx: &mut TestAppContext) {
    for (chord, name) in [
        ("f6", "focus_next_region"),
        ("shift-f6", "focus_previous_region"),
        ("f11", "toggle_full_screen"),
    ] {
        for explicit in [false, true] {
            let (fixture, cx) = cx.add_window_view(|window, cx| {
                ShortcutWindow(
                    cx.new(|cx| crate::sidebar::layout_tests::fixture_window(window, cx)),
                )
            });
            let view = fixture.update(cx, |fixture, _| fixture.0.clone());
            let mut peer = crate::window::MockPeer::advertising(&["command.invoke"]);
            cx.update(|window, cx| {
                view.update(cx, |view, cx| {
                    peer.prepare_local_tab(view, cx);
                    view.endpoints[0].attach_shown(peer.client.handle.clone());
                    let text = if explicit {
                        format!("[keys]\n{name} = 'prefix+{}'\n", chord.replace('-', "+"))
                    } else {
                        String::new()
                    };
                    view.keymap = resolved(&text);
                    let command = custom(&format!("prefix+{}", chord.replace('-', "+")));
                    assert_eq!(
                        view.keymap().custom_bindings([&command])[0][0].reach,
                        if explicit {
                            Reach::Shadowed
                        } else {
                            Reach::Runs
                        }
                    );
                    std::sync::Arc::make_mut(view.live.snapshot.as_mut().unwrap()).commands =
                        vec![command];
                    view.endpoints[0].live = view.live.clone();
                    window.focus(&view.focus.clone(), cx);
                });
                window.draw(cx).clear(cx);
            });
            for key in ["ctrl-b", chord] {
                cx.update(|window, cx| {
                    window.dispatch_event(
                        PlatformInput::KeyDown(KeyDownEvent {
                            keystroke: parsed(key),
                            is_held: false,
                            prefer_character_input: false,
                        }),
                        cx,
                    );
                });
                cx.run_until_parked();
            }
            if !explicit {
                let request = peer.request();
                assert_eq!(request["method"], "command.invoke");
                assert_eq!(request["params"]["command_id"], "custom.existing");
                assert_eq!(view.read_with(cx, |view, _| view.chrome_region), None);
                // Correlation is required by the actual worker even when the fixture is about to drop.
                let boot = view.read_with(cx, |view, _| {
                    view.live.snapshot.as_ref().unwrap().boot_id.clone()
                });
                peer.respond(
                    &boot,
                    request["id"].as_str().unwrap(),
                    &serde_json::json!({"id":request["id"], "result":{}}),
                );
            } else if chord == "f6" {
                assert_eq!(
                    view.read_with(cx, |view, _| view.chrome_region),
                    Some(crate::window::chrome_focus::Region::TitleBar)
                );
            }
            let marker = ClientPaneInputEvent::TextCommit("prefix-precedence-marker".into());
            view.read_with(cx, |view, _| {
                peer.client.handle.send_input(
                    &view.live.snapshot.as_ref().unwrap().boot_id,
                    "w1:p1",
                    [marker.clone()],
                )
            })
            .unwrap();
            loop {
                match peer.receive() {
                    ClientMessage::ClientShellPaneInput { events, .. }
                        if events == [marker.clone()] =>
                    {
                        break;
                    }
                    ClientMessage::ClientShellEndpointRequest { request, .. } => {
                        panic!("unexpected custom dispatch after chord: {explicit}: {request}")
                    }
                    _ => {}
                }
            }
        }
    }
}
