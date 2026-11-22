// ABOUTME: Checks the version text exposed by the status bar's real renderer.
// ABOUTME: Keeps the passive label readable without adding a control action.
use gpui::{Element, accesskit};

#[test]
fn the_version_is_accessible_text_without_an_action() {
    let element = super::version(&crate::config::Theme::default());
    assert_eq!(element.a11y_role(), Some(accesskit::Role::Label));
    let mut node = accesskit::Node::new(accesskit::Role::Label);
    element.write_a11y_info(&mut node);
    let spoken = crate::i18n::fl!("status-bar-version", version = crate::APP_VERSION);
    assert_eq!(node.value(), Some(spoken.as_str()));
    assert!(!node.supports_action(accesskit::Action::Click));
    assert!(!node.supports_action(accesskit::Action::Focus));
}
