// ABOUTME: Verifies the native accessibility sample through the real preview installer.
// ABOUTME: Keeps transcript, draft, and focus owners tied to the shared protocol fixture.
#![allow(clippy::unwrap_used)]
use super::*;
use gpui::{KeyDownEvent, KeyUpEvent};

#[gpui::test]
fn accessibility_preview_installs_the_protocol_fixture_and_preserved_draft(
    cx: &mut gpui::TestAppContext,
) {
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../../../tests/support/native-chat-accessibility.json"
    ))
    .unwrap();
    let expected: Transcript = serde_json::from_value(fixture["transcript"].clone()).unwrap();
    let (view, cx) = cx.add_window_view(crate::sidebar::layout_tests::fixture_window);
    let mut options = conversation_options();
    options.case = "accessibility".into();
    options.narrow = false;
    cx.update(|window, cx| {
        view.update(cx, |view, cx| install(view, window, cx, &options).unwrap())
    });
    view.read_with(cx, |view, cx| {
        let pane = view.chats.pane(PANE).unwrap();
        assert_eq!(pane.session.transcript(), &expected);
        assert!(pane.rail, "the sample must render its plan");
        assert!(pane.text_focus.is_some());
        assert_eq!(
            pane.composer.as_ref().unwrap().read(cx).text(),
            fixture["draft"].as_str().unwrap()
        );
    });
}

#[gpui::test]
fn rendered_permission_keys_feed_the_real_preview_session_probe(cx: &mut gpui::TestAppContext) {
    let (view, cx) = cx.add_window_view(crate::sidebar::layout_tests::fixture_window);
    let mut options = conversation_options();
    options.case = "accessibility".into();
    options.narrow = false;
    cx.update(|window, cx| {
        view.update(cx, |view, cx| {
            install(view, window, cx, &options).unwrap();
            view.prepare_chat_visuals(window, cx);
        });
        window.refresh();
        window.draw(cx).clear(cx);
    });
    let focus = view.read_with(cx, |view, cx| {
        view.chats
            .pane(PANE)
            .unwrap()
            .composer
            .as_ref()
            .unwrap()
            .read(cx)
            .focus
            .clone()
    });
    let mut probe = super::super::accessibility::Probe::new().unwrap();
    for key in ["enter", "space"] {
        cx.update(|window, cx| focus.focus(window, cx));
        let mut reached = false;
        for _ in 0..64 {
            let tab = crate::keymap::parse_keystroke("tab").unwrap();
            cx.simulate_event(KeyDownEvent {
                keystroke: tab.clone(),
                is_held: false,
                prefer_character_input: false,
            });
            cx.simulate_event(KeyUpEvent { keystroke: tab });
            cx.update(|window, cx| {
                window.refresh();
                window.draw(cx).clear(cx);
            });
            if cx.update(|window, _| {
                window
                    .context_stack()
                    .iter()
                    .any(|context| context.contains("ChatPermissionAnswer"))
            }) {
                reached = true;
                break;
            }
        }
        assert!(reached, "Tab must reach the actual permission control");
        let keystroke = crate::keymap::parse_keystroke(key).unwrap();
        cx.simulate_event(KeyDownEvent {
            keystroke: keystroke.clone(),
            is_held: false,
            prefer_character_input: false,
        });
        cx.simulate_event(KeyUpEvent { keystroke });
        cx.update(|window, cx| {
            view.update(cx, |view, cx| probe.observe(view, cx).unwrap());
            window.refresh();
            window.draw(cx).clear(cx);
        });
    }
    assert!(probe.complete());
    view.read_with(cx, |view, _| {
        assert!(
            view.chats
                .pane(PANE)
                .unwrap()
                .session
                .transcript()
                .permissions
                .is_empty()
        )
    });
}
