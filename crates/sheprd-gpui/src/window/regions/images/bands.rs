// ABOUTME: Maps current and historical image fragments onto the terminal scroll bands.
// ABOUTME: Clips fractional images to their source rows and the stationary pane viewport.
use crate::{smooth_scroll::Slide, terminal::CellMetrics, terminal_painter::ImageRegion};
use gpui::{Bounds, Pixels, Window, point, px, size};
use sheprd_client::protocol::{PaneSurfaceFrame, SurfaceRect};

pub(super) fn regions<'a>(
    surface: &'a PaneSurfaceFrame,
    slide: Option<&'a Slide>,
    grid: Bounds<Pixels>,
    metrics: CellMetrics,
    window: &Window,
) -> Vec<ImageRegion<'a>> {
    let placements = &surface.graphics.placements;
    let Some(slide) = slide else {
        return vec![ImageRegion {
            placements,
            origin: grid.origin,
            grid,
            clip: grid,
        }];
    };
    let local = metrics.bounds(slide.rect);
    let pane = Bounds::new(grid.origin + local.origin, local.size).intersect(&grid);
    let offset = super::super::painted_shift(slide.offset, metrics.height, window.scale_factor());
    // Outside fragments remain stationary even when a placement crosses the pane.
    let mut regions: Vec<_> = [
        Bounds::new(grid.origin, size(grid.size.width, pane.top() - grid.top())),
        Bounds::new(
            point(grid.left(), pane.bottom()),
            size(grid.size.width, grid.bottom() - pane.bottom()),
        ),
        Bounds::new(
            point(grid.left(), pane.top()),
            size(pane.left() - grid.left(), pane.size.height),
        ),
        Bounds::new(
            point(pane.right(), pane.top()),
            size(grid.right() - pane.right(), pane.size.height),
        ),
    ]
    .into_iter()
    .filter(|clip| clip.size.width > Pixels::ZERO && clip.size.height > Pixels::ZERO)
    .map(|clip| ImageRegion {
        placements,
        origin: grid.origin,
        grid,
        clip,
    })
    .collect();
    for (earlier, shift, band) in slide.bands() {
        let rows = SurfaceRect {
            x: slide.rect.x,
            y: band.start,
            width: slide.rect.width,
            height: band.end - band.start,
        };
        let band = metrics.bounds(rows);
        let origin = grid.origin + point(px(0.), px(offset + shift as f32 * metrics.height));
        regions.push(ImageRegion {
            placements: &earlier.graphics.placements,
            origin,
            grid: Bounds::new(origin, grid.size),
            clip: Bounds::new(origin + band.origin, band.size).intersect(&pane),
        });
    }
    let origin = grid.origin + point(px(0.), px(offset));
    regions.push(ImageRegion {
        placements,
        origin,
        grid: Bounds::new(origin, grid.size),
        clip: Bounds::new(origin + local.origin, local.size).intersect(&pane),
    });
    // Snap both shared edges, rather than independently rounding origins and sizes.
    // GPUI otherwise expands content masks and can blend adjacent fragments twice.
    for region in &mut regions {
        region.clip = Bounds::from_corners(
            window.pixel_snap_point(region.clip.origin),
            window.pixel_snap_point(region.clip.bottom_right()),
        );
    }
    regions
}
