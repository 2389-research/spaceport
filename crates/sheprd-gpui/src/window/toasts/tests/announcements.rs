// ABOUTME: Checks accessibility metadata on real notification card elements.
// ABOUTME: Keeps departing paint copies from announcing an old notification again.
use super::*;
use crate::window::toasts::{
    card::Look,
    motion::{Drawn, Key},
};
use gpui::{Element, IntoElement, Role};
use sheprd_client::protocol::SemanticNotificationKind;

#[gpui::test]
fn visible_toast_has_an_announced_status_node(cx: &mut TestAppContext) {
    let (view, cx) = cx.add_window_view(fixture_window);
    view.update(cx, |view, cx| {
        view.show_toast_preview(SemanticNotificationKind::Custom, cx);
        let endpoint = &view.endpoints[0];
        let (id, notice) = endpoint.toasts.entries.back().unwrap();
        let drawn = Drawn {
            key: Key {
                endpoint: endpoint.id.clone(),
                generation: endpoint.generation,
                id: *id,
            },
            label: endpoint.label.clone(),
            source: endpoint.connection_ref(),
            notice: notice.clone(),
            corner: notice.position,
        };
        let element = view
            .toast_card(
                &drawn,
                Look {
                    shown: 1.,
                    live: true,
                    navigable: false,
                    max_height: px(200.),
                    body_room: px(100.),
                },
                cx,
            )
            .into_element();
        assert_eq!(element.a11y_role(), Some(Role::Status));
        let mut node = gpui::accesskit::Node::new(Role::Status);
        element.write_a11y_info(&mut node);
        assert!(node.label().is_some_and(|label| !label.is_empty()));
    });
}
