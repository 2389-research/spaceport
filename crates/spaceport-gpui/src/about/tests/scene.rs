//! The stage under the About box: the scene behind the credits, the marquee
//! over them, and the clock they read.

use super::*;
use crate::about::effects::LOOP;
use crate::about::scene;
use crate::config::Theme;
use crate::sidebar::layout_tests::run_frames;
use std::cell::Cell;
use std::ffi::OsStr;
use std::rc::Rc;
use std::time::Duration;

/// The scene covers the credits column it is painted behind, and the
/// marquee rides the room the column leaves above the mark.
#[gpui::test]
fn the_scene_covers_the_credits_and_the_marquee_rides_its_top(cx: &mut TestAppContext) {
    let (view, cx) = cx.add_window_view(crate::sidebar::layout_tests::fixture_window);
    cx.update(|window, cx| view.update(cx, |view, cx| view.open_about(window, cx)));
    cx.update(|window, cx| window.draw(cx).clear(cx));
    let stage = cx.debug_bounds("about-scene").expect("the stage is drawn");
    let about = cx.debug_bounds("about").expect("the credits are drawn");
    assert_eq!(stage, about);
    let marquee = cx
        .debug_bounds("about-marquee")
        .expect("the marquee rides the stage");
    assert_eq!(marquee.top(), stage.top());
    assert_eq!(marquee.size.height, px(scene::BAND));
    assert_eq!(marquee.size.width, stage.size.width);
}

/// The rotozoomed field is centred on the mark the credits draw, so the
/// stage's own room above the mark and the mark's place are one agreement.
#[gpui::test]
fn the_field_frames_the_mark_the_credits_draw(cx: &mut TestAppContext) {
    let (view, cx) = cx.add_window_view(crate::sidebar::layout_tests::fixture_window);
    cx.update(|window, cx| view.update(cx, |view, cx| view.open_about(window, cx)));
    cx.update(|window, cx| window.draw(cx).clear(cx));
    let stage = cx.debug_bounds("about-scene").expect("the stage is drawn");
    let mark = cx.debug_bounds("about-icon").expect("the mark is drawn");
    assert_eq!(mark.size, size(px(scene::MARK_SIZE), px(scene::MARK_SIZE)));
    assert_eq!(scene::framed(stage).center(), mark.center());
}

/// A frame's share of the loop is the scene's own clock: no wall clock is
/// read, and slow motion stretches the loop exactly as every other motion is
/// stretched.
#[test]
fn the_scene_reads_its_time_from_the_animation() {
    assert_eq!(scene::time(0.), 0.);
    assert_eq!(scene::time(0.5), LOOP / 2.);
    assert_eq!(scene::time(1.), LOOP);
    assert_eq!(
        spaceport_motion::duration(Duration::from_secs_f32(LOOP)),
        Duration::from_secs_f32(LOOP)
    );
}

/// The panel the debug route asks for is the credits alone — no stage, no
/// marquee — while the shipped panel keeps both behind the same column.
#[gpui::test]
fn the_plain_panel_draws_the_credits_without_a_stage(cx: &mut TestAppContext) {
    for classic in [true, false] {
        let (_, cx) = cx.add_window_view(|_, _| CreditsPanel { classic });
        cx.simulate_resize(size(px(360.), px(600.)));
        cx.update(|window, cx| window.draw(cx).clear(cx));
        assert!(
            cx.debug_bounds("about-credits").is_some(),
            "the credits are drawn either way"
        );
        assert_eq!(cx.debug_bounds("about-scene").is_some(), !classic);
        assert_eq!(cx.debug_bounds("about-marquee").is_some(), !classic);
    }
}

/// The credits column a test lays out through the route the About box uses,
/// with a marked placeholder standing in for the panel's own words.
struct CreditsPanel {
    classic: bool,
}

impl Render for CreditsPanel {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let credits = div()
            .debug_selector(|| "about-credits".into())
            .w(px(320.))
            .h(px(480.));
        scene::panel(credits, &Theme::default(), 13., self.classic)
    }
}

#[test]
fn the_plain_panel_is_asked_for_by_a_variable_in_a_development_build() {
    let asked = Some(OsStr::new("1"));
    assert!(scene::classic_for(asked, true));
    assert!(!scene::classic_for(None, true));
    // A release build shows the scene, whatever the environment says.
    assert!(!scene::classic_for(asked, false));
}

/// The scene asks for its next frame as the clock runs, and nothing asks
/// under reduced motion: GPUI holds the scene on its first frame.
#[gpui::test]
fn the_scene_keeps_frames_coming_unless_motion_is_reduced(cx: &mut TestAppContext) {
    for reduced in [false, true] {
        cx.update(|cx| cx.set_reduce_motion(reduced));
        let (view, cx) = cx.add_window_view(crate::sidebar::layout_tests::fixture_window);
        cx.update(|window, cx| view.update(cx, |view, cx| view.open_about(window, cx)));
        cx.update(|window, cx| window.draw(cx).clear(cx));
        let frames = Rc::new(Cell::new(0));
        let counted = frames.clone();
        cx.update(|_, cx| {
            cx.observe(&view, move |_, _| counted.set(counted.get() + 1))
                .detach()
        });
        // Long enough for the scene's next frame, four times.
        run_frames(cx, 4, Duration::from_millis(60));
        let asked = frames.get();
        if reduced {
            assert_eq!(asked, 0, "reduced motion asked for {asked} frames");
        } else {
            assert!(asked >= 4, "the scene asked for {asked} frames");
        }
    }
}
