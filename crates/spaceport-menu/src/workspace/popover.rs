//! The workspace popover's body: a header naming the workspace, with the toggle
//! that mutes its notifications, a fixed grid of large action tiles, and plain
//! rows under it for the rarer actions.
//!
//! The app words every caption, names every selector its tests read, and hands
//! back what resting on an action and pressing it do through `interact`; what
//! is here is the layout and the paint.

use crate::chrome::Text;
use crate::page::{WorkspaceAction, WorkspaceMenuAction};
use gpui::prelude::*;
use gpui::*;
use spaceport_settings::{Theme, corners};

/// One action the popover offers, as the app worded it.
#[derive(Clone, Debug, PartialEq)]
pub struct Item {
    /// The action it runs.
    pub action: WorkspaceMenuAction,
    /// What its row says, which is also what a screen reader calls the short
    /// caption a tile or the header's toggle shows.
    pub label: String,
    /// The short caption a tile or the header's toggle shows, where its mark and
    /// its place already say the rest.
    pub caption: String,
    /// The asset its mark draws, where the action has one.
    pub icon: Option<&'static str>,
    /// The action's English name, which names its elements: the same in every
    /// language, so this app's tests find them whatever is shown.
    pub name: String,
}

impl Item {
    /// The name this app's tests find the action's element by.
    pub fn selector(&self) -> String {
        format!("workspace-menu-{}", self.name)
    }

    /// The name this app's tests find the caption inside it by.
    fn label_selector(&self) -> String {
        format!("workspace-menu-label-{}", self.name)
    }

    /// The name this app's tests find its mark by, where it has one.
    fn icon_selector(&self) -> String {
        format!("workspace-menu-icon-{}", self.name)
    }

    /// What a screen reader calls the action: its label without the "..." that
    /// marks a row opening a dialog, which the caption leaves out anyway.
    pub fn spoken(&self) -> String {
        self.label
            .trim_end_matches("...")
            .trim_end_matches('\u{2026}')
            .to_owned()
    }
}

/// The element a word the popover shows is painted as. The app builds it, so
/// that its headless and native layout tests can read the laid-out text back
/// from the element they probe.
pub type Painter = fn(&str) -> AnyElement;

/// What the popover draws from.
pub struct Hosts<'a> {
    /// The accent a muted workspace's toggle keeps, which is the app's own
    /// colour policy.
    pub accent: Rgba,
    /// The branch the workspace is on, under its name, when it has one.
    pub branch: Option<&'a str>,
    /// The actions the popover offers, in the order the arrow keys walk them.
    pub items: &'a [Item],
    /// The workspace's name, which the header is the one place to show in full.
    pub name: &'a str,
    /// How the app paints a word of it.
    pub paint: Painter,
    /// The action the arrow keys have selected.
    pub selected: Option<WorkspaceMenuAction>,
    /// The chrome's font.
    pub text: &'a Text,
    /// The colours it paints with.
    pub theme: &'a Theme,
}

/// The grid's fixed slots, row by row: the actions each takes, first available
/// first, so a slot never moves because another action came or went.
const SLOTS: [&[WorkspaceMenuAction]; 1] = {
    use WorkspaceAction::Rename;
    use WorkspaceMenuAction::Dialog;
    [&[Dialog(Rename)]]
};

/// How many tiles a grid row holds.
const COLUMNS: usize = 1;

/// The rows under the grid share one icon column and one label column: the
/// same inset, a fixed icon slot each icon centres in whatever its size, and
/// the same gap before the label.
const ROW_INSET: f32 = 8.;
const ICON_SLOT: f32 = 20.;
const ICON_SIZE: f32 = 16.;
const ROW_GAP: f32 = 8.;

/// The popover's width in English, where every caption fits its tile.
const WIDTH: f32 = 340.;

/// What the popover adds around its grid (inset, border, and the grid's own
/// padding), and around each caption (the tile's padding).
const PANEL_INSET: f32 = 28.;
const TILE_INSET: f32 = 8.;
const TILE_GAP: f32 = 6.;

/// `items` split into the grid's tiles, in the slots' order, and the rows under
/// them, in the order given. An action the header draws is in neither.
pub fn layout(items: &[Item]) -> (Vec<&Item>, Vec<&Item>) {
    let tiles: Vec<&Item> = SLOTS
        .iter()
        .filter_map(|choices| {
            choices
                .iter()
                .find_map(|choice| items.iter().find(|item| item.action == *choice))
        })
        .collect();
    let rest = items
        .iter()
        .filter(|item| {
            !item.action.in_header() && !tiles.iter().any(|tile| tile.action == item.action)
        })
        .collect();
    (tiles, rest)
}

/// The popover's width: 340, or as wide as the longest caption needs in a
/// longer language, and never wider than `room`. `shape` measures one caption in
/// the window's own text system, which caches each shaped line, so measuring
/// them as the popover draws costs no more than drawing them.
pub fn width(items: &[Item], room: Pixels, shape: impl Fn(&str) -> Pixels) -> Pixels {
    let widest = layout(items)
        .0
        .iter()
        .map(|tile| shape(&tile.caption))
        .max()
        .unwrap_or_default();
    let columns = COLUMNS as f32;
    let needed = (widest + px(TILE_INSET)) * columns + px(TILE_GAP * (columns - 1.) + PANEL_INSET);
    needed.max(px(WIDTH)).min(room)
}

/// The popover's body. `interact` takes one action's element back wired with
/// what resting on it and pressing it do, which only the window knows.
pub fn panel(
    hosts: &Hosts<'_>,
    mut interact: impl FnMut(WorkspaceMenuAction, Stateful<Div>) -> Stateful<Div>,
) -> Div {
    let theme = hosts.theme;
    let text = hosts.text;
    let selected = hosts.selected;
    let names = div()
        .flex_1()
        .min_w_0()
        .child(
            div()
                .debug_selector(|| "workspace-menu-name".into())
                // Long names wrap: the header is the one place the
                // full name shows when the sidebar row cuts it off.
                .whitespace_normal()
                .font_weight(FontWeight::SEMIBOLD)
                .child((hosts.paint)(hosts.name)),
        )
        .when_some(
            hosts.branch.filter(|branch| !branch.trim().is_empty()),
            |names, branch| {
                names.child(
                    div()
                        .debug_selector(|| "workspace-menu-branch".into())
                        .truncate()
                        .text_color(rgb(theme.muted))
                        .text_size(px(text.size * 0.9))
                        .child((hosts.paint)(branch)),
                )
            },
        );
    let header = div()
        .debug_selector(|| "workspace-menu-header".into())
        .px(px(8.))
        .py(px(6.))
        .mb(px(4.))
        .border_b_1()
        .border_color(rgb(theme.active))
        .flex()
        .items_start()
        .gap(px(8.))
        .child(names)
        .children(
            hosts
                .items
                .iter()
                .find(|item| item.action.in_header())
                .map(|item| {
                    let toggle = toggle(
                        theme,
                        text,
                        hosts.accent,
                        item,
                        selected == Some(item.action),
                    );
                    interact(item.action, toggle)
                }),
        );
    let (tiles, rest) = layout(hosts.items);
    let mut grid = div()
        .debug_selector(|| "workspace-menu-tiles".into())
        .flex()
        .flex_col()
        .gap(px(TILE_GAP))
        .p(px(2.));
    for slot in tiles.chunks(COLUMNS) {
        grid = grid.child(
            div()
                .flex()
                .gap(px(TILE_GAP))
                .children(slot.iter().map(|item| {
                    let tile = tile(theme, text, item, selected == Some(item.action));
                    interact(item.action, tile)
                })),
        );
    }
    let mut body = div().flex().flex_col().child(header).child(grid);
    if !rest.is_empty() {
        body = body.child(div().h(px(TILE_GAP)));
    }
    rest.into_iter().fold(body, |body, item| {
        let row = row(theme, text, item, selected == Some(item.action));
        body.child(interact(item.action, row))
    })
}

/// One grid cell: the action it runs, its caption and its mark. The grid's
/// tiles are large so a workspace's few actions read at a glance.
fn tile(theme: &Theme, text: &Text, item: &Item, selected: bool) -> Stateful<Div> {
    let ink = rgb(theme.foreground);
    let resting = rgb(theme.surface).blend(rgba((theme.foreground << 8) | 0x0f));
    let selector = item.selector();
    let label_selector = item.label_selector();
    let icon_selector = item.icon_selector();
    div()
        .id(selector.clone())
        .role(Role::MenuItem)
        .aria_label(item.spoken())
        .when(selected, |tile| tile.aria_active_descendant())
        .debug_selector(move || selector)
        .flex_1()
        .min_w_0()
        .min_h(px(64.))
        .px(px(4.))
        .py(px(10.))
        .flex()
        .flex_col()
        .items_center()
        .justify_center()
        .gap(px(6.))
        .rounded(px(corners::CONTROL))
        .bg(if selected { rgb(theme.active) } else { resting })
        .text_color(ink)
        .text_size(px((text.size - 1.).max(8.)))
        .when_some(item.icon, |tile, icon| {
            tile.child(
                svg()
                    .path(icon)
                    .size(px(20.))
                    .flex_none()
                    .text_color(ink)
                    .debug_selector(move || icon_selector),
            )
        })
        .child(
            div()
                .debug_selector(move || label_selector)
                .max_w_full()
                .truncate()
                .child(item.caption.clone()),
        )
        .cursor_pointer()
}

/// The header's toggle for the workspace's notifications, `item` with its short
/// caption: what pressing it does, with the bell it leaves. In the header, so
/// muting adds no row to a short window; still first in the arrow keys' walk.
/// Muted, it keeps the accent, so the state reads at a glance.
pub fn toggle(
    theme: &Theme,
    text: &Text,
    accent: Rgba,
    item: &Item,
    selected: bool,
) -> Stateful<Div> {
    let muted = item.action == WorkspaceMenuAction::Unmute;
    let ink = if muted { accent } else { rgb(theme.muted) };
    // One line of the header tall, so it never makes the header taller.
    let line = text.line_height;
    let selector = item.selector();
    div()
        .id(selector.clone())
        .debug_selector(move || selector)
        .role(Role::Button)
        .aria_label(item.spoken())
        .when(selected, |toggle| toggle.aria_active_descendant())
        .flex_none()
        .h(px(line))
        .flex()
        .items_center()
        .gap(px(4.))
        .px(px(6.))
        .rounded(px(corners::CONTROL))
        .border_1()
        .border_color(Rgba { a: 0.5, ..ink })
        .map(|toggle| match (selected, muted) {
            (true, _) => toggle.bg(rgb(theme.active)),
            (false, true) => toggle.bg(Rgba { a: 0.14, ..ink }),
            (false, false) => toggle,
        })
        .text_color(ink)
        .text_size(px((text.size - 2.).max(8.)))
        .cursor_pointer()
        .when_some(item.icon, |toggle, icon| {
            toggle.child(
                svg()
                    .path(icon)
                    .size(px((line - 6.).clamp(8., ICON_SIZE - 4.)))
                    .flex_none()
                    .text_color(ink)
                    .debug_selector(|| "workspace-menu-mute-icon".into()),
            )
        })
        .child(item.caption.clone())
}

/// A rarer action, as a plain row under the tiles.
fn row(theme: &Theme, text: &Text, item: &Item, selected: bool) -> Stateful<Div> {
    let selector = item.selector();
    let label_selector = item.label_selector();
    div()
        .id(selector.clone())
        .debug_selector(move || selector)
        .role(Role::MenuItem)
        .aria_label(item.spoken())
        .when(selected, |row| row.aria_active_descendant())
        .mx(px(2.))
        .min_h(px(text.line_height + 12.))
        .px(px(ROW_INSET))
        .flex()
        .items_center()
        .gap(px(ROW_GAP))
        .cursor_pointer()
        .rounded(px(corners::CONTROL))
        .when(selected, |row| row.bg(rgb(theme.active)))
        .child(leading_icon(
            item,
            rgb(if selected {
                theme.foreground
            } else {
                theme.muted
            }),
        ))
        .child(
            div()
                .flex_1()
                .min_w_0()
                .truncate()
                .debug_selector(move || label_selector)
                .child(item.label.clone()),
        )
}

/// A row's leading icon, centred in the shared slot. An action without an icon
/// keeps the empty slot so its label still lines up.
fn leading_icon(item: &Item, color: Rgba) -> Div {
    div()
        .size(px(ICON_SLOT))
        .flex_none()
        .flex()
        .items_center()
        .justify_center()
        .when_some(item.icon, |slot, icon| {
            let selector = item.icon_selector();
            slot.child(
                svg()
                    .path(icon)
                    .size(px(ICON_SIZE))
                    .flex_none()
                    .text_color(color)
                    .debug_selector(move || selector),
            )
        })
}
