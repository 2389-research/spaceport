// ABOUTME: Splits cross-pane images into stationary and fractional fragments.
// ABOUTME: Checks shared device-pixel edges and equal-z order at fractional display scales.
use super::*;

#[gpui::test]
fn crossing_images_keep_equal_z_order_and_nonoverlapping_masks(cx: &mut TestAppContext) {
    let session = Session::start();
    let (view, cx) = cx.add_window_view(fixture_window);
    for scale in [1., 1.25, 1.5, 2.] {
        cx.simulate_scale_factor_change(scale);
        view.update(cx, |view, _| view.presentation.scroll.clear());
        let mut surface = scrolling_surface();
        let mut assets = Vec::new();
        for (id, value) in [(1, 91), (2, 92)] {
            let (mut placed, mut encoded) = asset(main_source(id), value, 2);
            placed.x = 2;
            placed.cols = 5;
            encoded.data[3] = 128;
            surface.graphics.placements.push(placed);
            assets.push(encoded);
        }
        install(&view, assets, cx);
        settled(&view, &surface, &session, cx);
        wheel(&view, 0.25, cx);
        advance(&mut surface);
        for placed in &mut surface.graphics.placements {
            placed.y = 1;
        }
        session.reset();
        draw(&view, &surface, cx);
        let actual = session.painted();
        assert_eq!(
            actual.len(),
            4,
            "two stationary and two moving fragments at scale {scale}"
        );
        assert_eq!(
            actual
                .iter()
                .map(|image| pixels(image)[2])
                .collect::<Vec<_>>(),
            [91, 92, 91, 92]
        );
        let stationary = &actual[..2];
        let moving = &actual[2..];
        assert!(moving[0].bounds.top() < stationary[0].bounds.top());
        assert_eq!(
            moving[0].bounds.right(),
            stationary[0].bounds.left(),
            "shared mask edge at scale {scale}"
        );
        assert_eq!(moving[1].bounds.right(), stationary[1].bounds.left());
        // Reconstructing clipped rectangles rounds f32 operations independently.
        // Keep mask edges exact above and bound mapping roundoff to four epsilons.
        for (moving, stationary) in [
            (moving[0].mapped.size.width, stationary[0].mapped.size.width),
            (
                moving[0].mapped.size.height,
                stationary[0].mapped.size.height,
            ),
        ] {
            let (moving, stationary) = (f32::from(moving), f32::from(stationary));
            assert!(
                (moving - stationary).abs() <= stationary.abs() * f32::EPSILON * 4.,
                "both paint calls preserve full texture mapping at scale {scale}: {moving} vs {stationary}"
            );
        }
        assert_eq!(session.preparations().len(), 1);
    }
}
