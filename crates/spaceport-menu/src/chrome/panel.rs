//! The panel a page draws in: where a popover hangs, how much room each kind
//! of page takes, the surface it paints on, and where the panel lands.

use super::{
    ADD_DEVICE_HEIGHT, ADD_DEVICE_WIDTH, DEVICES_MENU_GAP, DEVICES_MENU_WIDTH, Hosts, Layer,
    MENU_MARGIN, footer_room,
};
use crate::page::Page;
use gpui::prelude::*;
use gpui::*;
use spaceport_settings::corners;
use spaceport_widgets::direction::{self, Logical};

/// The panel the app lays a page's own body in: where it sits and how big it
/// is, before the page's content, its cover and its shield go in.
pub fn panel<K>(hosts: &Hosts<'_, K>, page: Page, layer: &Layer) -> Stateful<Div> {
    let panel = panel_anchored(hosts, page, layer);
    let panel = panel_sized(hosts, page, layer, panel);
    panel_surface(hosts, page, layer, panel)
}

/// Where the panel hangs and how much room it may take: a popover beside what
/// opened it, a footer panel above the foot, or a dialog centred.
fn panel_anchored<K>(hosts: &Hosts<'_, K>, page: Page, layer: &Layer) -> Stateful<Div> {
    let chrome = hosts.metrics.chrome;
    let anchor = hosts.anchor;
    div()
        .id("menu-panel")
        .debug_selector(|| "menu-panel".into())
        .when(matches!(page, Page::Workspace | Page::Dialog(_)), |panel| {
            panel
                .w(layer
                    .popover
                    .unwrap_or(px(420.))
                    .min((layer.viewport.width - px(24.)).max(px(0.))))
                // Every dialog may use the window's height: a captioned form
                // whose buttons need scrolling into view reads as clipped.
                .max_h((layer.viewport.height - px(24.)).max(px(0.)))
                // Lift the popup off the terminal behind it, as the pickers do.
                .shadow_lg()
        })
        // All Devices rises out of the footer that opened it, so a box in
        // its place clips it on the way; see the end of this function.
        .when(layer.footer_anchored && !layer.rises, |panel| {
            anchor_footer_panel(panel, page, anchor, layer.viewport, chrome, layer.lift)
        })
        .when(layer.rises, |panel| {
            let (room, above) = footer_room(anchor, layer.viewport, chrome);
            let below = 1. - layer.shown;
            // Its own height, as the last frame laid it out; the box it
            // rises in takes its height from it, so a share of that box
            // would be a share of nothing. Until a frame has measured it,
            // the room it may take keeps it out of sight.
            let height = hosts.appear.rise_height.get().map_or(room, px);
            let offset = height * below;
            let measured = hosts.appear.rise_height.clone();
            panel
                .w_full()
                .max_h(room)
                .top(if above { offset } else { -offset })
                .child(
                    canvas(
                        move |bounds, _, _| measured.set(Some(bounds.size.height.into())),
                        |_, _, _, _| {},
                    )
                    .absolute()
                    .inset_0(),
                )
        })
        .when(matches!(page, Page::Usage(_)), |panel| {
            // Rises from the status bar segment that opened it, kept inside
            // the window and clear of the titlebar.
            let chrome = px(chrome);
            let width = px(hosts.metrics.usage_width)
                .min((layer.viewport.width - px(2. * MENU_MARGIN)).max(px(0.)));
            // The segment's own x, whichever side the bar starts on.
            panel
                .absolute()
                .left(
                    anchor
                        .x
                        .min(layer.viewport.width - width - px(MENU_MARGIN))
                        .max(px(MENU_MARGIN)),
                )
                .bottom((layer.viewport.height - anchor.y).max(px(MENU_MARGIN)) - layer.lift)
                .w(width)
                .max_h((anchor.y - chrome - px(MENU_MARGIN)).max(px(60.)))
                .flex()
                .flex_col()
                .overflow_hidden()
                .shadow_lg()
        })
}

/// The width and height each kind of page takes.
fn panel_sized<K>(
    hosts: &Hosts<'_, K>,
    page: Page,
    layer: &Layer,
    panel: Stateful<Div>,
) -> Stateful<Div> {
    let size = hosts.text.size;
    panel
        .when(
            matches!(
                page,
                Page::Tab
                    | Page::CoreWorkspaceActions
                    | Page::RenameTab
                    | Page::MoveTab
                    | Page::Group
                    | Page::Pane
                    | Page::RenamePane
                    | Page::MovePane
                    | Page::PaneProcesses
                    | Page::KillProcesses
                    | Page::Host
                    | Page::RemoveDevice
                    | Page::RemoveWsl
            ),
            |panel| {
                let room = (layer.viewport.width - px(24.)).max(px(0.));
                // A list of actions takes its longest label's width, which
                // the UI font and size decide, so no fixed width fits all.
                let forwards = page == Page::Host && hosts.host_forwards;
                if matches!(page, Page::Tab | Page::Pane | Page::Group | Page::Host) && !forwards {
                    let least = if page == Page::Group { 240. } else { 180. };
                    return panel
                        .min_w(px(least).min(room))
                        .max_w(room)
                        .max_h((layer.viewport.height - px(24.)).max(px(0.)));
                }
                panel
                    .w(room.min(px(if page == Page::CoreWorkspaceActions {
                        240.
                    } else if forwards {
                        // Room for a forward's port, state, and actions.
                        260.
                    } else if page == Page::PaneProcesses {
                        // Name, command, pid, CPU and memory columns.
                        560.
                    } else if matches!(page, Page::MovePane | Page::MoveTab) {
                        240.
                    } else {
                        360.
                    })))
                    .max_h((layer.viewport.height - px(24.)).max(px(0.)))
            },
        )
        .when(
            !layer.footer_anchored
                && !matches!(page, Page::Usage(_))
                && !layer.pointer_anchored
                && !matches!(page, Page::Dialog(_)),
            |panel| {
                panel
                    .w((layer.viewport.width - px(32.))
                        .max(px(0.))
                        .min(px(if layer.add_device {
                            ADD_DEVICE_WIDTH * (size / 12.)
                        } else if page == Page::Palette && hosts.palette_previews {
                            // Room for Go To's list beside its preview.
                            1120. * (size / 12.)
                        } else if page == Page::Palette {
                            // Wide like Raycast, so the query leads the panel.
                            720. * (size / 12.)
                        } else {
                            480.
                        })))
                    .max_h((layer.viewport.height - px(32.)).max(px(0.)))
            },
        )
        .when(
            !matches!(
                page,
                Page::Keybinds
                    | Page::Themes
                    | Page::DeviceTheme
                    | Page::Palette
                    | Page::AddDevice
                    | Page::AddWsl
                    | Page::Usage(_)
                    | Page::RenameDevice
                    | Page::ForwardPort
            ),
            |panel| {
                // Dialogs draw their own full-bleed header and footer rules,
                // so the panel's own inset would cut those rules short.
                // The device and session pickers scroll their own lists, so
                // the panel around them stays put; a second scroller there
                // still had its padding to move through. A session create or
                // delete form has no inner scroller and keeps the panel's.
                let lists = page == Page::Devices && !layer.session_modal;
                panel
                    .when(!lists, |panel| panel.overflow_y_scroll())
                    .when(lists, |panel| panel.overflow_hidden())
                    .when(!matches!(page, Page::Dialog(_)), |panel| panel.p(px(6.)))
            },
        )
        .when(
            matches!(
                page,
                Page::Keybinds | Page::Themes | Page::DeviceTheme | Page::Palette
            ),
            |panel| {
                panel
                    .flex()
                    .flex_col()
                    .h(px(560. * (size / 12.)).min((layer.viewport.height - px(32.)).max(px(0.))))
                    .overflow_hidden()
                    .shadow_lg()
            },
        )
        .when(layer.add_device, |panel| {
            panel.h(px(ADD_DEVICE_HEIGHT * (size / 12.))
                .min((layer.viewport.height - px(32.)).max(px(0.))))
        })
        .when(
            matches!(
                page,
                Page::AgentSkill | Page::VersionMismatch | Page::OpenLink
            ),
            |panel| {
                panel
                    .w((layer.viewport.width - px(24.)).max(px(0.)).min(px(420.)))
                    .max_h((layer.viewport.height - px(24.)).max(px(0.)))
            },
        )
        .when(
            matches!(
                page,
                Page::AddDevice | Page::AddWsl | Page::RenameDevice | Page::ForwardPort
            ),
            |panel| panel.flex().flex_col().overflow_hidden().shadow_lg(),
        )
        .when(page == Page::About, |panel| {
            // The About box is a release file: its longest row (the notices
            // address) must fit on one line, so the cap is the row's width
            // and not the popover's usual 340 px.
            panel.w((layer.viewport.width - px(24.)).max(px(0.)).min(px(480.)))
        })
}

/// The panel's own surface, over whatever it holds.
fn panel_surface<K>(
    hosts: &Hosts<'_, K>,
    page: Page,
    layer: &Layer,
    panel: Stateful<Div>,
) -> Stateful<Div> {
    let theme = hosts.theme;
    let text = &hosts.text;
    let panel = panel
        .rounded(px(corners::PANEL))
        .border_1()
        .border_color(rgb(theme.active))
        .bg(rgb(theme.surface))
        .text_color(rgb(theme.foreground));
    super::text_font(panel, text)
        .text_size(px(text.size))
        .line_height(px(text.line_height))
        .when(!layer.rises, |panel| panel.opacity(layer.shown))
        // A popover drops from the pointer that opened it; a dialog rises.
        // Footer and usage panels have slid already, by their own edge.
        .when(
            !layer.footer_anchored && !matches!(page, Page::Usage(_)),
            |panel| panel.top(if layer.dims { layer.lift } else { -layer.lift }),
        )
        .occlude()
        .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
        .on_mouse_down(MouseButton::Right, |_, _, cx| cx.stop_propagation())
        .on_click(|_, _, cx| cx.stop_propagation())
}

/// Where the panel lands in the window.
pub fn placement<K>(
    hosts: &Hosts<'_, K>,
    page: Page,
    layer: &Layer,
    panel: Stateful<Div>,
) -> AnyElement {
    if layer.pointer_anchored {
        let position = hosts.anchor;
        anchored()
            .position(position)
            // A menu hangs toward the end from the pointer, and toward
            // the start from the "…" at a strip's right end, which
            // stays there because strips never mirror.
            .anchor(match page {
                Page::Group | Page::CoreWorkspaceActions => Anchor::TopRight,
                _ => direction::anchor(Anchor::TopLeft),
            })
            .snap_to_window_with_margin(Edges::all(px(12.)))
            .child(panel)
            .into_any_element()
    } else if layer.rises {
        anchor_footer_panel(
            div().id("menu-rise").overflow_hidden(),
            page,
            hosts.anchor,
            layer.viewport,
            hosts.metrics.chrome,
            px(0.),
        )
        .child(panel)
        .into_any_element()
    } else {
        panel.into_any_element()
    }
}

/// The page as it goes: it takes no clicks, so what they would act on has
/// already closed, and it is the last thing it paints.
pub fn leaving(layer: &Layer, placed: AnyElement) -> Stateful<Div> {
    div()
        .id("menu-leaving")
        .absolute()
        .inset_0()
        .when(layer.dims, |layer| {
            layer.row().items_center().justify_center()
        })
        .child(placed)
}

/// Places a footer menu beside its anchor, `lift` short of it while it slides
/// into place.
pub fn anchor_footer_panel(
    panel: Stateful<Div>,
    page: Page,
    anchor: Point<Pixels>,
    viewport: Size<Pixels>,
    chrome: f32,
    lift: Pixels,
) -> Stateful<Div> {
    let (room, above) = footer_room(anchor, viewport, chrome);
    let list = page == Page::Devices;
    let width = if list { DEVICES_MENU_WIDTH } else { 180. };
    // From the window's start edge, which right to left is its right.
    let start = match direction::is_rtl() {
        false => anchor.x,
        true => viewport.width - anchor.x,
    };
    let start = match page {
        Page::Devices => start,
        _ => px(56.),
    };
    let panel = panel
        .absolute()
        .inset_s(start)
        .w(px(width).min((viewport.width - px(16.)).max(px(0.))));
    if above {
        let gap = match list {
            true => DEVICES_MENU_GAP,
            false => 12.,
        };
        panel
            .bottom((viewport.height - anchor.y + px(gap)).max(px(MENU_MARGIN)) - lift)
            .max_h(room)
    } else {
        panel.top(anchor.y + px(12.) - lift).max_h(room)
    }
}
