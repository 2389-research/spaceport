// ABOUTME: Cancels actual window scroll history before a replacement image generation paints.
// ABOUTME: Covers identity replacement even when the terminal surface pointer stays unchanged.
use super::*;

#[gpui::test]
fn replacing_an_asset_cancels_the_slide_before_presenting_new_pixels(cx: &mut TestAppContext) {
    let session = Session::start();
    let (view, cx) = cx.add_window_view(fixture_window);
    let mut surface = scrolling_surface();
    let (placed, encoded) = asset(main_source(1), 91, 2);
    let key = placed.asset.clone();
    surface.graphics.placements.push(placed);
    install(&view, vec![encoded], cx);
    settled(&view, &surface, &session, cx);
    wheel(&view, 0.25, cx);
    advance(&mut surface);
    surface.graphics.placements[0].y = 1;
    draw(&view, &surface, cx);
    assert!(
        view.read_with(cx, |view, _| view.presentation.scroll.offset())
            .is_some()
    );
    // A new delivery can reuse its wire key; its process-local serial distinguishes it.
    install(
        &view,
        vec![SurfaceGraphicsAsset {
            key,
            data: vec![92, 20, 30, 255],
        }],
        cx,
    );
    view.update(cx, |view, _| {
        let same = view.live.surface.as_ref().unwrap().clone();
        view.presentation.frame(&view.live);
        assert!(Arc::ptr_eq(view.live.surface.as_ref().unwrap(), &same));
        assert!(view.presentation.scroll.offset().is_none());
    });
    settled(&view, &surface, &session, cx);
    let painted = session.painted();
    assert_eq!(painted.len(), 1);
    assert_eq!(pixels(&painted[0]), &[30, 20, 92, 255]);
}
