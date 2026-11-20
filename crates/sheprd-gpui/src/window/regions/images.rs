// ABOUTME: Places uncached image passes between the cached terminal cell layers.
// ABOUTME: Shares one prepared batch and consumes it before the popup image epoch.
use crate::{
    presentation::Picture,
    smooth_scroll::Slide,
    terminal::CellMetrics,
    terminal_painter::{ImageBatch, TerminalPainter},
};
use gpui::{AnyElement, Bounds, canvas, prelude::*};
use std::{cell::RefCell, rc::Rc};

mod bands;

pub(super) fn layers(
    painter: Rc<RefCell<TerminalPainter>>,
    picture: Picture,
    slide: Option<Slide>,
    metrics: CellMetrics,
) -> [Option<AnyElement>; 2] {
    let ready: Rc<RefCell<Option<ImageBatch>>> = Rc::default();
    let upper_ready = ready.clone();
    let upper_painter = painter.clone();
    let below = canvas(
        |_, _, _| (),
        move |bounds, _, window, cx| {
            let mut painter = painter.borrow_mut();
            let grid = Bounds::new(
                bounds.origin,
                metrics.extent(picture.frame.frame.width, picture.frame.frame.height),
            );
            let batch = painter.prepare_image_batch(
                &picture.images,
                bands::regions(&picture.frame, slide.as_ref(), grid, metrics, window),
                (metrics.width, metrics.height),
                window,
                cx,
            );
            batch.paint_below(&mut painter, window);
            *ready.borrow_mut() = Some(batch);
        },
    )
    .absolute()
    .top_0()
    .left_0()
    .size_full()
    .into_any_element();
    let above = canvas(
        |_, _, _| (),
        move |_, _, window, _| {
            if let Some(batch) = upper_ready.borrow_mut().take() {
                batch.paint_above(&mut upper_painter.borrow_mut(), window);
            }
        },
    )
    .absolute()
    .top_0()
    .left_0()
    .size_full()
    .into_any_element();
    [Some(below), Some(above)]
}
