// ABOUTME: Keeps image metadata compatible with fractional pane scrolling.
// ABOUTME: Checks overlap boundaries, asset identity, and predecessor cleanup.
use super::*;
use sheprd_client::protocol::{
    SurfaceGraphicsAssetKey, SurfaceGraphicsFormat, SurfaceGraphicsPlacement, SurfaceGraphicsSource,
};

mod ownership;

fn image_at(x: u16, y: u16, cols: u32, rows: u32) -> SurfaceGraphicsPlacement {
    SurfaceGraphicsPlacement {
        asset: SurfaceGraphicsAssetKey {
            source: SurfaceGraphicsSource::PaneLayer {
                pane_id: "another-pane".into(),
                layer_id: "picture".into(),
            },
            image_width: 1,
            image_height: 1,
            format: SurfaceGraphicsFormat::Rgba,
            data_len: 4,
            data_fingerprint: 1,
        },
        logical_placement_id: 1,
        x,
        y,
        cols,
        rows,
        source_x: 0,
        source_y: 0,
        source_width: 0,
        source_height: 0,
        x_offset: 0,
        y_offset: 0,
        z: 0,
        scrollback_offset: 0,
    }
}

#[test]
fn image_geometry_does_not_disable_fractional_input() {
    for (x, y, cols, rows) in [(0, 0, 1, 1), (3, 5, 4, 4), (4, 0, 1, 1), (0, ROWS, 1, 1)] {
        let mut frame = back(10);
        Arc::make_mut(&mut frame)
            .graphics
            .placements
            .push(image_at(x, y, cols, rows));
        let mut scroll = SmoothScroll::default();
        assert_eq!(
            scroll.wheel(&frame, "pane", 0.25, Instant::now()),
            Some(1),
            "image at {x},{y} size {cols}x{rows}"
        );
    }
}

#[test]
fn image_metadata_can_enter_history_and_is_released_on_clear() {
    for image_in_previous in [false, true] {
        let mut previous = back(10);
        let mut next = back(11);
        let mut scroll = SmoothScroll::default();
        assert_eq!(
            scroll.wheel(&previous, "pane", 0.25, Instant::now()),
            Some(1)
        );
        let changed = if image_in_previous {
            &mut previous
        } else {
            &mut next
        };
        Arc::make_mut(changed)
            .graphics
            .placements
            .push(image_at(0, 0, 1, 1));
        let held = Arc::downgrade(&previous);
        scroll.observe(&previous, &next);
        assert!(
            scroll.slide(Instant::now() + SPREAD).is_some(),
            "image_in_previous={image_in_previous}"
        );
        drop(previous);
        assert!(
            held.upgrade().is_some(),
            "keep the historical cell and image metadata"
        );
        scroll.clear();
        assert!(held.upgrade().is_none(), "discard image-bearing history");
        Arc::make_mut(&mut next).graphics.placements.clear();
        assert_eq!(scroll.wheel(&next, "pane", 0.25, Instant::now()), Some(1));
    }
}
