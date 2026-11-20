// ABOUTME: Fences fractional history when its existing image assets change identity.
// ABOUTME: Preserves gestures across cloned asset maps without leasing encoded bytes.
use super::*;
use sheprd_client::protocol::SurfaceGraphicsAsset;

fn images(placement: &SurfaceGraphicsPlacement) -> SurfaceImages {
    [SurfaceGraphicsAsset {
        key: placement.asset.clone(),
        data: vec![10, 20, 30, 255],
    }]
    .into_iter()
    .collect()
}

#[test]
fn cloned_assets_preserve_history_but_replacement_serials_cancel_it() {
    let mut previous = back(10);
    let placed = image_at(0, 0, 1, 1);
    let assets = images(&placed);
    Arc::make_mut(&mut previous)
        .graphics
        .placements
        .push(placed.clone());
    let mut scroll = SmoothScroll::default();
    assert_eq!(
        scroll.wheel(&previous, "pane", 0.25, Instant::now()),
        Some(1)
    );
    scroll.observe(&previous, &back(11));
    let next = assets.clone();
    scroll.retain_images(&back(11), &assets, &next);
    assert!(scroll.slide(Instant::now() + SPREAD).is_some());
    scroll.retain_images(&back(11), &next, &images(&placed));
    assert!(scroll.slide(Instant::now() + SPREAD).is_none());
}

#[test]
fn missing_or_newly_available_assets_cancel_the_gesture() {
    for appearing in [false, true] {
        let mut previous = back(10);
        let placed = image_at(0, 0, 1, 1);
        let assets = images(&placed);
        Arc::make_mut(&mut previous)
            .graphics
            .placements
            .push(placed);
        let mut scroll = SmoothScroll::default();
        assert_eq!(
            scroll.wheel(&previous, "pane", 0.25, Instant::now()),
            Some(1)
        );
        let empty = SurfaceImages::default();
        let (before, next) = if appearing {
            (&empty, &assets)
        } else {
            (&assets, &empty)
        };
        scroll.retain_images(&previous, before, next);
        assert!(scroll.slide(Instant::now() + SPREAD).is_none());
    }
}

#[test]
fn asset_changes_outside_the_scrolling_pane_preserve_its_gesture() {
    let mut previous = back(10);
    let placed = image_at(4, 0, 1, 1);
    let assets = images(&placed);
    Arc::make_mut(&mut previous)
        .graphics
        .placements
        .push(placed);
    let mut scroll = SmoothScroll::default();
    assert_eq!(
        scroll.wheel(&previous, "pane", 0.25, Instant::now()),
        Some(1)
    );
    scroll.retain_images(&previous, &assets, &SurfaceImages::default());
    assert!(scroll.slide(Instant::now() + SPREAD).is_some());
}
