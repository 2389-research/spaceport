// ABOUTME: Paints popup shells with shared active and departure geometry.
// ABOUTME: Keeps input wiring separate from each popup's prepared content.
#[cfg(any(windows, test, feature = "integration-test"))]
use crate::menu::Cover;
use crate::menu::{
    MENU_MARGIN, Page, WorkspaceAction,
    appear::{self, Appear},
    devices::{self, rail},
};
use crate::{SheprdWindow, actions, direction::Logical, fonts::StyledFont};
use gpui::{prelude::*, *};

#[cfg(any(windows, test, feature = "integration-test"))]
const COVER_MARGIN: f32 = 8.;

fn page_dims(page: Page, session_modal: bool) -> bool {
    let footer_anchored = matches!(page, Page::Menu | Page::Devices) && !session_modal;
    !footer_anchored
        && !matches!(page, Page::Usage(_) | Page::Application)
        && !page.pointer_anchored()
}

fn shared_shell_scroll(page: Page) -> bool {
    matches!(
        page,
        Page::About
            | Page::AgentSkill
            | Page::ConfirmClose
            | Page::Menu
            | Page::Messages
            | Page::OpenLink
            | Page::Update
            | Page::VersionMismatch
    )
}

impl SheprdWindow {
    pub(in crate::menu) fn menu_shell_offset(&self) -> Option<Pixels> {
        self.menu
            .page
            .filter(|page| shared_shell_scroll(*page))
            .map(|_| self.menu.shell_scroll.offset().y)
    }

    pub(crate) fn render_menu(&self, window: &Window, cx: &mut Context<Self>) -> Stateful<Div> {
        if self.menu.page == Some(Page::Application) {
            let Some(paint) = self.application_menu_paint(window, cx) else {
                return div().id("application-menu-overlay");
            };
            let shown = self.menu.appear.panel(crate::motion::now());
            return self.render_application_menu_layer(
                &paint,
                true,
                (shown, Appear::offset(false, shown)),
                window,
                cx,
            );
        }
        if self.menu.page == Some(Page::Devices) && self.menu.session_edit.is_some() {
            let picker = self.render_session_picker_underlay(None, window, cx);
            // The picker remains visible in its original position beneath the
            // modal's dimmed, input-occluding layer. Only that top layer owns focus.
            return div()
                .id("session-menu-stack")
                .absolute()
                .inset_0()
                .child(picker)
                .child(self.render_menu_layer(
                    self.menu.page.unwrap_or(Page::Menu),
                    None,
                    None,
                    window,
                    cx,
                ));
        }
        self.render_menu_layer(self.menu.page.unwrap_or(Page::Menu), None, None, window, cx)
    }

    pub(in crate::menu) fn render_session_picker_underlay(
        &self,
        paint: Option<&devices::DevicesPaint>,
        window: &Window,
        cx: &mut Context<Self>,
    ) -> Stateful<Div> {
        let theme = &self.theme;
        let font = &self.config.ui;
        self.anchor_footer_panel(
            div()
                .id("session-picker-underlay")
                .debug_selector(|| "session-picker-underlay".into()),
            window.viewport_size(),
            Page::Devices,
            px(0.),
        )
        .when(paint.is_none(), |panel| panel.overflow_y_scroll())
        .when(paint.is_some(), |panel| panel.overflow_hidden())
        .p(px(6.))
        .rounded(px(crate::config::corners::PANEL))
        .border_1()
        .border_color(rgb(theme.active))
        .bg(rgb(theme.surface))
        .text_color(rgb(theme.foreground))
        .text_font(font)
        .text_size(px(font.size))
        .line_height(px(font.line_height()))
        .child(match paint {
            Some(paint) => self.render_devices_paint(paint, cx),
            None => self.render_devices(cx),
        })
    }

    /// Places a footer menu beside its anchor, `lift` short of it while it
    /// slides into place.
    fn anchor_footer_panel(
        &self,
        panel: Stateful<Div>,
        viewport: Size<Pixels>,
        page: Page,
        lift: Pixels,
    ) -> Stateful<Div> {
        let (room, above) = self.footer_room(viewport);
        let list = page == Page::Devices;
        let width = if list { devices::MENU_WIDTH } else { 180. };
        // From the window's start edge, which right to left is its right.
        let anchor = match crate::direction::is_rtl() {
            false => self.menu.anchor.x,
            true => viewport.width - self.menu.anchor.x,
        };
        let start = match page {
            Page::Devices => anchor,
            _ => px(56.),
        };
        let panel = panel
            .absolute()
            .inset_s(start)
            .w(px(width).min((viewport.width - px(16.)).max(px(0.))));
        if above {
            panel
                .bottom(
                    (viewport.height - self.menu.anchor.y
                        + px(if list { devices::MENU_GAP } else { 12. }))
                    .max(px(MENU_MARGIN))
                        - lift,
                )
                .max_h(room)
        } else {
            panel.top(self.menu.anchor.y + px(12.) - lift).max_h(room)
        }
    }

    /// The room a footer menu has, and whether it opens above its anchor,
    /// as it does from the sidebar footer at the window's foot.
    fn footer_room(&self, viewport: Size<Pixels>) -> (Pixels, bool) {
        let chrome = px(crate::titlebar::HEIGHT);
        let band = (viewport.height - chrome - px(2. * MENU_MARGIN)).max(px(60.));
        let room = |side: Pixels| side.clamp(px(0.), band).max(px(60.)).min(band);
        let above = room(self.menu.anchor.y - px(12. + MENU_MARGIN) - chrome);
        let below = room(viewport.height - self.menu.anchor.y - px(12. + MENU_MARGIN));
        if above >= below {
            (above, true)
        } else {
            (below, false)
        }
    }

    /// Whether the open menu is a dialog that dims the window behind it,
    /// rather than a popover beside what opened it.
    pub(crate) fn menu_dims(&self) -> bool {
        let Some(page) = self.menu.page else {
            return false;
        };
        let session_modal = page == Page::Devices && self.menu.session_edit.is_some();
        page_dims(page, session_modal)
    }

    /// Draws `page`: the open menu, or with `leaving` the one that just
    /// closed, still showing that much of itself as it goes.
    pub(in crate::menu) fn render_menu_layer(
        &self,
        page: Page,
        leaving: Option<f32>,
        paint: Option<&appear::Kept>,
        window: &Window,
        cx: &mut Context<Self>,
    ) -> Stateful<Div> {
        if leaving.is_some() && !paint.is_some_and(|paint| paint.holds(page)) {
            return div().id("menu-panel");
        }
        #[cfg(all(windows, feature = "integration-test"))]
        if let Some(shown) = leaving {
            self.menu.appear.drawn_departure.set(Some((page, shown)));
        }
        let font = &self.config.ui;
        let theme = &self.theme;
        let viewport = window.viewport_size();
        let session_modal = page == Page::Devices
            && paint.and_then(|paint| paint.devices.as_ref()).map_or_else(
                || self.menu.session_edit.is_some(),
                |paint| paint.session_modal(),
            );
        // Add Device's sources share one wide dialog; a narrow window keeps
        // the rail to icons so the form still has room.
        let add_device = rail::Source::of(page).is_some();
        let compact_rail = viewport.width - px(32.) < px(rail::COMPACT_BELOW * (font.size / 12.));
        let footer_anchored = matches!(page, Page::Menu | Page::Devices) && !session_modal;
        let settled = page == Page::Dialog(WorkspaceAction::OpenWorktree);
        // Context menus open where the pointer asked for them. A dialog is a
        // modal decision, not a continuation of the row it came from, so it
        // centres over a dimmed window the way the Herdr TUI's dialogs do.
        let pointer_anchored = page.pointer_anchored();
        let dims = page_dims(page, session_modal);
        let now = crate::motion::now();
        let shown = leaving.unwrap_or_else(|| self.menu.appear.panel(now));
        let shell_offset = paint.and_then(|paint| paint.shell_offset).unwrap_or(px(0.));
        let lift = Appear::offset(dims, shown);
        let rises = page == Page::Devices && !session_modal;
        // The worktree popover widens for captions a language makes longer.
        let popover = (page == Page::Workspace).then(|| {
            let room = (viewport.width - px(24.)).max(px(0.));
            match paint.and_then(|paint| paint.workspace_popover.as_ref()) {
                Some(paint) => self.workspace_popover_paint_width(paint, window, room),
                None => self.workspace_popover_width(window, room),
            }
        });
        let mut panel = div()
            .id("menu-panel")
            .debug_selector(|| "menu-panel".into())
            .when(matches!(page, Page::Workspace | Page::Dialog(_)), |panel| {
                panel
                    .w(popover
                        .unwrap_or(px(if settled {
                            // A listing needs room for a title and its branch.
                            560.
                        } else {
                            420.
                        }))
                        .min((viewport.width - px(24.)).max(px(0.))))
                    // Every dialog may use the window's height: a captioned form
                    // whose buttons need scrolling into view reads as clipped.
                    .max_h((viewport.height - px(24.)).max(px(0.)))
                    // A listing is a picker: it takes a settled height and
                    // scrolls inside it, as the theme and command pickers do.
                    .when(settled, |panel| {
                        panel
                            .flex()
                            .flex_col()
                            .h(px(560. * (font.size / 12.))
                                .min((viewport.height - px(24.)).max(px(0.))))
                            .overflow_hidden()
                    })
                    // Lift the popup off the terminal behind it, as the pickers do.
                    .shadow_lg()
            })
            // All Devices rises out of the footer that opened it, so a box in
            // its place clips it on the way; see the end of this function.
            .when(footer_anchored && !rises, |panel| {
                self.anchor_footer_panel(panel, viewport, page, lift)
            })
            .when(rises, |panel| {
                let (room, above) = self.footer_room(viewport);
                let below = 1. - shown;
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
                    .min((viewport.width - px(2. * MENU_MARGIN)).max(px(0.)));
                // The segment's own x, whichever side the bar starts on.
                panel
                    .absolute()
                    .left(
                        self.menu
                            .anchor
                            .x
                            .min(viewport.width - width - px(MENU_MARGIN))
                            .max(px(MENU_MARGIN)),
                    )
                    .bottom((viewport.height - self.menu.anchor.y).max(px(MENU_MARGIN)) - lift)
                    .w(width)
                    .max_h((self.menu.anchor.y - chrome - px(MENU_MARGIN)).max(px(60.)))
                    .flex()
                    .flex_col()
                    .overflow_hidden()
                    .shadow_lg()
            })
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
                    let room = (viewport.width - px(24.)).max(px(0.));
                    // A list of actions takes its longest label's width, which
                    // the UI font and size decide, so no fixed width fits all.
                    let forwards = page == Page::Host
                        && paint.and_then(|paint| paint.host.as_ref()).map_or_else(
                            || self.host_menu_lists_forwards(),
                            |paint| paint.forwards(),
                        );
                    if matches!(page, Page::Tab | Page::Pane | Page::Group | Page::Host)
                        && !forwards
                    {
                        let least = if page == Page::Group { 240. } else { 180. };
                        return panel
                            .min_w(px(least).min(room))
                            .max_w(room)
                            .max_h((viewport.height - px(24.)).max(px(0.)));
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
                        .max_h((viewport.height - px(24.)).max(px(0.)))
                },
            )
            .when(
                !footer_anchored
                    && !matches!(page, Page::Usage(_))
                    && !pointer_anchored
                    && !matches!(page, Page::Dialog(_)),
                |panel| {
                    panel
                        .w((viewport.width - px(32.))
                            .max(px(0.))
                            .min(px(if add_device {
                                rail::WIDTH * (font.size / 12.)
                            } else if page == Page::Palette && self.palette_previews(viewport) {
                                // Room for Go To's list beside its preview.
                                1120. * (font.size / 12.)
                            } else if page == Page::Palette {
                                // Wide like Raycast, so the query leads the panel.
                                720. * (font.size / 12.)
                            } else {
                                480.
                            })))
                        .max_h((viewport.height - px(32.)).max(px(0.)))
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
                    let lists = page == Page::Devices && !session_modal;
                    panel
                        .when(!settled && !lists, |panel| panel.overflow_y_scroll())
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
                            .min((viewport.height - px(32.)).max(px(0.))))
                        .overflow_hidden()
                        .shadow_lg()
                },
            )
            .when(add_device, |panel| {
                panel.h(px(rail::HEIGHT * (font.size / 12.))
                    .min((viewport.height - px(32.)).max(px(0.))))
            })
            .when(
                matches!(
                    page,
                    Page::AgentSkill | Page::VersionMismatch | Page::OpenLink
                ),
                |panel| {
                    panel
                        .w((viewport.width - px(24.)).max(px(0.)).min(px(420.)))
                        .max_h((viewport.height - px(24.)).max(px(0.)))
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
                panel.w((viewport.width - px(24.)).max(px(0.)).min(px(340.)))
            })
            .rounded(px(crate::config::corners::PANEL))
            .border_1()
            .border_color(rgb(theme.active))
            .bg(rgb(theme.surface))
            .text_color(rgb(theme.foreground))
            .text_font(font)
            .text_size(px(font.size))
            .line_height(px(font.line_height()))
            .when(!rises, |panel| panel.opacity(shown))
            // A popover drops from the pointer that opened it; a dialog rises.
            // Footer and usage panels have slid already, by their own edge.
            .when(
                !footer_anchored && !matches!(page, Page::Usage(_)),
                |panel| panel.top(if dims { lift } else { -lift }),
            )
            .when(leaving.is_none(), |panel| {
                panel
                    .occlude()
                    .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
                    .on_mouse_down(MouseButton::Right, |_, _, cx| cx.stop_propagation())
                    .on_click(|_, _, cx| cx.stop_propagation())
            })
            // A menu the pointer opened follows the pointer's own report of
            // whether it is over the popup, which occlusion and snapping make
            // impossible to infer from the anchor alone.
            .when(leaving.is_none() && self.hover_menu.is_some(), |panel| {
                panel.on_hover(cx.listener(|this, hovered: &bool, _, _| {
                    if let Some(open) = &mut this.hover_menu {
                        open.inside = *hovered;
                    }
                }))
            });
        if leaving.is_none() {
            if shared_shell_scroll(page) {
                panel = panel.track_scroll(&self.menu.shell_scroll);
            }
            if page == Page::Devices && session_modal {
                panel = panel.track_scroll(&self.menu.session_edit_scroll);
            }
            if page == Page::RemoveWsl {
                panel = panel.track_scroll(&self.menu.wsl_remove_scroll);
            }
            if matches!(page, Page::Workspace)
                || matches!(
                    page,
                    Page::CoreWorkspaceActions
                        | Page::CoreWorkspaceCreate
                        | Page::CoreWorkspaceRename
                        | Page::CoreWorkspaceDelete
                )
                || matches!(page, Page::Dialog(action) if action != WorkspaceAction::OpenWorktree)
            {
                panel = panel.track_scroll(&self.menu.workspace_scroll);
            }
            if let Some(scroll) = self.host_shell_scroll() {
                panel = panel.track_scroll(&scroll);
            }
            if let Some(scroll) = self.tab_shell_scroll() {
                panel = panel.track_scroll(&scroll);
            }
            if let Some(scroll) = self.pane_shell_scroll() {
                panel = panel.track_scroll(&scroll);
            }
        }
        if let Some(paint) = paint.and_then(|paint| paint.static_dialog.as_ref()) {
            let body = self.render_static_dialog_paint(paint, cx);
            panel = panel.child(if shell_offset == px(0.) {
                body
            } else {
                div()
                    .relative()
                    .top(shell_offset)
                    .child(body)
                    .into_any_element()
            });
        } else if page == Page::Menu {
            let (items, selected) = paint
                .and_then(|paint| paint.footer.clone())
                .unwrap_or_else(|| (self.menu_items(), self.menu.selected));
            for (index, item) in items.into_iter().enumerate() {
                panel = panel.child(
                    div()
                        .id(item.selector())
                        .debug_selector(move || format!("menu-{}", item.selector()))
                        .when(leaving.is_none(), |row| {
                            row.role(Role::MenuItem).aria_label(item.label())
                        })
                        .when(leaving.is_none() && Some(index) == selected, |row| {
                            row.aria_active_descendant()
                        })
                        .min_h(px(font.line_height() + 12.))
                        .when(leaving.is_some(), |row| row.relative().top(shell_offset))
                        .px(px(8.))
                        .row()
                        .items_center()
                        .when(leaving.is_none(), |row| row.cursor_pointer())
                        .when(Some(index) == selected, |row| row.bg(rgb(theme.active)))
                        .child(item.label())
                        .when(leaving.is_none(), |row| {
                            row.on_hover(cx.listener(move |this, hovered, _, cx| {
                                if *hovered {
                                    this.menu.selected = Some(index);
                                } else if this.menu.selected == Some(index) {
                                    this.menu.selected = None;
                                }
                                cx.notify();
                            }))
                            .on_click(cx.listener(
                                move |this, _, window, cx| {
                                    cx.stop_propagation();
                                    this.activate_menu(item, window, cx);
                                },
                            ))
                        }),
                );
            }
        } else if page == Page::Devices {
            panel = panel.child(match paint.and_then(|paint| paint.devices.as_ref()) {
                Some(paint) if paint.session_modal() => self
                    .render_devices_session_paint(paint, cx)
                    .unwrap_or_else(div),
                Some(paint) => self.render_devices_paint(paint, cx),
                None if self.menu.session_edit.is_some() => self.render_session_edit(cx),
                None => self.render_devices(cx),
            });
        } else if let Page::Usage(provider) = page {
            panel = panel.child(match paint.and_then(|paint| paint.usage.as_ref()) {
                Some(paint) => self.render_usage_paint(paint, cx),
                None => self.render_usage_panel(provider, cx),
            });
        } else if let Some(paint) = paint.and_then(|paint| paint.device_setup.as_ref()) {
            panel = panel.child(self.render_device_setup_paint(paint, compact_rail, cx));
        } else if page == Page::AddDevice {
            panel = panel.child(self.render_add_device(compact_rail, cx));
        } else if page == Page::AddWsl {
            panel = panel.child(self.render_add_wsl(compact_rail, cx));
        } else if page == Page::RemoveWsl {
            panel = panel.child(self.render_remove_wsl(cx));
        } else if page == Page::Workspace {
            panel = match paint.and_then(|paint| paint.workspace_popover.as_ref()) {
                Some(paint) => panel.child(self.render_workspace_popover_paint(paint, cx)),
                None => panel.child(self.render_workspace_popover(cx)),
            };
        } else if let Page::Dialog(action) = page {
            panel = match paint.and_then(|paint| paint.workspace_dialog.as_ref()) {
                Some(paint) => panel.child(self.render_workspace_dialog_paint(paint, cx)),
                None => panel.child(self.render_workspace_dialog(action, cx)),
            };
        } else if let Some(paint) = paint.and_then(|paint| paint.core_workspace.as_ref()) {
            panel = panel.child(self.render_core_workspace_paint(paint, cx));
        } else if page == Page::CoreWorkspaceActions {
            panel = panel.child(self.render_core_workspace_actions(cx));
        } else if matches!(
            page,
            Page::CoreWorkspaceCreate | Page::CoreWorkspaceDelete | Page::CoreWorkspaceRename
        ) {
            panel = panel.child(self.render_core_workspace_dialog(cx));
        } else if page == Page::DeviceTheme {
            panel = match paint.and_then(|paint| paint.device_theme.as_ref()) {
                Some(paint) => panel.child(self.render_device_theme_paint(paint, cx)),
                None => panel.child(self.render_host_menu(cx)),
            };
        } else if matches!(
            page,
            Page::Host
                | Page::RenameDevice
                | Page::ForwardPort
                | Page::DeviceTags
                | Page::RemoveDevice
        ) {
            panel = match paint.and_then(|paint| paint.host.as_ref()) {
                Some(paint) => panel.child(self.render_host_paint(paint, cx)),
                None => panel.child(self.render_host_menu(cx)),
            };
        } else if page == Page::MoveTab {
            panel = match paint.and_then(|paint| paint.tab.as_ref()) {
                Some(paint) => self.render_tab_move_paint(panel, paint, cx),
                None => self.render_tab_move_picker(panel, cx),
            };
        } else if matches!(page, Page::Tab | Page::RenameTab) {
            panel = match paint.and_then(|paint| paint.tab.as_ref()) {
                Some(paint) => panel.child(self.render_tab_menu_paint(paint, cx)),
                None => panel.child(self.render_tab_menu(cx)),
            };
        } else if page == Page::Group {
            panel = match paint.and_then(|paint| paint.group.as_ref()) {
                Some(paint) => self.render_group_menu_paint(panel, paint, cx),
                None => self.render_group_menu(panel, cx),
            };
        } else if page == Page::MovePane {
            panel = match paint.and_then(|paint| paint.pane.as_ref()) {
                Some(paint) => self.render_pane_move_paint(panel, paint, cx),
                None => self.render_move_picker(panel, cx),
            };
        } else if matches!(
            page,
            Page::Pane | Page::RenamePane | Page::PaneProcesses | Page::KillProcesses
        ) {
            panel = panel.child(match paint.and_then(|paint| paint.pane.as_ref()) {
                Some(paint) => self.render_pane_menu_paint(paint, cx),
                None => self.render_pane_menu(cx),
            });
        } else if page == Page::Keybinds {
            panel = match paint.and_then(|paint| paint.keybinds.as_ref()) {
                Some(paint) => panel.child(self.render_keybinds_paint(paint, cx)),
                None => panel.child(self.render_keybinds(cx)),
            };
        } else if page == Page::Messages {
            panel = match paint.and_then(|paint| paint.messages.as_ref()) {
                Some(paint) => panel.child(
                    self.render_messages_paint(&paint.entries, paint.offset, window)
                        .relative()
                        .top(shell_offset),
                ),
                None => panel.child(self.render_messages(window)),
            };
        } else if page == Page::Themes {
            panel = match paint.and_then(|paint| paint.themes.as_ref()) {
                Some(paint) => panel.child(self.render_theme_picker_paint(paint, cx)),
                None => panel.child(self.render_theme_picker(cx)),
            };
        } else if page == Page::Palette {
            panel = match paint.and_then(|paint| paint.palette.as_ref()) {
                Some(paint) => panel.child(self.render_palette_paint(paint, cx)),
                None => panel.child(self.render_palette(viewport, cx)),
            };
        } else if page == Page::ConfirmClose {
            panel = match paint.and_then(|paint| paint.close.as_ref()) {
                Some(paint) => panel.child(
                    self.render_close_paint(paint, cx)
                        .relative()
                        .top(shell_offset),
                ),
                None => panel.child(self.render_close_confirmation(cx)),
            };
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
                | Page::Messages
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
        // Closed shells retain numeric geometry, never a scroll owner or wheel listener.
        let panel = panel.when(leaving.is_some(), |panel| panel.overflow_hidden());
        // On Windows pages sit above everything GPUI draws, so the menu says
        // what it covers: a dimmed dialog covers the window, a popover its
        // panel. Tests and native checks read it to find the menu.
        #[cfg(any(windows, test, feature = "integration-test"))]
        let panel = {
            let cover = self.menu.cover.clone();
            if dims && leaving.is_none() {
                cover.set(Cover::All);
            }
            // A menu on its way out no longer covers anything.
            panel.when(!dims && leaving.is_none(), |panel| {
                panel.child(
                    canvas(
                        // Laid out inside the panel's border; the margin takes
                        // in the border and the start of its shadow.
                        move |bounds, _, _| {
                            cover.set(Cover::Panel(bounds.dilate(px(COVER_MARGIN))))
                        },
                        |_, _, _, _| {},
                    )
                    .absolute()
                    .inset_0(),
                )
            })
        };
        let placed = if pointer_anchored {
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
        } else if rises {
            self.anchor_footer_panel(
                div().id("menu-rise").overflow_hidden(),
                viewport,
                page,
                px(0.),
            )
            .child(panel)
            .into_any_element()
        } else {
            panel.into_any_element()
        };
        if leaving.is_some() {
            // Captured templates have no live handlers. Do not cover current
            // controls with a closing panel's invisible mouse shield.
            return div()
                .id("menu-leaving")
                .absolute()
                .inset_0()
                .when(dims, |layer| layer.row().items_center().justify_center())
                .child(placed);
        }
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
            .when(dims, |overlay| {
                overlay
                    .row()
                    .items_center()
                    .justify_center()
                    .bg(appear::backdrop(
                        theme.background,
                        self.menu.appear.backdrop(now),
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
