use super::*;
use crate::menu::workspace::popover::{Label, Tile, layout};

fn tile(action: WorkspaceMenuAction, label: Label, enabled: bool) -> Tile {
    Tile {
        action,
        label,
        enabled,
    }
}

#[test]
fn every_slot_keeps_its_place_and_grays_what_is_unavailable() {
    use WorkspaceMenuAction::*;
    // A plain checkout has no worktree or fan-out action to offer.
    let items = [
        (Dialog(WorkspaceAction::Rename), Label::RENAME),
        (Dialog(WorkspaceAction::Close), Label::CLOSE),
    ];
    let (tiles, rest) = layout(&items);
    assert_eq!(
        tiles,
        [
            tile(FanOut, Label::FAN_OUT, false),
            tile(Dialog(WorkspaceAction::Rename), Label::RENAME, true),
            tile(Dialog(WorkspaceAction::Close), Label::CLOSE, true),
        ]
    );
    assert!(rest.is_empty());
}

#[test]
fn rows_and_the_destructive_action_follow_the_grid_in_order() {
    use WorkspaceMenuAction::*;
    let items = [
        (Dialog(WorkspaceAction::Rename), Label::RENAME),
        (Dialog(WorkspaceAction::Close), Label::CLOSE_GROUP),
        (Dialog(WorkspaceAction::OpenWorktree), Label::OPEN_WORKTREE),
        (Collapse, Label::COLLAPSE_GROUP),
        (
            Dialog(WorkspaceAction::DeleteWorktree),
            Label::DELETE_WORKTREE,
        ),
    ];
    let (tiles, rest) = layout(&items);
    assert!(tiles[1].enabled && tiles[2].enabled);
    assert_eq!(tiles[2].label, Label::CLOSE_GROUP);
    assert_eq!(
        rest,
        [
            (Dialog(WorkspaceAction::OpenWorktree), Label::OPEN_WORKTREE),
            (Collapse, Label::COLLAPSE_GROUP),
            (
                Dialog(WorkspaceAction::DeleteWorktree),
                Label::DELETE_WORKTREE
            ),
        ]
    );
}

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
    let (tiles, rest) = layout(&items);
    // The header's toggle, then the grid and the rows under it.
    let drawn: Vec<_> = items
        .iter()
        .copied()
        .filter(|(action, _)| action.in_header())
        .chain(
            tiles
                .iter()
                .filter(|tile| tile.enabled)
                .map(|tile| (tile.action, tile.label)),
        )
        .chain(rest)
        .collect();
    assert_eq!(drawn, items, "keyboard order must match the drawn order");
    // Right walks forward and left walks back, like down and up.
    for (key, expected) in [
        ("right", 0),
        ("right", 1),
        ("left", 0),
        ("left", items.len() - 1),
    ] {
        cx.simulate_keystrokes(key);
        cx.update(|_, cx| {
            assert_eq!(
                view.read(cx).menu.workspace_selected,
                Some(items[expected].0),
                "{key}"
            );
        });
    }
}

#[gpui::test]
fn rows_and_the_delete_strip_share_icon_and_label_columns(cx: &mut gpui::TestAppContext) {
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
    let mut measured = measure("w3", &["Open worktree...", "Collapse group"]);
    measured.extend(measure("w4", &["Delete worktree checkout"]));
    let (icon, text) = measured[0];
    for (other_icon, other_text) in &measured[1..] {
        assert!((other_icon.center().x - icon.center().x).abs() <= px(0.5));
        assert_eq!(other_text.left(), text.left());
        assert!((other_icon.center().y - other_text.center().y).abs() <= px(1.));
    }
    assert!(icon.right() <= text.left());
}
