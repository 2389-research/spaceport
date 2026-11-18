// ABOUTME: Shares uncached terminal image drawing across whole and layered frames.
// ABOUTME: Prepares one bounded batch and consumes its upper pass before popup painting.
use super::{
    ImageRegion, ImageTarget, TerminalPainter,
    images::{ImageGeometry, below_text},
};
use gpui::{App, ContentMask, Corners, RenderImage, Window};
use sheprd_client::SurfaceImages;
use std::{sync::Arc, time::Instant};

type ReadyImage = (i32, ImageGeometry, Arc<RenderImage>);

/// Only one window frame owns this batch; no cached scene retains its textures.
pub(crate) struct ImageBatch {
    ready: Vec<ReadyImage>,
    upper: usize,
}

impl ImageBatch {
    pub(crate) fn paint_below(&self, painter: &mut TerminalPainter, window: &mut Window) {
        paint(painter, &self.ready[..self.upper], window);
    }

    /// Consume the batch before another target starts an eviction epoch.
    pub(crate) fn paint_above(self, painter: &mut TerminalPainter, window: &mut Window) {
        paint(painter, &self.ready[self.upper..], window);
    }
}

fn paint(painter: &mut TerminalPainter, ready: &[ReadyImage], window: &mut Window) {
    let failures = painter.paint_images(ready, window);
    if let Some(count) = painter.diagnostics.take_errors(Instant::now(), failures) {
        tracing::warn!(category = "glyph_paint", count, "Terminal paint failed");
    }
}

impl TerminalPainter {
    pub(crate) fn prepare_image_batch<'a>(
        &mut self,
        images: &SurfaceImages,
        regions: impl IntoIterator<Item = ImageRegion<'a>>,
        cell: (f32, f32),
        window: &mut Window,
        cx: &mut App,
    ) -> ImageBatch {
        let ready =
            self.images
                .prepare_regions(images, ImageTarget::Main, regions, cell, window, cx);
        let upper = ready.partition_point(|(z, ..)| below_text(*z));
        ImageBatch { ready, upper }
    }

    /// Paints images in order, returning how many failed.
    pub(super) fn paint_images(&mut self, placed: &[ReadyImage], window: &mut Window) -> u64 {
        let mut failed = 0;
        for (_z, geometry, texture) in placed {
            // Cropping paint_image's bounds quantizes the source UVs to atlas pixels.
            // A destination mask preserves the mapping for fractional image fragments.
            let painted = window.with_content_mask(
                Some(ContentMask {
                    bounds: geometry.mask,
                }),
                |window| {
                    let painted = window.paint_image(
                        geometry.visible,
                        geometry.image,
                        Corners::default(),
                        texture.clone(),
                        0,
                        false,
                    );
                    #[cfg(test)]
                    super::image_probe::image(*_z, geometry, texture, painted.is_ok(), window);
                    painted
                },
            );
            failed += u64::from(painted.is_err());
            #[cfg(test)]
            self.painted_images
                .push((*_z, geometry.visible.intersect(&geometry.mask)));
        }
        failed
    }
}
