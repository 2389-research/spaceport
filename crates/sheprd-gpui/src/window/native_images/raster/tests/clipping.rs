// ABOUTME: Rejects clipping evidence that omits its image or one side of the viewport boundary.
// ABOUTME: Uses actual RGBA buffers to test the capture validator's acceptance contract.
use super::*;
use gpui::{point, px, size};

fn pane() -> Bounds<Pixels> {
    Bounds::new(point(px(0.), px(0.)), size(px(16.), px(16.)))
}
fn edge() -> Bounds<Pixels> {
    Bounds::new(point(px(4.), px(12.)), size(px(4.), px(8.)))
}
fn pixels(height: u32) -> RgbaImage {
    let mut image = RgbaImage::from_pixel(16, height, image::Rgba([0, 0, 0, 255]));
    for y in 12..16 {
        for x in 4..8 {
            image.put_pixel(x, y, image::Rgba([0, 255, 255, 255]));
        }
    }
    image
}

#[test]
fn clipping_evidence_requires_the_edge_image_to_be_visible() {
    let missing = RgbaImage::from_pixel(16, 20, image::Rgba([0, 0, 0, 255]));
    assert!(
        clipping(&missing, 1., pane(), edge(), pane(), true).is_err(),
        "a completely missing image must not pass clipping qualification"
    );
}

#[test]
fn clipping_evidence_requires_both_sides_and_a_crossing_mapping() {
    assert!(clipping(&pixels(20), 1., pane(), edge(), pane(), true).is_ok());
    assert!(
        clipping(&pixels(16), 1., pane(), edge(), pane(), true).is_err(),
        "an unavailable outside sample must not pass clipping qualification"
    );
    let inside = Bounds::new(point(px(4.), px(12.)), size(px(4.), px(4.)));
    assert!(
        clipping(&pixels(20), 1., pane(), inside, pane(), true).is_err(),
        "fractional clipping requires a mapping that actually crosses the viewport"
    );
}
