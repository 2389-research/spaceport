use crate::page::{WorkspaceAction, WorkspaceMenuAction};
use crate::workspace::popover::{self, Item};
use gpui::px;

/// An action as the app would hand it over: named and captioned as it says.
fn item(action: WorkspaceMenuAction, name: &str) -> Item {
    Item {
        action,
        caption: name.to_owned(),
        icon: None,
        label: name.to_owned(),
        name: name.to_owned(),
    }
}

fn actions<'a>(items: impl IntoIterator<Item = &'a Item>) -> Vec<WorkspaceMenuAction> {
    items.into_iter().map(|item| item.action).collect()
}

#[test]
fn every_slot_keeps_its_place_and_the_rows_follow_in_order() {
    use WorkspaceMenuAction::*;
    // A plain checkout keeps the ordinary workspace actions.
    let items = [item(Dialog(WorkspaceAction::Rename), "Rename")];
    let (tiles, rest) = popover::layout(&items);
    assert_eq!(actions(tiles), [Dialog(WorkspaceAction::Rename)]);
    assert!(rest.is_empty());
}

#[test]
fn rows_follow_the_grid_in_order() {
    use WorkspaceMenuAction::*;
    let items = [
        item(Dialog(WorkspaceAction::Rename), "Rename"),
        item(Dialog(WorkspaceAction::Note), "Note..."),
        item(Collapse, "Collapse group"),
    ];
    let (tiles, rest) = popover::layout(&items);
    assert_eq!(actions(tiles), [Dialog(WorkspaceAction::Rename)]);
    assert_eq!(
        actions(rest),
        [Dialog(WorkspaceAction::Note), Collapse],
        "the rows keep the order the actions came in"
    );
}

#[test]
fn the_header_draws_its_action_in_neither_the_grid_nor_the_rows() {
    use WorkspaceMenuAction::*;
    let items = [
        item(Mute, "Mute notifications"),
        item(Dialog(WorkspaceAction::Rename), "Rename"),
        item(Dialog(WorkspaceAction::Note), "Note..."),
    ];
    let (tiles, rest) = popover::layout(&items);
    assert!(!tiles.iter().any(|item| item.action.in_header()));
    assert!(!rest.iter().any(|item| item.action.in_header()));
}

#[test]
fn a_shortened_caption_still_names_the_action_in_full() {
    let mute = Item {
        action: WorkspaceMenuAction::Mute,
        caption: "Mute".into(),
        icon: None,
        label: "Mute notifications".into(),
        name: "Mute notifications".into(),
    };
    assert_eq!(mute.spoken(), "Mute notifications");
    assert_eq!(mute.selector(), "workspace-menu-Mute notifications");
    let note = item(
        WorkspaceMenuAction::Dialog(WorkspaceAction::Note),
        "Note...",
    );
    assert_eq!(note.spoken(), "Note");
}

#[test]
fn the_width_grows_for_a_longer_caption_and_never_past_the_room() {
    let items = [item(
        WorkspaceMenuAction::Dialog(WorkspaceAction::Rename),
        "Rename",
    )];
    // Every caption fits its tile, so the popover keeps its English width.
    assert_eq!(popover::width(&items, px(1000.), |_| px(44.)), px(340.));
    let wide = [item(
        WorkspaceMenuAction::Dialog(WorkspaceAction::Rename),
        "Rename this workspace",
    )];
    assert_eq!(popover::width(&wide, px(1000.), |_| px(400.)), px(436.));
    // A narrow window takes the room it has, and no more.
    assert_eq!(popover::width(&wide, px(200.), |_| px(400.)), px(200.));
}
