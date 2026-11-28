use super::*;
use spaceport_menu::workspace::popover::layout;

#[gpui::test]
fn the_keyboard_walks_actions_in_the_order_they_are_drawn(cx: &mut gpui::TestAppContext) {
    let (view, cx) = cx.add_window_view(sidebar::layout_tests::fixture_window);
    cx.update(|window, cx| {
        view.update(cx, |view, cx| {
            view.live.status = crate::state::ConnectionStatus::Connected;
            view.open_workspace_menu("w3", Default::default(), window, cx);
        })
    });
    let items = cx.update(|_, cx| view.read(cx).workspace_items());
    let actions: Vec<_> = items.iter().map(|item| item.action).collect();
    let (tiles, rest) = layout(&items);
    // The header's toggle, then the grid and the rows under it.
    let drawn: Vec<_> = items
        .iter()
        .filter(|item| item.action.in_header())
        .chain(tiles)
        .chain(rest)
        .map(|item| item.action)
        .collect();
    assert_eq!(drawn, actions, "keyboard order must match the drawn order");
    // Right walks forward and left walks back, like down and up.
    for (key, expected) in [
        ("right", 0),
        ("right", 1),
        ("left", 0),
        ("left", actions.len() - 1),
    ] {
        cx.simulate_keystrokes(key);
        cx.update(|_, cx| {
            assert_eq!(
                view.read(cx).menu.workspace_selected,
                Some(actions[expected]),
                "{key}"
            );
        });
    }
}

#[gpui::test]
fn rows_share_icon_and_label_columns(cx: &mut gpui::TestAppContext) {
    use gpui::{Bounds, Pixels, px};
    let (view, cx) = cx.add_window_view(sidebar::layout_tests::fixture_window);
    // One icon column and one label column, across popovers opened at the same anchor.
    let mut measure =
        |id: &str, labels: &[&'static str]| -> Vec<(Bounds<Pixels>, Bounds<Pixels>)> {
            cx.update(|window, cx| {
                view.update(cx, |view, cx| {
                    view.live.status = crate::state::ConnectionStatus::Connected;
                    view.dismiss_menu(window, cx);
                    view.open_workspace_menu(id, Default::default(), window, cx);
                });
                window.draw(cx).clear(cx);
            });
            labels
                .iter()
                .map(|label| {
                    let icon = cx
                        .debug_bounds(format!("workspace-menu-icon-{label}").leak())
                        .unwrap();
                    let text = cx
                        .debug_bounds(format!("workspace-menu-label-{label}").leak())
                        .unwrap();
                    (icon, text)
                })
                .collect()
        };
    let measured = measure("w3", &["Note...", "Collapse group"]);
    let (icon, text) = measured[0];
    for (other_icon, other_text) in &measured[1..] {
        assert!((other_icon.center().x - icon.center().x).abs() <= px(0.5));
        assert_eq!(other_text.left(), text.left());
        assert!((other_icon.center().y - other_text.center().y).abs() <= px(1.));
    }
    assert!(icon.right() <= text.left());
}
