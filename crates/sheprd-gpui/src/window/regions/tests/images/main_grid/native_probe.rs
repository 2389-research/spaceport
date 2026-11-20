// ABOUTME: Verifies nonvisual paint observations through the real terminal region renderer.
// ABOUTME: Requires actual cell callbacks, decoded image uploads, and unchanged cached replay.
use super::*;
use crate::window::native_images;

#[gpui::test]
fn nonvisual_probe_sees_real_uploads_and_cached_cell_replay(cx: &mut TestAppContext) {
    let atlas = Session::start();
    let (view, cx) = cx.add_window_view(fixture_window);
    let window = cx.update(|window, _| window.window_handle().window_id());
    let probe = native_images::Session::start(window).unwrap();
    let mut surface = surface();
    let first = draw(&view, &surface, cx);
    assert!(
        probe.frame().unwrap().cells > 0,
        "initial cells must really paint"
    );

    let (placed, encoded) = asset(main_source(1), 91, 2);
    surface.graphics.placements.push(placed);
    install(&view, vec![encoded], cx);
    for _ in 0..3 {
        draw(&view, &surface, cx);
        cx.run_until_parked();
    }
    probe.reset().unwrap();
    atlas.reset();
    assert_eq!(draw(&view, &surface, cx), first);
    let replay = probe.frame().unwrap();
    assert_eq!(
        replay.cells, 0,
        "unchanged cells must replay their cached scene"
    );
    assert!(!replay.overflow);
    assert_eq!(replay.images.len(), 1);
    let image = &replay.images[0];
    assert!(image.painted, "the real image paint must succeed");
    let uploaded = atlas.painted();
    assert_eq!(uploaded.len(), replay.images.len());
    assert!(uploaded[0].uploaded, "the headless atlas entry must exist");
    assert_eq!(image.id, uploaded[0].texture.id);
    assert_eq!(image.z, 2);
    assert!(image.visible.size.width > px(0.));
    assert!(image.visible.size.height > px(0.));
    assert!(image.visible.intersect(&image.mask).size.width > px(0.));
    assert!(image.image.size.width >= image.visible.size.width);
    let identity = image.id;
    probe.reset().unwrap();
    draw(&view, &surface, cx);
    assert_eq!(probe.frame().unwrap().images[0].id, identity);
    drop(probe);
    assert!(
        native_images::Session::start(window).is_ok(),
        "dropping a probe releases its window claim"
    );
}
