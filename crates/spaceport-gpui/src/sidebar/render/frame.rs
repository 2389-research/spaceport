//! Painting the sidebar from measured, built and settled state: its sections,
//! their headings, the two lists, the footer, and the edges dragged to resize.

use super::super::{
    DEVICE_FOOTER_HEIGHT, SidebarDrag, agents_sort, label_text, line_height, resolve, search,
    sidebar_width,
};
use super::header::header;
use super::measure::SidebarMeasure;
use super::rows::SidebarRows;
use super::settle::SidebarSettled;
use crate::direction::Logical;
use crate::i18n::fl;
use crate::{Command, SpaceportWindow, fonts::StyledFont};
use gpui::{prelude::*, *};

impl SpaceportWindow {
    /// Paints the panel: the spaces section with its heading, list and footer,
    /// the agents section below it, and the edges that resize them, with the
    /// pointer handlers that carry a drag on outside the panel.
    #[allow(clippy::too_many_lines, reason = "2026-11-27 baseline: 273 lines")]
    pub(super) fn sidebar_frame(
        &self,
        m: &SidebarMeasure<'_>,
        rows: SidebarRows,
        settled: SidebarSettled,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Stateful<Div> {
        let SidebarSettled { pinned_host } = settled;
        let SidebarRows {
            spaces,
            agents,
            search: results,
            filtered,
            drop_rows,
            drop_requests,
            drop_dragged,
            ..
        } = rows;
        let view = cx.entity().downgrade();
        // The pointer handler outlives this frame, so it takes the drop it
        // resolves against rather than borrowing the plan's.
        let device_drop = m.device_plan.as_ref().map(|plan| plan.drop.clone());
        let font = &self.config.sidebar;
        let theme = &self.theme;
        div()
            .id("sidebar")
            .debug_selector(|| "sidebar".into())
            .role(Role::Navigation)
            .aria_label(fl!("sidebar-label"))
            .relative()
            .w(px(m.width))
            .flex_none()
            .h_full()
            .min_h_0()
            .overflow_hidden()
            .flex()
            .flex_col()
            .text_font(font)
            .text_size(px(font.size))
            .line_height(px(line_height(font)))
            .text_color(rgb(theme.foreground))
            .bg(self.fill().panel(theme.sidebar_background()))
            .border_e(px(1.))
            .border_color(rgb(theme.active))
            // Zero flex bases keep long workspace lists from displacing agents.
            .child(
                div()
                    .debug_selector(|| "spaces-section".into())
                    .flex()
                    .flex_col()
                    .flex_1()
                    .map(|mut section| {
                        section.style().flex_grow = Some(if m.agents_panel { m.split } else { 1. });
                        section
                    })
                    .when(m.workspaces_collapsed && m.agents_panel, |section| {
                        section.flex_none()
                    })
                    .min_h_0()
                    .overflow_hidden()
                    // With search on, the heading is its field.
                    .child(
                        m.searchable
                            .then_some(search::Heading::Spaces)
                            .or(m.tree_search.then_some(search::Heading::DeviceTree))
                            .and_then(|heading| self.search_heading(heading, m.look, window, cx))
                            .unwrap_or_else(|| {
                                let (key, label) = if m.devices {
                                    ("devices", fl!("sidebar-header-devices"))
                                } else {
                                    ("spaces", fl!("sidebar-header-spaces"))
                                };
                                let title = header(key, &label, font, theme, m.look);
                                if m.devices {
                                    title
                                } else {
                                    div().debug_selector(|| "header-spaces".into()).child(
                                        self.workspace_disclosure(
                                            title.justify_between(),
                                            !m.workspaces_collapsed,
                                            cx,
                                        ),
                                    )
                                }
                            }),
                    )
                    .children(
                        (m.headed && !m.workspaces_collapsed)
                            .then(|| self.tag_switch(cx))
                            .flatten(),
                    )
                    // The wrapper clips the pinned header as the next host's
                    // pushes it up, so it never paints over the title above.
                    .when(!m.workspaces_collapsed, |section| {
                        section.child(
                            div()
                                .relative()
                                .flex_1()
                                .min_h_0()
                                .flex()
                                .flex_col()
                                .overflow_hidden()
                                .map(|list| match results {
                                    Some(results) => list.child(results),
                                    None => list
                                        .child(spaces)
                                        .children(pinned_host.map(crate::mark_layer::covering)),
                                }),
                        )
                    })
                    .child(
                        div()
                            .flex_none()
                            .h(px(line_height(font) + 2. * m.look.density.footer_padding()))
                            .px(px(m.content_x))
                            .row()
                            .items_center()
                            // Menu hugs the sidebar's edge, as in the terminal client.
                            .justify_between()
                            .text_color(rgb(theme.muted))
                            .gap(px(20.))
                            .child(
                                div()
                                    .id("new-workspace")
                                    .map(|button| {
                                        self.sidebar_button(button, fl!("sidebar-new-workspace"))
                                    })
                                    .cursor_pointer()
                                    .hover(|s| s.text_color(rgb(theme.foreground)))
                                    .child(fl!("sidebar-new"))
                                    .on_click(cx.listener(|this, _, window, cx| {
                                        this.command(Command::Workspace, window, cx)
                                    })),
                            )
                            .child(
                                div()
                                    .id("sidebar-menu")
                                    .debug_selector(|| "sidebar-menu".into())
                                    .map(|button| {
                                        self.sidebar_button(button, fl!("sidebar-menu-label"))
                                    })
                                    .cursor_pointer()
                                    .hover(|s| s.text_color(rgb(theme.foreground)))
                                    .child(label_text(&fl!("sidebar-menu")))
                                    .on_click(cx.listener(
                                        |this, event: &ClickEvent, window, cx| {
                                            this.menu.anchor = event.position();
                                            this.open_menu(window, cx);
                                        },
                                    )),
                            ),
                    ),
            )
            .when(m.agents_panel, |sidebar| {
                sidebar
                    .when(!m.workspaces_collapsed, |sidebar| {
                        sidebar.child(self.sidebar_split_edge(theme, cx))
                    })
                    .child(
                        div()
                            .debug_selector(|| "agents-section".into())
                            .flex()
                            .flex_col()
                            .flex_1()
                            .map(|mut section| {
                                section.style().flex_grow = Some(if m.workspaces_collapsed {
                                    1.
                                } else {
                                    1. - m.split
                                });
                                section
                            })
                            .min_h_0()
                            .overflow_hidden()
                            // A plugin view on any listed host filters the
                            // rows, so the header stays to say so.
                            .when(self.config.show_agents_header || filtered, |section| {
                                section.child(
                                    header(
                                        "agents",
                                        &fl!("sidebar-header-agents"),
                                        font,
                                        theme,
                                        m.look,
                                    )
                                    .justify_between()
                                    .child(agents_sort(self, cx)),
                                )
                            })
                            .child(agents)
                            .children(self.core_sidebar_rows(cx)),
                    )
            })
            .map(|sidebar| {
                if self.device_footer_shown() {
                    sidebar.child(self.render_device_footer(cx))
                } else {
                    sidebar.child(self.render_sessions_anchor())
                }
            })
            .child(self.sidebar_width_edge(m.width, cx))
            .child(
                canvas(
                    |_, _, _| (),
                    move |bounds, _, window, _| {
                        // Capture globally so dragging continues outside the narrow divider,
                        // and terminal handlers never receive the resize gesture's release.
                        let moving = view.clone();
                        window.on_mouse_event(move |event: &MouseMoveEvent, phase, window, cx| {
                            if phase == DispatchPhase::Capture {
                                let _ = moving.update(cx, |this, cx| {
                                    let scroll = this.sidebar_scroll[0].clone();
                                    if this.move_workspace_drag(
                                        event.position,
                                        event.pressed_button == Some(MouseButton::Left),
                                        |lift| {
                                            resolve(
                                                &drop_rows,
                                                &drop_requests,
                                                drop_dragged,
                                                &scroll,
                                                lift,
                                            )
                                        },
                                        cx,
                                    ) || this.move_device_drag(
                                        event.position,
                                        event.pressed_button == Some(MouseButton::Left),
                                        device_drop.as_ref(),
                                        cx,
                                    ) {
                                        cx.stop_propagation();
                                    } else if let Some(drag) = this.sidebar_drag {
                                        match drag {
                                            SidebarDrag::Width { start, width } => {
                                                this.sidebar_width = Some(sidebar_width(
                                                    // The edge is on the sidebar's end.
                                                    Some(
                                                        width
                                                            + crate::direction::toward_end(
                                                                f32::from(event.position.x) - start,
                                                            ),
                                                    ),
                                                    f32::from(window.viewport_size().width),
                                                ));
                                            }
                                            SidebarDrag::Split => {
                                                let footer = if this.device_footer_shown() {
                                                    DEVICE_FOOTER_HEIGHT
                                                } else {
                                                    0.
                                                };
                                                let height =
                                                    (f32::from(bounds.size.height) - 6. - footer)
                                                        .max(1.);
                                                this.sidebar_split = Some(
                                                    ((f32::from(
                                                        event.position.y - bounds.origin.y,
                                                    ) - 3.)
                                                        / height)
                                                        .clamp(0.1, 0.9),
                                                );
                                            }
                                        }
                                        cx.stop_propagation();
                                        cx.notify();
                                    }
                                });
                            }
                        });
                        let released = view.clone();
                        window.on_mouse_event(move |event: &MouseUpEvent, phase, _, cx| {
                            if phase == DispatchPhase::Capture && event.button == MouseButton::Left
                            {
                                let _ = released.update(cx, |this, cx| {
                                    // A lifted row's release is its drop, not a click.
                                    if this.release_workspace_drag(cx)
                                        || this.release_device_drag(cx)
                                    {
                                        cx.stop_propagation();
                                    } else if this.sidebar_drag.take().is_some() {
                                        this.save_chrome();
                                        cx.stop_propagation();
                                        cx.notify();
                                    }
                                });
                            }
                        });
                    },
                )
                .absolute()
                .size_full(),
            )
    }
}
