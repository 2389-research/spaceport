//! The stage under the About box: the demo's scene behind the credits, the
//! marquee over them, and the clock they read.

use super::*;
use crate::about::demos;
use crate::about::effects::LOOP;
use crate::about::scene;
use crate::about::scene::margin;
use crate::config::Theme;
use crate::sidebar::layout_tests::run_frames;
use std::cell::Cell;
use std::ffi::OsStr;
use std::rc::Rc;
use std::time::Duration;

/// The scene covers the credits column it is painted behind, and the marquee
/// rides the room the column leaves above the mark, as tall as the demo's
/// band.
#[gpui::test]
fn the_scene_covers_the_credits_and_the_marquee_rides_its_top(cx: &mut TestAppContext) {
    let (view, cx) = cx.add_window_view(crate::sidebar::layout_tests::fixture_window);
    for demo in demos::POOL {
        open_about_with(&view, demo, cx);
        cx.update(|window, cx| window.draw(cx).clear(cx));
        let text = text_size(&view, cx);
        let stage = cx.debug_bounds("about-scene").expect("the stage is drawn");
        let about = cx.debug_bounds("about").expect("the credits are drawn");
        assert_eq!(stage, about);
        let marquee = cx
            .debug_bounds("about-marquee")
            .expect("the marquee rides the stage");
        assert_eq!(marquee.top(), stage.top() + px(2.));
        assert_eq!(marquee.size.height, px(demo.band(text)));
        assert_eq!(marquee.size.width, stage.size.width);
    }
}

/// The mark the credits draw is the wordmark's own square, standing in the
/// room the demo's band leaves, so the stage and the document agree on where
/// the art goes whatever the demo paints behind it.
#[gpui::test]
fn the_mark_stands_in_the_room_the_credits_leave_it(cx: &mut TestAppContext) {
    let (view, cx) = cx.add_window_view(crate::sidebar::layout_tests::fixture_window);
    for demo in demos::POOL {
        open_about_with(&view, demo, cx);
        cx.update(|window, cx| window.draw(cx).clear(cx));
        let text = text_size(&view, cx);
        let stage = cx.debug_bounds("about-scene").expect("the stage is drawn");
        let about = cx.debug_bounds("about").expect("the credits are drawn");
        let mark = cx.debug_bounds("about-icon").expect("the mark is drawn");
        assert_eq!(stage, about);
        assert_eq!(mark.size.height, px(wordmark_height(text)));
        assert!(
            mark.size.width < stage.size.width,
            "the art is its own width"
        );
        assert_eq!(mark.top(), about.top() + px(margin(demo, text)));
        assert_eq!(mark.center().x, stage.center().x);
    }
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
        scene::panel(
            credits,
            &Theme::default(),
            13.,
            self.classic,
            Demo::Starfield,
        )
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
        open_about_with(&view, Demo::Plasma, cx);
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

/// A bidi control takes no room on the greeting's line, so the marquee must
/// give it no glyph slot either: Fluent isolates the placeables in U+2068 and
/// U+2069, and a pseudo-locale wraps the line again, and either would leave a
/// gap in the wave if the control took a slot.
#[test]
fn the_marquee_gives_bidi_controls_no_glyph_slot() {
    let plain = "2389 Research, Inc. presents Spaceport";
    let wrapped = "\u{2067}2389 Research, Inc.\u{200e} presents Spaceport\u{2069}";
    assert_eq!(scene::letters(wrapped), scene::letters(plain));
    assert_eq!(scene::letters(wrapped).len(), plain.chars().count());
}

/// The wordmark is block letters and gaps alone, drawn on the pixel grid:
/// nothing in it is a word, so it needs no translation and a screen reader
/// reads the credits instead.
#[test]
fn the_wordmark_is_block_letters_and_gaps() {
    assert_eq!(nfo::mark_size(13.), nfo::mark_size(13.).round());
    for line in nfo::MARK {
        assert!(
            line.chars().all(|cell| cell == '\u{2588}' || cell == ' '),
            "{line}"
        );
    }
}
