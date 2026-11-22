// ABOUTME: Lays out prepared sidebar rows, headings and resize controls.
// ABOUTME: Keeps runtime-specific sections behind the running session's feature choices.
//! Laying out the sidebar: the two lists, their headings, and the drag handle
//! that resizes the panel. Render works from prepared state and the bounded
//! caches only.

use super::{
    DEVICE_FOOTER_HEIGHT, SidebarDrag, agents::Indicators, agents_sort, label_text, line_height,
    reorder, sidebar_width, sticky,
};
use crate::direction::Logical;
use crate::i18n::fl;
use crate::{Command, SpaceportWindow, fonts::StyledFont};
use gpui::{prelude::*, *};

mod agent_rows;
mod device_tree;
mod edges;
mod header;
mod measure;
mod nested_agents;
mod rows;

pub(super) use header::header;
pub(crate) use nested_agents::NestedReveal;

use measure::SidebarMeasure;
use rows::SidebarRows;

impl SpaceportWindow {
    pub(crate) fn render_sidebar(
        &self,
        indicators: Indicators,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Stateful<Div> {
        let m = self.sidebar_measure(window, cx);
        let rows = self.sidebar_rows(&m, indicators, cx);
        let SidebarMeasure {
            width,
            split,
            look,
            headed,
            agents_panel,
            devices,
            searchable,
            tree_search,
            device_plan,
            ..
        } = m;
        let SidebarRows {
            spaces,
            mut agents,
            search,
            highlighted,
            agent_count,
            filtered,
            nested_focus,
            host_rows,
            drop_rows,
            drop_requests,
            drop_dragged,
            sliding,
        } = rows;
        let layout = look.density;
        let content_x = look.content_x();
        let view = cx.entity().downgrade();
        let font = &self.config.sidebar;
        let theme = &self.theme;
        if sliding {
            window.request_animation_frame();
        }
        let mut headers = if headed {
            self.host_headers(&host_rows)
        } else {
            Vec::new()
        };
        if let Some(row) = self.sidebar_pin_reveal.take()
            && self.uncover_revealed(row, &headers)
        {
            headers = self.host_headers(&host_rows);
        }
        // Rows under the pinned header are hidden, so a reveal must clear it.
        let cover = px(sticky::cover(&headers));
        // Search results stand in for the spaces list, so no nested row paints.
        self.reveal_nested_agent(nested_focus.filter(|_| search.is_none()), cover, window);
        // Folded headers slide under a pinned one, so none is pinned then,
        // nor in the frame that unfolds, whose header places are the folded
        // list's; the frame after pins from the unfolded one.
        let unfolding = self.sidebar_folded.replace(device_plan.is_some()) && device_plan.is_none();
        if unfolding {
            window.request_animation_frame();
        }
        let pinned_host = sticky::pinned(&headers)
            .filter(|_| device_plan.is_none() && !unfolding)
            .and_then(|pinned| self.pinned_host_row(pinned, look, width, cx));
        // Follow the selection, but only once a frame has measured the viewport:
        // the handle resolves the request against the previous frame's bounds, so
        // an unmeasured list would scroll to a meaningless offset. Recording what
        // was revealed keeps later frames from undoing the user's own scrolling.
        for (list, row) in highlighted.iter().enumerate() {
            let Some(row) = *row else { continue };
            if self.sidebar_revealed[list].get() != Some(row)
                && self.sidebar_scroll[list].bounds().size.height > px(0.)
            {
                let scroll = &self.sidebar_scroll[list];
                let cover = if list == 0 { cover } else { px(0.) };
                let visible = scroll.bounds_for_item(row).is_some_and(|bounds| {
                    let offset = scroll.offset().y;
                    bounds.bottom() + offset > scroll.bounds().top() + cover
                        && bounds.top() + offset < scroll.bounds().bottom()
                });
                // Even a partially visible worktree is already seen. GPUI's
                // reveal also moves clipped rows, so only request it off-screen.
                if !visible {
                    scroll.scroll_to_item(row);
                    // The scroll lands in prepaint, after this render pinned a
                    // header for the old offset, and puts a row from above at
                    // the top edge, under the pinned header. Another frame pins
                    // for the new offset and moves the row out from under it.
                    if list == 0 && headed {
                        window.request_animation_frame();
                        self.sidebar_pin_reveal.set(Some(row));
                    }
                }
                self.sidebar_revealed[list].set(Some(row));
            }
        }
        if agent_count == 0 {
            agents = agents.child(
                div()
                    .px(px(content_x))
                    .text_color(rgb(theme.muted))
                    .truncate()
                    .child(label_text(&if filtered {
                        fl!("sidebar-no-matching-agents")
                    } else {
                        fl!("sidebar-no-agents")
                    })),
            );
        }
        div()
            .id("sidebar")
            .debug_selector(|| "sidebar".into())
            .role(Role::Navigation)
            .aria_label(fl!("sidebar-label"))
            .relative()
            .w(px(width))
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
                        section.style().flex_grow = Some(if agents_panel { split } else { 1. });
                        section
                    })
                    .min_h_0()
                    .overflow_hidden()
                    // With search on, the heading is its field.
                    .child(
                        searchable
                            .then_some(super::search::Heading::Spaces)
                            .or(tree_search.then_some(super::search::Heading::DeviceTree))
                            .and_then(|heading| self.search_heading(heading, look, window, cx))
                            .unwrap_or_else(|| {
                                let (key, label) = if devices {
                                    ("devices", fl!("sidebar-header-devices"))
                                } else {
                                    ("spaces", fl!("sidebar-header-spaces"))
                                };
                                header(key, &label, font, theme, look)
                            }),
                    )
                    .children(headed.then(|| self.tag_switch(cx)).flatten())
                    // The wrapper clips the pinned header as the next host's
                    // pushes it up, so it never paints over the title above.
                    .child(
                        div()
                            .relative()
                            .flex_1()
                            .min_h_0()
                            .flex()
                            .flex_col()
                            .overflow_hidden()
                            .map(|list| match search {
                                Some(results) => list.child(results),
                                None => list
                                    .child(spaces)
                                    .children(pinned_host.map(crate::mark_layer::covering)),
                            }),
                    )
                    .child(
                        div()
                            .flex_none()
                            .h(px(line_height(font) + 2. * layout.footer_padding()))
                            .px(px(content_x))
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
            .when(agents_panel, |sidebar| {
                sidebar.child(self.sidebar_split_edge(theme, cx)).child(
                    div()
                        .debug_selector(|| "agents-section".into())
                        .flex()
                        .flex_col()
                        .flex_1()
                        .map(|mut section| {
                            section.style().flex_grow = Some(1. - split);
                            section
                        })
                        .min_h_0()
                        .overflow_hidden()
                        // A plugin view on any listed host filters the
                        // rows, so the header stays to say so.
                        .when(self.config.show_agents_header || filtered, |section| {
                            section.child(
                                header("agents", &fl!("sidebar-header-agents"), font, theme, look)
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
            .child(self.sidebar_width_edge(width, cx))
            .child(
                canvas(
                    |_, _, _| (),
                    move |bounds, _, window, _| {
                        // Capture globally so dragging continues outside the narrow divider,
                        // and terminal handlers never receive the resize gesture's release.
                        let moving = view.clone();
                        let device_drop = device_plan.map(|plan| plan.drop);
                        window.on_mouse_event(move |event: &MouseMoveEvent, phase, window, cx| {
                            if phase == DispatchPhase::Capture {
                                let _ = moving.update(cx, |this, cx| {
                                    let scroll = this.sidebar_scroll[0].clone();
                                    if this.move_workspace_drag(
                                        event.position,
                                        event.pressed_button == Some(MouseButton::Left),
                                        |lift| {
                                            reorder::resolve(
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
