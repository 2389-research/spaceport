//! The terminal's cells: the regions that paint them, the canvas that takes
//! their input and paints what changed without them, and what sits over them.

use super::Paint;
use crate::SheprdWindow;
use crate::config::ClipboardToastPosition;
use crate::terminal::*;
use crate::terminal_painter::{Ground, ImageTarget, PlacedImages};
use crate::window::PressedLink;
use gpui::{prelude::*, *};
use sheprd_client::ConnectOptions;

impl SheprdWindow {
    /// The cells box: the regions that paint, the canvas under them, and the
    /// overlays the user's own gestures put over them.
    pub(super) fn terminal_cells(
        &mut self,
        mut paint: Paint,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Div {
        let chats = std::mem::take(&mut paint.chats);
        let copy_badge = paint.copy_badge.take();
        let find_bar = paint.find_bar.take();
        let regions = std::mem::take(&mut paint.regions);
        let canvas = self.terminal_canvas(paint, window, cx);
        div()
            .relative()
            .size_full()
            .overflow_hidden()
            .children(regions)
            .child(canvas)
            .children(chats)
            .children(self.render_reconnecting(cx))
            .when_some(find_bar, |terminal, bar| terminal.child(bar))
            .when_some(copy_badge, |terminal, badge| terminal.child(badge))
            // Direct feedback for the user's own gesture, not a daemon notice:
            // it sits over the cells it copied and needs no dismissing.
            .when_some(self.flash_shown(), |terminal, (flash, shown)| {
                use ClipboardToastPosition::*;
                let position = self.config.clipboard_toast.position;
                // It comes in from the edge it sits by and goes back to it.
                let edge = px(12. - (1. - shown) * super::super::flash::SLIDE);
                terminal.child(
                    div()
                        .absolute()
                        .opacity(shown)
                        .map(|row| match position {
                            TopLeft | TopCenter | TopRight => row.top(edge),
                            BottomLeft | BottomCenter | BottomRight => row.bottom(edge),
                        })
                        .map(|row| match position {
                            TopLeft | BottomLeft => row.justify_start(),
                            TopCenter | BottomCenter => row.justify_center(),
                            TopRight | BottomRight => row.justify_end(),
                        })
                        // The flash spans the cells, so centering centers on
                        // them and a corner is the corner of the grid.
                        .left_0()
                        .right_0()
                        .px(px(12.))
                        .flex()
                        .overflow_hidden()
                        .child(
                            div()
                                .debug_selector(|| "flash".into())
                                .min_w_0()
                                .flex()
                                .items_center()
                                .gap(px(8.))
                                .px(px(12.))
                                .py(px(6.))
                                .rounded(px(crate::config::corners::CONTROL))
                                .border_1()
                                .border_color(rgb(flash.accent(&self.theme)))
                                .bg(rgb(self.theme.surface))
                                .text_color(rgb(self.theme.foreground))
                                .child(
                                    div()
                                        .size(px(6.))
                                        .flex_none()
                                        .rounded_full()
                                        .bg(rgb(flash.accent(&self.theme))),
                                )
                                .child(div().truncate().child(flash.text.clone())),
                        ),
                )
            })
    }

    /// The canvas over the cells: it reports the box the terminal is laid out
    /// in, and on every paint takes the pointer's input and paints the frame.
    fn terminal_canvas(
        &mut self,
        paint: Paint,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let entity = paint.entity.clone();
        let paint_entity = paint.entity.clone();
        let cell_width = paint.cell_width;
        let cell_height = paint.cell_height;
        canvas(
            move |bounds, _, cx| {
                entity.update(cx, |this, _| {
                    this.bounds = bounds;
                    this.options = ConnectOptions {
                        surface_size: viewport(
                            bounds.size.width.to_f64() as f32,
                            bounds.size.height.to_f64() as f32,
                            cell_width,
                            cell_height,
                        ),
                        cell_width_px: cell_width.round().max(1.) as u32,
                        cell_height_px: cell_height.round().max(1.) as u32,
                    };
                    this.resize();
                });
            },
            move |bounds, _, window, cx| {
                paint_entity.update(cx, |this, cx| {
                    this.terminal_pointer_regions(window, cx);
                    this.paint_cells(bounds, &paint, window, cx);
                });
            },
        )
        .size_full()
    }
}

impl SheprdWindow {
    /// Registers the pointer work the terminal needs window-wide, before any
    /// element can stop propagation of a drag that leaves it.
    fn terminal_pointer_regions(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        // Capture movement outside the terminal too, before any
        // element can stop propagation of a drag-away event.
        let moving = cx.entity();
        window.on_mouse_event(move |event: &MouseMoveEvent, phase, _, cx| {
            if phase == DispatchPhase::Capture {
                moving.update(cx, |this, cx| {
                    // Window-wide, so leaving the terminal
                    // drops the underline too.
                    this.hover_link(event.position, event.modifiers, cx);
                    if this.scrollbar_mouse_move(event, cx)
                        || this.split_mouse_move(event, cx)
                        || this.terminal_mouse_move(event, cx)
                    {
                        cx.stop_propagation();
                        return;
                    }
                    if this.pressed_terminal_link.as_ref().is_some_and(
                        |PressedLink { position, .. }| {
                            (event.position.x - position.x).abs() > px(4.)
                                || (event.position.y - position.y).abs() > px(4.)
                        },
                    ) {
                        this.pressed_terminal_link = None;
                    }
                    // A drag that leaves the terminal keeps
                    // selecting, and hover work elsewhere stays
                    // out of the gesture.
                    if this.extend_selection(event.position, cx) {
                        cx.stop_propagation();
                    }
                });
            }
        });
        let released = cx.entity();
        window.on_mouse_event(move |event: &MouseUpEvent, phase, _, cx| {
            if phase == DispatchPhase::Capture {
                released.update(cx, |this, cx| {
                    // Global: the overlay occludes the opener, and release
                    // may also precede the overlay's first frame.
                    if matches!(event.button, MouseButton::Left | MouseButton::Right) {
                        this.menu.opening_right_click = false;
                    }
                    if this.scrollbar_mouse_up(event, cx)
                        || this.split_mouse_up(event, cx)
                        || this.terminal_mouse_up(event, cx)
                        || (event.button == MouseButton::Left
                            && !cx.has_active_drag()
                            && this.release_selection(cx))
                    {
                        cx.stop_propagation();
                    }
                });
            }
        });
    }

    /// The input the canvas takes, and the cells, links, popup, and IME
    /// composition it paints.
    fn paint_cells(
        &mut self,
        bounds: Bounds<Pixels>,
        paint: &Paint,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Paint {
            cell_height: _,
            cell_width,
            focus,
            font,
            ground,
            highlights,
            images,
            link_rows,
            link_shift,
            marked,
            menu_open,
            metrics,
            painter,
            popup_highlights,
            surface,
            whole,
            ..
        } = paint;
        window.handle_input(
            focus,
            crate::input::TerminalInputHandler::new(bounds, cx.entity(), *menu_open),
            cx,
        );
        if let Some(surface) = &surface {
            if *whole {
                let paint = |window: &mut Window, cx: &mut App| {
                    painter.borrow_mut().paint_frame(
                        &surface.frame,
                        bounds.origin,
                        Some(bounds.size),
                        *ground,
                        *cell_width,
                        font,
                        &highlights,
                        &surface.panes,
                        None,
                        images.as_deref().map(|images| PlacedImages {
                            placements: &surface.graphics.placements,
                            images,
                            target: ImageTarget::Main,
                        }),
                        window,
                        cx,
                    );
                };
                paint(window, cx);
            } else {
                // Regions place no images, so nothing
                // else releases the ones that went away.
                painter.borrow_mut().release_idle_images(window);
            }
            let paint_link = |window: &mut Window, shift| {
                painter.borrow().paint_link(
                    &surface.frame,
                    bounds.origin + point(px(0.), shift),
                    *cell_width,
                    link_rows,
                    window,
                )
            };
            match link_shift {
                Some((pane, shift)) => window.with_content_mask(
                    Some(ContentMask {
                        bounds: Bounds::new(bounds.origin + pane.origin, pane.size),
                    }),
                    |window| paint_link(window, *shift),
                ),
                None => paint_link(window, px(0.)),
            }
            if let Some(popup) = &surface.popup {
                let offset = popup_origin(&surface.frame, &popup.frame, *metrics);
                painter.borrow_mut().paint_frame(
                    &popup.frame,
                    bounds.origin + offset,
                    None,
                    Ground::Opaque,
                    *cell_width,
                    font,
                    &popup_highlights,
                    &[],
                    None,
                    images.as_deref().map(|images| PlacedImages {
                        placements: &surface.graphics.placements,
                        images,
                        target: ImageTarget::Popup(&popup.terminal_id),
                    }),
                    window,
                    cx,
                );
            }
        }
        if let Some((marked, live, shift)) = &marked {
            let live = live.as_deref();
            painter.borrow().paint_composition(
                marked,
                input_cursor_bounds(live, bounds.origin, *metrics).origin + point(px(0.), *shift),
                input_area(live, bounds, *metrics),
                &font,
                window,
            );
        }
    }
}
