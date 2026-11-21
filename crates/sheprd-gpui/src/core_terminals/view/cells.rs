// ABOUTME: Replays Core terminal cells while their frame and appearance stay unchanged.
// ABOUTME: Leaves input registration and geometry in the enclosing terminal view.

use super::{CoreTerminalView, Look, PaneLook};
use crate::terminal_painter::{Appearance, Highlight, TerminalPainter};
use gpui::{prelude::*, *};
use sheprd_backend::FrameData;
use std::{cell::RefCell, rc::Rc, sync::Arc};

pub(super) struct Cells {
    frame: Arc<FrameData>,
    highlights: Vec<Highlight>,
    look: Look,
    pane: Option<PaneLook>,
    appearance: Appearance,
    scale: f32,
    painter: Rc<RefCell<TerminalPainter>>,
}

impl CoreTerminalView {
    pub(super) fn cell_layer(
        &mut self,
        frame: Option<Arc<FrameData>>,
        scale: f32,
        cx: &mut Context<Self>,
    ) -> Option<ViewElement<Entity<Cells>>> {
        let Some(frame) = frame else {
            self.cells = None;
            return None;
        };
        let appearance = self.painter.borrow().appearance();
        let highlights = self.highlights();
        let cells = self.cells.take().filter(|cells| {
            let cells = cells.read(cx);
            Arc::ptr_eq(&cells.frame, &frame)
                && cells.look == self.look
                && cells.pane == self.pane
                && cells.appearance == appearance
                && cells.highlights == highlights
                && cells.scale == scale
        });
        // A fresh entity paints the change this frame. Notifying a cached
        // entity during render would invalidate its scene for the next frame.
        let cells = cells.unwrap_or_else(|| {
            cx.new(|_| Cells {
                frame,
                highlights,
                look: self.look.clone(),
                pane: self.pane,
                appearance,
                scale,
                painter: self.painter.clone(),
            })
        });
        self.cells = Some(cells.clone());
        Some(
            cells.cached(
                StyleRefinement::default()
                    .absolute()
                    .top_0()
                    .left_0()
                    .size_full(),
            ),
        )
    }
}

impl Render for Cells {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let frame = self.frame.clone();
        let highlights = self.highlights.clone();
        let font = self.look.font.clone();
        let ground = self.look.ground;
        let cell_width = self.look.cell_width;
        let painter = self.painter.clone();
        canvas(
            |_, _, _| (),
            move |bounds, _, window, cx| {
                window.with_content_mask(Some(ContentMask { bounds }), |window| {
                    #[cfg(test)]
                    crate::window::native_images::cells(window.window_handle().window_id());
                    painter.borrow_mut().paint_frame(
                        &frame,
                        bounds.origin,
                        Some(bounds.size),
                        ground,
                        cell_width,
                        &font,
                        &highlights,
                        &[],
                        None,
                        None,
                        window,
                        cx,
                    );
                });
            },
        )
        .size_full()
    }
}
