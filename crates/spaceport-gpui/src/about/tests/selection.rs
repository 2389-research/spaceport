//! The demo pool: which demo a launch shows, and what each one owns.

use super::*;
use std::ffi::OsStr;

/// The pinned demo wins, whatever the pick would have been, and every demo in
/// the pool can be named.
#[test]
fn the_pinned_demo_wins() {
    for demo in demos::POOL {
        for entropy in [0, 1, u64::MAX] {
            assert_eq!(
                demos::choose_with(Some(OsStr::new(demo.name())), entropy),
                demo
            );
        }
        assert_eq!(Demo::named(demo.name()), Some(demo));
    }
    // A name the pool does not offer, a value that is not text, and no pin at
    // all each fall back to the pick.
    for pinned in [Some(OsStr::new("no-such-demo")), Some(OsStr::new(""))] {
        assert!(demos::POOL.contains(&demos::choose_with(pinned, 3)));
    }
    assert!(demos::POOL.contains(&demos::choose_with(None, 3)));
}

/// Every whole number lands on a demo, so the pick is total.
#[test]
fn every_entropy_lands_on_a_demo() {
    let mut seen: Vec<Demo> = Vec::new();
    for entropy in 0..demos::POOL.len() as u64 {
        let demo = demos::choose_with(None, entropy);
        assert!(!seen.contains(&demo), "{demo:?} twice");
        seen.push(demo);
    }
    assert_eq!(
        seen.len(),
        demos::POOL.len(),
        "the pick walks the whole pool"
    );
    for entropy in 0..1_000 {
        assert!(demos::POOL.contains(&demos::choose_with(None, entropy)));
    }
}

/// No two demos share a stage: each leaves its own room for the marquee, and
/// the scroller's is the two-line one.
#[test]
fn every_demo_owns_its_own_stage() {
    for size in [11., 13., 20.] {
        let mut bands: Vec<String> = demos::POOL
            .iter()
            .map(|demo| format!("{:.2}", demo.band(size)))
            .collect();
        bands.sort();
        bands.dedup();
        assert_eq!(bands.len(), demos::POOL.len(), "at {size}");
    }
    assert_eq!(
        demos::POOL
            .iter()
            .filter(|demo| demo.rides().len() > 1)
            .copied()
            .collect::<Vec<_>>(),
        [Demo::SineScroller]
    );
}

/// The demo the box opens with is the one it keeps: a redraw does not roll
/// again, so the scene never changes under the reader.
#[gpui::test]
fn a_redraw_keeps_the_demo_the_box_opened_with(cx: &mut TestAppContext) {
    let (view, cx) = cx.add_window_view(crate::sidebar::layout_tests::fixture_window);
    for demo in demos::POOL {
        open_about_with(&view, demo, cx);
        cx.update(|window, cx| window.draw(cx).clear(cx));
        for _ in 0..3 {
            cx.update(|window, cx| window.draw(cx).clear(cx));
            view.read_with(cx, |view, _| assert_eq!(view.about_demo, demo));
        }
        let riding = view.read_with(cx, |view, _| view.about_demo.rides().len());
        let band = cx.debug_bounds("about-marquee").unwrap().size.height;
        let text = text_size(&view, cx);
        assert_eq!(band, px(demo.band(text)));
        assert_eq!(riding, demo.rides().len());
    }
}

/// Two pinned demos draw two different stages in the same window.
#[gpui::test]
fn two_pinned_demos_draw_two_stages(cx: &mut TestAppContext) {
    let (view, cx) = cx.add_window_view(crate::sidebar::layout_tests::fixture_window);
    let mut bands = Vec::new();
    for demo in [Demo::Starfield, Demo::SineScroller] {
        open_about_with(&view, demo, cx);
        cx.update(|window, cx| window.draw(cx).clear(cx));
        let marquee = cx.debug_bounds("about-marquee").unwrap();
        bands.push(marquee.size.height);
    }
    assert_ne!(bands[0], bands[1]);
}
