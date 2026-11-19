// ABOUTME: Exercises the installed preview composer's real focus and accessible-name contracts.
// ABOUTME: Keeps native qualification fixtures aligned with the production chat input owner.
#![allow(clippy::unwrap_used)]
use super::*;
use crate::multiline_input::MultilineInput;
use gpui::{Element, IntoElement, KeyDownEvent, KeyUpEvent, Render, Styled, accesskit};

#[gpui::test]
fn preview_composer_returns_through_tab_traversal_without_editing_the_draft(
    cx: &mut gpui::TestAppContext,
) {
    let (view, cx) = cx.add_window_view(crate::sidebar::layout_tests::fixture_window);
    let mut options = conversation_options();
    options.draft = Some("Keep this fixture draft".into());
    cx.update(|window, cx| {
        view.update(cx, |view, cx| install(view, window, cx, &options).unwrap())
    });
    let input = view.read_with(cx, |view, _| {
        view.chats.pane(PANE).unwrap().composer.clone().unwrap()
    });
    let focus = input.read_with(cx, |input, _| input.focus.clone());
    cx.update(|window, cx| {
        window.refresh();
        window.draw(cx).clear(cx);
        focus.focus(window, cx);
    });
    let mut left = false;
    let mut returned = false;
    for _ in 0..64 {
        let keystroke = crate::keymap::parse_keystroke("tab").unwrap();
        cx.simulate_event(KeyDownEvent {
            keystroke: keystroke.clone(),
            is_held: false,
            prefer_character_input: false,
        });
        cx.simulate_event(KeyUpEvent { keystroke });
        cx.update(|window, cx| {
            window.refresh();
            window.draw(cx).clear(cx);
        });
        let focused = cx.update(|window, _| focus.is_focused(window));
        if !focused {
            left = true;
        }
        if left && focused {
            returned = true;
            break;
        }
    }
    assert!(
        left && returned,
        "Tab must leave and return to the real preview composer"
    );
    assert_eq!(
        input.read_with(cx, |input, _| input.text().to_owned()),
        options.draft.unwrap()
    );
}

#[gpui::test]
fn preview_composer_exposes_the_production_accessible_name(cx: &mut gpui::TestAppContext) {
    struct Probe {
        input: gpui::Entity<MultilineInput>,
        output: std::rc::Rc<std::cell::RefCell<Option<accesskit::Node>>>,
    }
    impl Render for Probe {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            let input = self.input.clone();
            let output = self.output.clone();
            gpui::canvas(
                move |_, window, cx| {
                    input.update(cx, |input, cx| {
                        let element = input.render(window, cx).into_element();
                        let mut node = accesskit::Node::new(element.a11y_role().unwrap());
                        element.write_a11y_info(&mut node);
                        *output.borrow_mut() = Some(node);
                    })
                },
                |_, _, _, _| {},
            )
            .size_full()
        }
    }
    let (view, cx) = cx.add_window_view(crate::sidebar::layout_tests::fixture_window);
    cx.update(|window, cx| {
        view.update(cx, |view, cx| {
            install(view, window, cx, &conversation_options()).unwrap()
        })
    });
    let input = view.read_with(cx, |view, _| {
        view.chats.pane(PANE).unwrap().composer.clone().unwrap()
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
