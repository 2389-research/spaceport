use super::*;

/// At 150% display scale the row's edges snap to device pixels, so the label
/// lands a third of a logical pixel short of its unsnapped width. Windows
/// desktops default to fractional scales; a whole pixel is still a failure.
#[test]
fn child_probe_allows_less_than_one_device_pixel_of_snapping() {
    use crate::config::{Density, LayoutMode};
    let mode = LayoutMode::from(Density::Normal);
    let bounds = Bounds::new(point(px(44.), px(100.)), size(px(179.), px(16.)));
    let mut probe = PaintedText {
        bounds,
        mask: bounds,
        cached: "sidebar-child".into(),
        glyph_text: "sidebar-child".into(),
        width: px(94.),
        clipped: false,
    };
    let panel = crate::sidebar::SIDEBAR_WIDTH;
    probe.bounds.size.width = px(268. / 1.5);
    probe.mask = probe.bounds;
    probe
        .verify_child("sidebar-child", mode, panel, 0., 1.5)
        .unwrap();
    probe.bounds.size.width = px(178.);
    probe.mask = probe.bounds;
    assert!(
        probe
            .verify_child("sidebar-child", mode, panel, 0., 1.5)
            .unwrap_err()
            .to_string()
            .contains("child label width: actual")
    );
}

/// The child row is laid out at the width the frame measured, which a resize
/// or a chosen width changes. The probe used to measure every frame against the
/// default width, and the native fixture skipped the rows it could not match,
/// so nothing checked a row painted while the sidebar was resized.
#[test]
fn child_probe_follows_the_frame_s_panel_width() {
    use crate::config::{Density, LayoutMode};
    let mode = LayoutMode::from(Density::Normal);
    let look = crate::sidebar::layout::for_mode(mode);
    let layout = look.density;
    let resized = 320.;
    let bounds = Bounds::new(
        point(px(44.), px(100.)),
        size(
            px(look.content_width(resized)
                - crate::sidebar::STATUS_WIDTH
                - layout.gap()
                - layout.child_indent()),
            px(16.),
        ),
    );
    let probe = PaintedText {
        bounds,
        mask: bounds,
        cached: "sidebar-child".into(),
        glyph_text: "sidebar-child".into(),
        width: bounds.size.width,
        clipped: false,
    };
    probe
        .verify_child("sidebar-child", mode, resized, 0., 1.)
        .unwrap();
    // The same row is wrong for the default panel, which is what made the
    // fixture paint its resized frames unverified.
    assert!(
        probe
            .verify_child("sidebar-child", mode, crate::sidebar::SIDEBAR_WIDTH, 0., 1.)
            .unwrap_err()
            .to_string()
            .contains("child label width: actual")
    );
}
