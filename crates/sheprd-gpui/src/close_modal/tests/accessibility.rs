// ABOUTME: Checks the actual close controls' roles and localized names.
// ABOUTME: Leaves the dialog's focus ownership on the existing menu overlay.
use super::*;
use gpui::{Element, IntoElement};

#[test]
fn close_controls_expose_named_button_nodes() {
    for (id, label) in [
        ("close-cancel", fl!("close-modal-cancel")),
        ("close-confirm", fl!("close-modal-close-pane")),
    ] {
        let element = close_button(id, label.clone(), true).into_element();
        assert_eq!(element.a11y_role(), Some(Role::Button));
        let mut node = accesskit::Node::new(Role::Button);
        element.write_a11y_info(&mut node);
        assert_eq!(node.label(), Some(label.as_str()));
    }
    assert_eq!(Page::ConfirmClose.accessible_role(), Role::Dialog);
    assert!(!Page::ConfirmClose.accessible_name().is_empty());
}

#[test]
fn departing_close_controls_do_not_expose_action_nodes() {
    for id in ["close-cancel", "close-confirm"] {
        let element = close_button(id, "Captured consent".into(), false).into_element();
        assert_eq!(element.a11y_role(), None);
    }
}
