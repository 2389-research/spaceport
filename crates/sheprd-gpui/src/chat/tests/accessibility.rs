// ABOUTME: Exercises chat permission answers through real keyboard events and frames.
// ABOUTME: Preserves drafts and requires a deliberate activation of the current permission.
#![allow(clippy::unwrap_used)]
use super::{
    overlay::{attach_with, open, poll},
    *,
};
use crate::SheprdWindow;
use gpui::{
    Element, EntityInputHandler, IntoElement, KeyDownEvent, KeyUpEvent, Modifiers, MouseButton,
    MouseDownEvent, MouseUpEvent, Render, Styled, TestAppContext, accesskit,
};
use sheprd_chat_protocol::{Command, Permission, PermissionId, PermissionKind, PermissionOption};
use sheprd_client::ConnectTarget;

fn permission(id: u64) -> Permission {
    Permission {
        call_id: None,
        id: PermissionId(id),
        options: vec![PermissionOption {
            id: "allow".into(),
            kind: PermissionKind::AllowOnce,
            name: "Allow once".into(),
        }],
        title: "Read the fixture file?".into(),
    }
}

fn waiting(id: u64) -> ServerMessage {
    let mut transcript = Transcript::new("Fixture agent");
    transcript.permissions = vec![permission(id)];
    ServerMessage::Snapshot {
        cursor: cursor(1, 0),
        transcript: Box::new(transcript),
    }
}

fn focus_answer(
    view: &gpui::Entity<SheprdWindow>,
    program: &Program,
    cx: &mut gpui::VisualTestContext,
) {
    focus_control(view, program, "ChatPermissionAnswer", cx);
}

fn focus_control(
    view: &gpui::Entity<SheprdWindow>,
    program: &Program,
    context: &str,
    cx: &mut gpui::VisualTestContext,
) {
    for _ in 0..20 {
        cx.simulate_keystrokes("tab");
        poll(view, program, cx);
        if cx.update(|window, _| {
            window
                .context_stack()
                .iter()
                .any(|key| key.contains(context))
        }) {
            return;
        }
    }
    panic!("Tab could not reach {context}");
}

#[gpui::test]
fn the_actual_composer_field_has_a_name(cx: &mut TestAppContext) {
    struct Probe {
        input: gpui::Entity<crate::multiline_input::MultilineInput>,
        output: std::rc::Rc<std::cell::RefCell<Option<accesskit::Node>>>,
    }
    impl Render for Probe {
        fn render(
            &mut self,
            _: &mut gpui::Window,
            _: &mut gpui::Context<Self>,
        ) -> impl IntoElement {
            let input = self.input.clone();
            let output = self.output.clone();
            gpui::canvas(
                move |_, window, cx| {
                    input.update(cx, |input, cx| {
                        let element = input.render(window, cx).into_element();
                        let mut node = accesskit::Node::new(element.a11y_role().unwrap());
                        element.write_a11y_info(&mut node);
                        *output.borrow_mut() = Some(node);
                    });
                },
                |_, _, _, _| {},
            )
            .size_full()
        }
    }
    let (view, _daemon, cx) = open(cx, ConnectTarget::Local);
    let program = Program::new();
    let _stream = attach_with(&view, &program, waiting(41), cx);
    let input = view.read_with(cx, |view, _| {
        view.chats.pane("w1:p1").unwrap().composer.clone().unwrap()
    });
    let output = std::rc::Rc::new(std::cell::RefCell::new(None));
    cx.update(|window, cx| {
        window.replace_root(cx, |_, _| Probe {
            input,
            output: output.clone(),
        });
        window.refresh();
        window.draw(cx).clear(cx);
    });
    assert_eq!(
        output.borrow().as_ref().unwrap().label(),
        Some("Message the agent")
    );
}

#[gpui::test]
fn tab_leaves_composition_with_its_original_editor(cx: &mut TestAppContext) {
    let (view, _daemon, cx) = open(cx, ConnectTarget::Local);
    let program = Program::new();
    let _stream = attach_with(&view, &program, waiting(41), cx);
    let input = view.read_with(cx, |view, _| {
        view.chats.pane("w1:p1").unwrap().composer.clone().unwrap()
    });
    input.update_in(cx, |input, window, cx| {
        input.replace_and_mark_text_in_range(None, "候", Some(0..1), window, cx);
    });
    cx.simulate_event(KeyDownEvent {
        keystroke: crate::keymap::parse_keystroke("tab").unwrap(),
        is_held: false,
        prefer_character_input: false,
    });
    input.update_in(cx, |input, window, _| {
        assert!(input.focus.is_focused(window));
        assert!(input.is_composing());
        assert_eq!(input.text(), "候");
    });
}

#[gpui::test]
fn a_tool_disclosure_opens_from_its_keyboard_control(cx: &mut TestAppContext) {
    let (view, _daemon, cx) = open(cx, ConnectTarget::Local);
    let program = Program::new();
    let tool = sheprd_chat_protocol::Tool {
        call_id: "fixture-tool".into(),
        content: vec![sheprd_chat_protocol::ToolContent::Text {
            text: "Complete fixture output".into(),
        }],
        kind: sheprd_chat_protocol::ToolKind::Execute,
        locations: vec![],
        parent: None,
        started_ms: None,
        status: sheprd_chat_protocol::ToolStatus::Completed,
        title: "Read fixture results".into(),
    };
    let message = snapshot(
        cursor(1, 0),
        vec![Entry {
            id: EntryId(1),
            kind: EntryKind::Tool(tool),
        }],
    );
    let _stream = attach_with(&view, &program, message, cx);
    focus_control(&view, &program, "ChatDisclosure", cx);
    let enter = crate::keymap::parse_keystroke("enter").unwrap();
    cx.simulate_event(KeyDownEvent {
        keystroke: enter.clone(),
        is_held: false,
        prefer_character_input: false,
    });
    cx.simulate_event(KeyUpEvent { keystroke: enter });
    view.read_with(cx, |view, _| {
        let pane = view.chats.pane("w1:p1").unwrap();
        assert!(pane.expanded.contains(&EntryId(1)));
        let projection = pane.prepared.text().entry(EntryId(1)).unwrap();
        assert!(
            projection
                .texts()
                .any(|text| text.contains("Complete fixture output"))
        );
    });
}

#[gpui::test]
fn pointer_send_preserves_pending_paste_refusal_and_composer_focus(cx: &mut TestAppContext) {
    let (view, _daemon, cx) = open(cx, ConnectTarget::Local);
    let program = Program::new();
    let mut stream = attach_with(&view, &program, composer::idle(), cx);
    cx.update(|window, _| window.activate_window());
    cx.simulate_input("keep my draft");
    poll(&view, &program, cx);
    let (_id, input, sender) = clipboard::pending(&view, cx);
    let position = cx.debug_bounds("chat-send").unwrap().center();
    cx.simulate_event(MouseDownEvent {
        position,
        modifiers: Modifiers::default(),
        button: MouseButton::Left,
        click_count: 1,
        first_mouse: false,
    });
    poll(&view, &program, cx);
    cx.simulate_event(MouseUpEvent {
        position,
        modifiers: Modifiers::default(),
        button: MouseButton::Left,
        click_count: 1,
    });
    poll(&view, &program, cx);
    view.update(cx, |view, cx| view.chat_send("w1:p1", Command::Cancel, cx));
    assert_eq!(command(&mut stream), Command::Cancel);
    assert!(input.read_with(cx, |input, _| input.is_pasting()));
    assert!(cx.update(|window, cx| input.read(cx).focus.is_focused(window)));
    let _ = sender.send(Ok(None));
    cx.run_until_parked();
}

fn command(stream: &mut Peer) -> Command {
    match read(stream).unwrap() {
        ClientMessage::Command { command, .. } => command,
        other => panic!("expected command, got {other:?}"),
    }
}

#[gpui::test]
fn tab_and_enter_answer_without_submitting_or_editing_the_draft(cx: &mut TestAppContext) {
    let (view, _daemon, cx) = open(cx, ConnectTarget::Local);
    let program = Program::new();
    let mut stream = attach_with(&view, &program, waiting(41), cx);
    cx.simulate_input("keep my draft");
    focus_answer(&view, &program, cx);
    let enter = crate::keymap::parse_keystroke("enter").unwrap();
    cx.simulate_event(KeyDownEvent {
        keystroke: enter.clone(),
        is_held: false,
        prefer_character_input: false,
    });
    // A later command proves key-down alone did not answer or submit the draft.
    view.update(cx, |view, cx| view.chat_send("w1:p1", Command::Cancel, cx));
    assert_eq!(command(&mut stream), Command::Cancel);
    cx.simulate_event(KeyUpEvent { keystroke: enter });
    assert_eq!(
        command(&mut stream),
        Command::Answer {
            permission: PermissionId(41),
            option: "allow".into(),
        }
    );
    view.read_with(cx, |view, cx| {
        let input = view.chats.pane("w1:p1").unwrap().composer.as_ref().unwrap();
        assert_eq!(input.read(cx).text(), "keep my draft");
    });
}

#[gpui::test]
fn the_largest_permission_id_renders_without_overflow(cx: &mut TestAppContext) {
    let (view, _daemon, cx) = open(cx, ConnectTarget::Local);
    let program = Program::new();
    let _stream = attach_with(&view, &program, waiting(u64::MAX), cx);
    assert!(cx.debug_bounds("chat-answer-0").is_some());
}

#[gpui::test]
fn the_agent_settings_gear_opens_and_dismisses_from_the_keyboard(cx: &mut TestAppContext) {
    let (view, _daemon, cx) = open(cx, ConnectTarget::Local);
    let program = Program::new();
    let mut message = menus::settings();
    if let ServerMessage::Snapshot { transcript, .. } = &mut message {
        transcript.options.push(sheprd_chat_protocol::ConfigOption {
            category: sheprd_chat_protocol::OptionCategory::Other,
            description: None,
            id: "fixture-toggle".into(),
            kind: sheprd_chat_protocol::OptionKind::Toggle { current: false },
            name: "Fixture option".into(),
        });
    }
    let mut stream = attach_with(&view, &program, message, cx);
    focus_control(&view, &program, "ChatAgentSettings", cx);
    let enter = crate::keymap::parse_keystroke("enter").unwrap();
    cx.simulate_event(KeyDownEvent {
        keystroke: enter.clone(),
        is_held: false,
        prefer_character_input: false,
    });
    cx.simulate_event(KeyUpEvent { keystroke: enter });
    poll(&view, &program, cx);
    assert!(cx.debug_bounds("chat-menu").is_some());
    cx.simulate_keystrokes("escape");
    poll(&view, &program, cx);
    assert!(cx.debug_bounds("chat-menu").is_none());
    assert!(cx.update(|window, cx| {
        view.read(cx)
            .chats
            .pane("w1:p1")
            .unwrap()
            .composer
            .as_ref()
            .unwrap()
            .read(cx)
            .focus
            .is_focused(window)
    }));
    view.update(cx, |view, cx| view.chat_send("w1:p1", Command::Cancel, cx));
    assert_eq!(command(&mut stream), Command::Cancel);
}

#[gpui::test]
fn a_held_key_cannot_answer_a_replacement_session_with_reused_permission_ids(
    cx: &mut TestAppContext,
) {
    let (view, _daemon, cx) = open(cx, ConnectTarget::Local);
    let program = Program::new();
    let mut stream = attach_with(&view, &program, waiting(41), cx);
    focus_answer(&view, &program, cx);
    let enter = crate::keymap::parse_keystroke("enter").unwrap();
    cx.simulate_event(KeyDownEvent {
        keystroke: enter.clone(),
        is_held: false,
        prefer_character_input: false,
    });
    let mut replacement = waiting(41);
    if let ServerMessage::Snapshot { cursor: at, .. } = &mut replacement {
        *at = cursor(2, 0);
    }
    send(&mut stream, &replacement);
    wait_until(|| {
        poll(&view, &program, cx);
        view.read_with(cx, |view, _| {
            view.chats.pane("w1:p1").unwrap().session.since() == Some(cursor(2, 0))
        })
    });
    cx.simulate_event(KeyUpEvent { keystroke: enter });
    view.update(cx, |view, cx| view.chat_send("w1:p1", Command::Cancel, cx));
    assert_eq!(command(&mut stream), Command::Cancel);
}
