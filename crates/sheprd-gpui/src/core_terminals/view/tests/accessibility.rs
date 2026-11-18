// ABOUTME: Checks the actual Core terminal element's accessibility metadata.
// ABOUTME: Keeps terminal reading available for both interactive and observing panes.
use crate::{
    core_terminals::{view::CoreTerminalView, worker::Link},
    sidebar::layout_tests::fixture_window,
};
use gpui::{AppContext, Element, IntoElement, Render, Role};
use sheprd_backend::TerminalId;

#[gpui::test]
fn core_terminal_render_exposes_a_named_terminal_node(cx: &mut gpui::TestAppContext) {
    let (owner, cx) = cx.add_window_view(fixture_window);
    for observe in [false, true] {
        let terminal = cx.update(|_, cx| {
            let owner = owner.read(cx);
            let look = owner.core_look();
            let painter = owner.painter.clone();
            cx.new(|cx| {
                CoreTerminalView::new(
                    TerminalId::new("accessible-core").unwrap(),
                    Link::for_test(),
                    look,
                    painter,
                    observe,
                    cx,
                )
            })
        });
        terminal.update_in(cx, |terminal, window, cx| {
            let element = terminal.render(window, cx).into_element();
            assert_eq!(element.a11y_role(), Some(Role::Terminal));
            let mut node = gpui::accesskit::Node::new(Role::Terminal);
            element.write_a11y_info(&mut node);
            assert!(node.label().is_some_and(|label| !label.is_empty()));
        });
    }
}
