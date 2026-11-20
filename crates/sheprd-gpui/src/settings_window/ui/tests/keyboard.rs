// ABOUTME: Exercises Settings controls through GPUI focus traversal and keyboard clicks.
// ABOUTME: Keeps disabled controls out of navigation and checks actual accessibility actions.
use super::*;
use crate::settings_window::tests::press_key;
use gpui::{
    Element, FocusHandle,
    accesskit::{Action, Node},
};

struct Keyboard {
    focus: FocusHandle,
    events: Rc<RefCell<Vec<&'static str>>>,
}
impl Render for Keyboard {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let theme = Theme::default();
        let toggled = self.events.clone();
        let pressed = self.events.clone();
        let picked = self.events.clone();
        div()
            .track_focus(&self.focus)
            .size_full()
            .child(
                toggle(&theme, "keyboard-switch", false, true)
                    .on_click(move |_, _, _| toggled.borrow_mut().push("switch")),
            )
            .child(
                toggle(&theme, "disabled-switch", false, false)
                    .on_click(|_, _, _| panic!("disabled switch received a key")),
            )
            .child(
                button(&theme, "keyboard-button", "Apply", true)
                    .on_click(move |_, _, _| pressed.borrow_mut().push("button")),
            )
            .child(
                button(&theme, "disabled-button", "Unavailable", false)
                    .on_click(|_, _, _| panic!("disabled button received a key")),
            )
            .child(segmented(
                &theme,
                "keyboard-segment",
                &[(1, "One"), (2, "Two")],
                Some(1),
                true,
                move |_, _, _| picked.borrow_mut().push("segment"),
            ))
    }
}

#[gpui::test]
fn enabled_controls_are_tab_stops_and_keyboard_clicks_use_their_real_callbacks(
    cx: &mut TestAppContext,
) {
    let events = Rc::new(RefCell::new(Vec::new()));
    let (_, cx) = cx.add_window_view(|window, cx| {
        let focus = cx.focus_handle();
        window.focus(&focus, cx);
        Keyboard {
            focus,
            events: events.clone(),
        }
    });
    cx.update(|window, cx| window.draw(cx).clear(cx));
    for key in ["space", "enter", "space", "enter"] {
        cx.update(|window, cx| {
            window.focus_next(cx);
            window.draw(cx).clear(cx);
        });
        press_key(cx, key);
    }
    assert_eq!(*events.borrow(), ["switch", "button", "segment"]);
    cx.update(|window, cx| {
        window.focus_prev(cx);
        window.focus_prev(cx);
        window.draw(cx).clear(cx);
    });
    press_key(cx, "space");
    assert_eq!(*events.borrow(), ["switch", "button", "segment", "button"]);
}

#[test]
fn enabled_control_nodes_offer_focus_and_disabled_controls_do_not() {
    let theme = Theme::default();
    for enabled in [false, true] {
        for element in [
            toggle(&theme, "switch", false, enabled),
            button(&theme, "button", "Apply", enabled),
        ] {
            let mut node = Node::new(element.a11y_role().unwrap());
            element.write_a11y_info(&mut node);
            assert_eq!(node.supports_action(Action::Focus), enabled);
        }
    }
}

#[test]
fn dropdown_buttons_expose_focus_and_expanded_state() {
    let theme = Theme::default();
    for enabled in [false, true] {
        let element = dropdown(
            &theme,
            "keyboard-dropdown",
            "Choose",
            false,
            enabled,
            None,
            |_, _| {},
            |_, _, _| {},
            |_, _| {},
        );
        assert_eq!(element.a11y_role(), Some(Role::Button));
        let mut node = Node::new(Role::Button);
        element.write_a11y_info(&mut node);
        assert_eq!(node.supports_action(Action::Focus), enabled);
        assert_eq!(node.is_expanded(), Some(false));
    }
}
