//! Drawing the popup layer: what the chrome asks of this window for one call,
//! and the handlers and page bodies it hands back to be laid in the window.

use super::super::appear::{self, Kept};
use super::super::devices::rail;
use super::{Page, metrics, text};
use crate::SpaceportWindow;
use crate::actions;
use gpui::{prelude::*, *};
use spaceport_menu::chrome::{self, Hosts, Layer};
use spaceport_menu::page_title;
use spaceport_menu::rows::{self, Row};

/// How far past its panel a popover counts as covering, for the Windows
/// pages that hide under it.
#[cfg(any(windows, test, feature = "integration-test"))]
const COVER_MARGIN: f32 = 8.;

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
        let hosts = self.menu_hosts(page, leaving, window);
        let layer = chrome::layer(&hosts, page);
        let panel = chrome::panel(&hosts, page, &layer);
        // A menu the pointer opened follows the pointer's own report of
        // whether it is over the popup, which occlusion and snapping make
        // impossible to infer from the anchor alone.
        let panel = panel.when(self.hover_menu.is_some(), |panel| {
            panel.on_hover(cx.listener(|this, hovered: &bool, _, _| {
                if let Some(open) = &mut this.hover_menu {
                    open.inside = *hovered;
                }
            }))
        });
        let panel = self.menu_page_body(page, &layer, panel, cx);
        #[cfg(any(windows, test, feature = "integration-test"))]
        let panel = self.menu_panel_cover(&layer, panel);
        let panel = self.menu_panel_shield(&layer, panel);
        let placed = chrome::placement(&hosts, page, &layer, panel);
        // On its way out it takes no clicks: what they would act on has
        // already closed, and it is the last thing it paints.
        if layer.leaving.is_some() {
            return chrome::leaving(&layer, placed);
        }
        chrome::overlay(
            &hosts,
            &layer,
            page_title::role(page),
            page_title::name(page),
            placed,
        )
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
        .on_action(
            cx.listener(|this, _: &actions::SelectAll, window, cx| this.menu_edit("a", window, cx)),
        )
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
    }

    /// What the popup chrome may read of this window for one call.
    fn menu_hosts(&self, page: Page, leaving: Option<f32>, window: &Window) -> Hosts<'_, Kept> {
        let now = crate::motion::now();
        let viewport = window.viewport_size();
        Hosts {
            add_device: rail::Source::of(page).is_some(),
            anchor: self.menu.anchor,
            appear: &self.menu.appear.state,
            backdrop: appear::backdrop(self.theme.background, self.menu.appear.state.backdrop(now)),
            host_forwards: self.host_menu_lists_forwards(),
            leaving,
            metrics: metrics(viewport),
            now,
            palette_previews: self.palette_previews(viewport),
            popover: (page == Page::Workspace).then(|| {
                self.workspace_popover_width(window, (viewport.width - px(24.)).max(px(0.)))
            }),
            session_modal: self.menu.session_modal(page),
            text: text(&self.config.ui),
            theme: &self.theme,
        }
    }

    /// The page the menu is on, under the panel's own padding.
    #[allow(
        clippy::cognitive_complexity,
        clippy::too_many_lines,
        reason = "2026-11-27 baseline: complexity 31, 109 lines"
    )]
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

    /// The app menu's own rows, each an item the window runs.
    fn menu_item_list(&self, panel: Stateful<Div>, cx: &mut Context<Self>) -> Stateful<Div> {
        let items = self.menu_items();
        let rows: Vec<Row> = items
            .iter()
            .map(|item| Row {
                label: item.label(),
                selector: format!("menu-{}", item.selector()),
                ..Row::default()
            })
            .collect();
        panel.children(rows::list(
            &text(&self.config.ui),
            &self.theme,
            &rows,
            self.menu.selected,
            |index, row| {
                let item = items[index];
                row.on_hover(cx.listener(move |this, hovered, _, cx| {
                    if *hovered {
                        this.menu.selected = Some(index);
                    } else if this.menu.selected == Some(index) {
                        this.menu.selected = None;
                    }
                    cx.notify();
                }))
                .on_click(cx.listener(move |this, _, window, cx| {
                    cx.stop_propagation();
                    this.activate_menu(item, window, cx);
                }))
            },
        ))
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
}
