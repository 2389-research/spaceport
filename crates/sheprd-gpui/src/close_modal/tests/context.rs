// ABOUTME: Checks the actual confirmation text presented to assistive technology.
// ABOUTME: Keeps outgoing confirmation captures absent from the accessible tree.
use super::{Role, close_error, close_text};
use gpui::{Element, accesskit};

#[test]
fn active_confirmation_text_exposes_its_visible_value_without_an_action() {
    for (id, text) in [
        ("close-title", "Close workspace?"),
        ("close-target-text", "private-workspace"),
        ("close-note", "This closes one terminal pane."),
        ("close-interrupts", "An agent is waiting on you."),
    ] {
        let element = close_text(id, text.into(), true);
        assert_eq!(element.a11y_role(), Some(Role::Label), "{id}");
        let mut node = accesskit::Node::new(Role::Label);
        element.write_a11y_info(&mut node);
        assert_eq!(node.value(), Some(text), "{id}");
        assert!(!node.supports_action(accesskit::Action::Click));
        assert!(!node.supports_action(accesskit::Action::Focus));
    }
}

#[test]
fn a_close_refusal_is_an_accessible_alert_without_an_action() {
    let element = close_error("Captured target changed".into(), true);
    assert_eq!(element.a11y_role(), Some(Role::Alert));
    let mut node = accesskit::Node::new(Role::Alert);
    element.write_a11y_info(&mut node);
    assert_eq!(node.label(), Some("Captured target changed"));
    assert!(!node.supports_action(accesskit::Action::Click));
    assert!(!node.supports_action(accesskit::Action::Focus));
    assert_eq!(close_error("Captured".into(), false).a11y_role(), None);
}

#[test]
fn departing_confirmation_text_has_no_accessibility_node() {
    assert_eq!(
        close_text("close-target-text", "Captured".into(), false).a11y_role(),
        None
    );
}
