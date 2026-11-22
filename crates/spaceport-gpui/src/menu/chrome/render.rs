//! Drawing the popup layer: where its panel sits, what page it draws, and
//! the overlay that keeps the window behind it inert.

#[cfg(any(windows, test, feature = "integration-test"))]
use super::COVER_MARGIN;
use super::{MENU_MARGIN, Page, rail};
use crate::direction::Logical;
use crate::{SpaceportWindow, actions, fonts::StyledFont};
use gpui::{prelude::*, *};
use std::time::Instant;

impl SpaceportWindow {
    /// Draws `page`: the open menu, or with `leaving` the one that just
    /// closed, still showing that much of itself as it goes.
    pub(in crate::menu) fn render_menu_layer(
        &self,
        page: Page,
        leaving: Option<f32>,
        window: &Window,
        cx: &mut Context<Self>,
    ) -> Stateful<Div> {
        let layer = self.menu_layer(page, leaving, window);
        let panel = self.menu_panel(page, &layer, cx);
        let placed = self.menu_placement(page, &layer, panel);
        // On its way out it takes no clicks: what they would act on has
        // already closed, and it is the last thing it paints.
        if layer.leaving.is_some() {
            return div()
                .id("menu-leaving")
                .absolute()
                .inset_0()
                .when(layer.dims, |layer| {
                    layer.row().items_center().justify_center()
                })
                .child(placed);
        }
        self.menu_overlay(page, &layer, placed, cx)
    }

    /// What the popup measured before it draws.
    fn menu_layer(&self, page: Page, leaving: Option<f32>, window: &Window) -> Layer {
        let font = &self.config.ui;
        let viewport = window.viewport_size();
        let session_modal = page == Page::Devices && self.menu.session_edit.is_some();
        // Add Device's sources share one wide dialog; a narrow window keeps
        // the rail to icons so the form still has room.
        let add_device = rail::Source::of(page).is_some();
        let compact_rail = viewport.width - px(32.) < px(rail::COMPACT_BELOW * (font.size / 12.));
        let footer_anchored = matches!(page, Page::Menu | Page::Devices) && !session_modal;
        // Context menus open where the pointer asked for them. A dialog is a
        // modal decision, not a continuation of the row it came from, so it
        // centres over a dimmed window the way the Herdr TUI's dialogs do.
        let pointer_anchored = page.pointer_anchored();
        let dims = self.menu_dims();
        let now = crate::motion::now();
        let shown = leaving.unwrap_or_else(|| self.menu.appear.panel(now));
        let lift = crate::menu::appear::Appear::offset(dims, shown);
        let rises = page == Page::Devices && !session_modal;
        // The worktree popover widens for captions a language makes longer.
        let popover = (page == Page::Workspace)
            .then(|| self.workspace_popover_width(window, (viewport.width - px(24.)).max(px(0.))));
        Layer {
            add_device,
            compact_rail,
            dims,
            footer_anchored,
            leaving,
            lift,
            now,
            pointer_anchored,
            popover,
            rises,
            session_modal,
            shown,
            viewport,
        }
    }

    /// The panel itself: where it sits, how big it is, what page it draws, and
    /// the cover and shield the frame around it needs.
    fn menu_panel(&self, page: Page, layer: &Layer, cx: &mut Context<Self>) -> Stateful<Div> {
        let panel = self.menu_panel_anchored(page, layer);
        let panel = self.menu_panel_sized(page, layer, panel);
        let panel = self.menu_panel_surface(page, layer, panel, cx);
        let panel = self.menu_page_body(page, layer, panel, cx);
        // Pages sit above everything GPUI draws on Windows, so only there
        // does the menu have to say what it covers.
        #[cfg(any(windows, test, feature = "integration-test"))]
        let panel = self.menu_panel_cover(layer, panel);
        self.menu_panel_shield(layer, panel)
    }

    /// Where the panel hangs and how much room it may take: a popover beside
    /// what opened it, a footer panel above the foot, or a dialog centred.
    fn menu_panel_anchored(&self, page: Page, layer: &Layer) -> Stateful<Div> {
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
                self.anchor_footer_panel(panel, layer.viewport, page, layer.lift)
            })
            .when(layer.rises, |panel| {
                let (room, above) = self.footer_room(layer.viewport);
                let below = 1. - layer.shown;
                // Its own height, as the last frame laid it out; the box it
                // rises in takes its height from it, so a share of that box
                // would be a share of nothing. Until a frame has measured it,
                // the room it may take keeps it out of sight.
                let height = self.menu.appear.rise_height.get().unwrap_or(room);
                let offset = height * below;
                let measured = self.menu.appear.rise_height.clone();
                panel
                    .w_full()
                    .max_h(room)
                    .top(if above { offset } else { -offset })
                    .child(
                        canvas(
                            move |bounds, _, _| measured.set(Some(bounds.size.height)),
                            |_, _, _, _| {},
                        )
                        .absolute()
                        .inset_0(),
                    )
            })
            .when(matches!(page, Page::Usage(_)), |panel| {
                // Rises from the status bar segment that opened it, kept inside
                // the window and clear of the titlebar.
                let chrome = px(crate::titlebar::HEIGHT);
                let width = px(crate::usage::PANEL_WIDTH)
                    .min((layer.viewport.width - px(2. * MENU_MARGIN)).max(px(0.)));
                // The segment's own x, whichever side the bar starts on.
                panel
                    .absolute()
                    .left(
                        self.menu
                            .anchor
                            .x
                            .min(layer.viewport.width - width - px(MENU_MARGIN))
                            .max(px(MENU_MARGIN)),
                    )
                    .bottom(
                        (layer.viewport.height - self.menu.anchor.y).max(px(MENU_MARGIN))
                            - layer.lift,
                    )
                    .w(width)
                    .max_h((self.menu.anchor.y - chrome - px(MENU_MARGIN)).max(px(60.)))
                    .flex()
                    .flex_col()
                    .overflow_hidden()
                    .shadow_lg()
            })
    }

    /// The width and height each kind of page takes.
    fn menu_panel_sized(&self, page: Page, layer: &Layer, panel: Stateful<Div>) -> Stateful<Div> {
        let font = &self.config.ui;
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
                    let forwards = page == Page::Host && self.host_menu_lists_forwards();
                    if matches!(page, Page::Tab | Page::Pane | Page::Group | Page::Host)
                        && !forwards
                    {
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
                        .w((layer.viewport.width - px(32.)).max(px(0.)).min(px(
                            if layer.add_device {
                                rail::WIDTH * (font.size / 12.)
                            } else if page == Page::Palette && self.palette_previews(layer.viewport)
                            {
                                // Room for Go To's list beside its preview.
                                1120. * (font.size / 12.)
                            } else if page == Page::Palette {
                                // Wide like Raycast, so the query leads the panel.
                                720. * (font.size / 12.)
                            } else {
                                480.
                            },
                        )))
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
                        .h(px(560. * (font.size / 12.))
                            .min((layer.viewport.height - px(32.)).max(px(0.))))
                        .overflow_hidden()
                        .shadow_lg()
                },
            )
            .when(layer.add_device, |panel| {
                panel.h(px(rail::HEIGHT * (font.size / 12.))
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
                panel.w((layer.viewport.width - px(24.)).max(px(0.)).min(px(340.)))
            })
    }

    /// The panel's own surface, over whatever it holds.
    fn menu_panel_surface(
        &self,
        page: Page,
        layer: &Layer,
        panel: Stateful<Div>,
        cx: &mut Context<Self>,
    ) -> Stateful<Div> {
        let theme = &self.theme;
        let font = &self.config.ui;
        panel
            .rounded(px(crate::config::corners::PANEL))
            .border_1()
            .border_color(rgb(theme.active))
            .bg(rgb(theme.surface))
            .text_color(rgb(theme.foreground))
            .text_font(font)
            .text_size(px(font.size))
            .line_height(px(font.line_height()))
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
            // A menu the pointer opened follows the pointer's own report of
            // whether it is over the popup, which occlusion and snapping make
            // impossible to infer from the anchor alone.
            .when(self.hover_menu.is_some(), |panel| {
                panel.on_hover(cx.listener(|this, hovered: &bool, _, _| {
                    if let Some(open) = &mut this.hover_menu {
                        open.inside = *hovered;
                    }
                }))
            })
    }

    /// The page the menu is on, under the panel's own padding.
    fn menu_page_body(
        &self,
        page: Page,
        layer: &Layer,
        panel: Stateful<Div>,
        cx: &mut Context<Self>,
    ) -> Stateful<Div> {
        let mut panel = panel;
        if page == Page::Menu {
            return self.menu_item_list(panel, cx);
        }
        if page == Page::Devices {
            panel = match self.menu.session_edit {
                Some(_) => panel.child(self.render_session_edit(cx)),
                None => panel.child(self.render_devices(cx)),
            };
        } else if let Page::Usage(provider) = page {
            panel = panel.child(self.render_usage_panel(provider, cx));
        } else if page == Page::AddDevice {
            panel = panel.child(self.render_add_device(layer.compact_rail, cx));
        } else if page == Page::AddWsl {
            panel = panel.child(self.render_add_wsl(layer.compact_rail, cx));
        } else if page == Page::RemoveWsl {
            panel = panel.child(self.render_remove_wsl(cx));
        } else if page == Page::Workspace {
            panel = panel.child(self.render_workspace_popover(cx));
        } else if let Page::Dialog(action) = page {
            panel = panel.child(self.render_workspace_dialog(action, cx));
        } else if page == Page::CoreWorkspaceActions {
            panel = panel.child(self.render_core_workspace_actions(cx));
        } else if matches!(
            page,
            Page::CoreWorkspaceCreate | Page::CoreWorkspaceDelete | Page::CoreWorkspaceRename
        ) {
            panel = panel.child(self.render_core_workspace_dialog(cx));
        } else if matches!(
            page,
            Page::Host
                | Page::RenameDevice
                | Page::ForwardPort
                | Page::DeviceTags
                | Page::DeviceTheme
                | Page::RemoveDevice
        ) {
            panel = panel.child(self.render_host_menu(cx));
        } else if page == Page::MoveTab {
            panel = self.render_tab_move_picker(panel, cx);
        } else if matches!(page, Page::Tab | Page::RenameTab) {
            panel = panel.child(self.render_tab_menu(cx));
        } else if page == Page::Group {
            panel = self.render_group_menu(panel, cx);
        } else if page == Page::MovePane {
            panel = self.render_move_picker(panel, cx);
        } else if matches!(
            page,
            Page::Pane | Page::RenamePane | Page::PaneProcesses | Page::KillProcesses
        ) {
            panel = panel.child(self.render_pane_menu(cx));
        } else if page == Page::Keybinds {
            panel = panel.child(self.render_keybinds(cx));
        } else if page == Page::Themes {
            panel = panel.child(self.render_theme_picker(cx));
        } else if page == Page::Palette {
            panel = panel.child(self.render_palette(layer.viewport, cx));
        } else if page == Page::ConfirmClose {
            panel = panel.child(self.render_close_confirmation(cx));
        } else if page == Page::About {
            panel = panel.child(self.render_about(cx));
        } else if page == Page::AgentSkill {
            panel = panel.child(self.render_agent_skill_offer(cx));
        } else if page == Page::VersionMismatch {
            panel = panel.child(self.render_version_mismatch(cx));
        } else if page == Page::OpenLink {
            panel = panel.child(self.render_open_link(cx));
        } else {
            // Exhaustive, so a new page fails to compile here until it is
            // drawn: by a branch above, or by an arm of its own.
            match page {
                Page::Update => panel = panel.child(self.render_release_notes(cx)),
                // `render_menu` draws the application menu before this layer.
                Page::Application => {}
                // Drawn by the branches above.
                Page::About
                | Page::AddDevice
                | Page::AddWsl
                | Page::AgentSkill
                | Page::ConfirmClose
                | Page::CoreWorkspaceActions
                | Page::CoreWorkspaceCreate
                | Page::CoreWorkspaceDelete
                | Page::CoreWorkspaceRename
                | Page::Devices
                | Page::DeviceTags
                | Page::DeviceTheme
                | Page::Dialog(_)
                | Page::ForwardPort
                | Page::Group
                | Page::Host
                | Page::Keybinds
                | Page::KillProcesses
                | Page::Menu
                | Page::MovePane
                | Page::MoveTab
                | Page::OpenLink
                | Page::Palette
                | Page::Pane
                | Page::PaneProcesses
                | Page::RemoveDevice
                | Page::RemoveWsl
                | Page::RenameDevice
                | Page::RenamePane
                | Page::RenameTab
                | Page::Tab
                | Page::Themes
                | Page::Usage(_)
                | Page::VersionMismatch
                | Page::Workspace => {}
            }
        }
        panel
    }

    /// What the menu covers, for the Windows pages that hide under it: a
    /// dimmed dialog covers the window, a popover its panel. Tests and native
    /// checks read it to find the menu.
    #[cfg(any(windows, test, feature = "integration-test"))]
    fn menu_panel_cover(&self, layer: &Layer, panel: Stateful<Div>) -> Stateful<Div> {
        let cover = self.menu.cover.clone();
        if layer.dims {
            cover.set(crate::menu::state::Cover::All);
        }
        // A menu on its way out no longer covers anything.
        panel.when(!layer.dims && layer.leaving.is_none(), |panel| {
            panel.child(
                canvas(
                    // Laid out inside the panel's border; the margin takes
                    // in the border and the start of its shadow.
                    move |bounds, _, _| {
                        cover.set(crate::menu::state::Cover::Panel(
                            bounds.dilate(px(COVER_MARGIN)),
                        ))
                    },
                    |_, _, _, _| {},
                )
                .absolute()
                .inset_0(),
            )
        })
    }

    /// A menu on its way out takes no clicks, so it asks for none.
    fn menu_panel_shield(&self, layer: &Layer, panel: Stateful<Div>) -> Stateful<Div> {
        // On its way out it takes no clicks: what they would act on has
        // already closed.
        panel.when(layer.leaving.is_some(), |panel| {
            panel.child(
                div()
                    .id("menu-leaving-shield")
                    .debug_selector(|| "menu-leaving-shield".into())
                    .absolute()
                    .inset_0()
                    .occlude(),
            )
        })
    }

    /// The application menu's own rows, each an item the window runs.
    fn menu_item_list(&self, panel: Stateful<Div>, cx: &mut Context<Self>) -> Stateful<Div> {
        let font = &self.config.ui;
        let theme = &self.theme;
        let mut panel = panel;
        let items = self.menu_items();
        for (index, item) in items.into_iter().enumerate() {
            panel = panel.child(
                div()
                    .id(item.selector())
                    .debug_selector(move || format!("menu-{}", item.selector()))
                    .role(Role::MenuItem)
                    .aria_label(item.label())
                    .when(Some(index) == self.menu.selected, |row| {
                        row.aria_active_descendant()
                    })
                    .min_h(px(font.line_height() + 12.))
                    .px(px(8.))
                    .row()
                    .items_center()
                    .cursor_pointer()
                    .when(Some(index) == self.menu.selected, |row| {
                        row.bg(rgb(theme.active))
                    })
                    .on_hover(cx.listener(move |this, hovered, _, cx| {
                        if *hovered {
                            this.menu.selected = Some(index);
                        } else if this.menu.selected == Some(index) {
                            this.menu.selected = None;
                        }
                        cx.notify();
                    }))
                    .child(item.label())
                    .on_click(cx.listener(move |this, _, window, cx| {
                        cx.stop_propagation();
                        this.activate_menu(item, window, cx);
                    })),
            );
        }
        panel
    }

    /// Where the panel lands in the window.
    fn menu_placement(&self, page: Page, layer: &Layer, panel: Stateful<Div>) -> AnyElement {
        if layer.pointer_anchored {
            let position = self.menu.anchor;
            anchored()
                .position(position)
                // A menu hangs toward the end from the pointer, and toward
                // the start from the "…" at a strip's right end, which
                // stays there because strips never mirror.
                .anchor(match page {
                    Page::Group | Page::CoreWorkspaceActions => Anchor::TopRight,
                    _ => crate::direction::anchor(Anchor::TopLeft),
                })
                .snap_to_window_with_margin(Edges::all(px(12.)))
                .child(panel)
                .into_any_element()
        } else if layer.rises {
            self.anchor_footer_panel(
                div().id("menu-rise").overflow_hidden(),
                layer.viewport,
                page,
                px(0.),
            )
            .child(panel)
            .into_any_element()
        } else {
            panel.into_any_element()
        }
    }

    /// The layer that holds the panel, keeps the keyboard, and dismisses the
    /// menu on a click or a right click outside it.
    fn menu_overlay(
        &self,
        page: Page,
        layer: &Layer,
        placed: AnyElement,
        cx: &mut Context<Self>,
    ) -> Stateful<Div> {
        let theme = &self.theme;
        div()
            .id("menu-overlay")
            .debug_selector(|| "menu-overlay".into())
            // Focus stays here while the popup is open, so this is the menu
            // or dialog a screen reader announces.
            .role(page.accessible_role())
            .aria_label(page.accessible_name())
            .absolute()
            .top_0()
            .inset_s(px(0.))
            .bottom_0()
            .inset_e(px(0.))
            .when(layer.dims, |overlay| {
                overlay
                    .row()
                    .items_center()
                    .justify_center()
                    .bg(crate::menu::appear::backdrop(
                        theme.background,
                        self.menu.appear.backdrop(layer.now),
                    ))
            })
            .occlude()
            .track_focus(&self.menu.focus)
            .on_action(
                cx.listener(|this, _: &actions::Cut, window, cx| this.menu_edit("x", window, cx)),
            )
            .on_action(
                cx.listener(|this, _: &actions::Copy, window, cx| this.menu_edit("c", window, cx)),
            )
            .on_action(
                cx.listener(|this, _: &actions::Paste, window, cx| this.menu_edit("v", window, cx)),
            )
            .on_action(cx.listener(|this, _: &actions::SelectAll, window, cx| {
                this.menu_edit("a", window, cx)
            }))
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(|this, _, window, cx| {
                    cx.stop_propagation();
                    this.dismiss_menu(window, cx);
                }),
            )
            .on_scroll(|_, _, cx| cx.stop_propagation())
            .on_mouse_down(
                MouseButton::Right,
                cx.listener(|this, _, window, cx| {
                    if this.menu.opening_right_click {
                        cx.stop_propagation();
                        return;
                    }
                    // Only workspace rows may retarget this gesture. The overlay
                    // still occludes ordinary terminal and chrome handlers.
                    if this.menu.page != Some(Page::Workspace) {
                        cx.stop_propagation();
                    }
                    this.dismiss_menu(window, cx);
                }),
            )
            .on_key_down(cx.listener(Self::menu_key_down))
            .child(placed)
    }
}

/// What the popup measured once, before it draws: where it hangs, how big the
/// window is, and how far it has come in.
struct Layer {
    add_device: bool,
    compact_rail: bool,
    dims: bool,
    footer_anchored: bool,
    leaving: Option<f32>,
    lift: Pixels,
    now: Instant,
    pointer_anchored: bool,
    popover: Option<Pixels>,
    rises: bool,
    session_modal: bool,
    shown: f32,
    viewport: Size<Pixels>,
}
