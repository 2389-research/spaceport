//! The About box's stage: the scene painted behind the credits, the marquee
//! that rides over them, and the clock both read.
//!
//! The credits themselves stay the NFO document [`nfo`] lays out; the stage
//! only paints under them, as decoration. Which demo paints is [`demos`]'s
//! pick, kept in the window for as long as the box is open. Its time comes
//! from the app's motion clock through an animation element, so QA > Slow
//! Animations slows the scene tenfold like every other motion and reduced
//! motion holds it still, and both elements read one phase-locked animation,
//! so the marquee always rides the frame the canvas paints behind it.
//!
//! [`nfo`]: super::nfo

use super::demos::{self, Demo, Ink, Ride};
use super::effects::{LOOP, scroller};
use crate::config::Theme;
use crate::i18n::fl;
use gpui::{prelude::*, *};
use spaceport_motion::duration;
use std::{cell::Cell, ffi::OsStr, rc::Rc, time::Duration};

/// The variable that asks for the plain panel, for QA of the box without the
/// scene. Development and test builds only: a release build always shows the
/// scene.
pub(super) const CLASSIC: &str = "SPACEPORT_ABOUT_CLASSIC";
/// The stage's room below the credits.
pub(super) const ROOM_BOTTOM: f32 = 12.;
/// The room the stage leaves above the marquee's band, so it sits off the
/// panel's own edge.
const BAND_TOP: f32 = 2.;
/// The most frames a second the scene asks for. A scene wants smoother
/// motion than a status mark, and the About box is the one place allowed to
/// burn a frame: the cost is a window scene rebuild per frame, about what a
/// working agent's mark costs at [`spaceport_motion::MARK_MAX_FPS`], and it
/// stops the moment the box closes, because the scene goes with it.
pub(super) const FPS: f32 = 30.;

/// Whether the plain panel is asked for: the variable is set, and this is a
/// build that keeps a way back to it.
pub(super) fn classic() -> bool {
    classic_for(std::env::var_os(CLASSIC).as_deref(), cfg!(debug_assertions))
}

/// The same question asked of a value, so a test can pose it without
/// reaching for the process's environment.
pub(super) fn classic_for(value: Option<&OsStr>, development: bool) -> bool {
    development && value.is_some()
}

/// The seconds of the scene's loop `delta` — the share of it one animation
/// frame has run — has reached.
pub(super) fn time(delta: f32) -> f32 {
    delta * LOOP
}

/// The room the credits leave above the mark: the demo's marquee band, and
/// the few pixels that keep it off the panel's edge.
pub(super) fn margin(demo: Demo, text: f32) -> f32 {
    demo.band(text) + BAND_TOP
}

/// The one animation both stage elements read: a loop of the scene,
/// phase-locked to the app's clock so canvas and marquee agree, at the
/// scene's frame rate.
fn animation() -> Animation {
    Animation::new(duration(Duration::from_secs_f32(LOOP)))
        .repeat_synced()
        .with_max_fps(FPS)
}

/// The box's panel: the credits alone when the plain panel was asked for,
/// and the credits over the scene otherwise.
pub(super) fn panel(credits: Div, theme: &Theme, text: f32, classic: bool, demo: Demo) -> Div {
    if classic {
        return credits;
    }
    stage(credits, theme, text, demo)
}

/// The credits laid over the scene, which paints behind them as decoration.
pub(super) fn stage(credits: Div, theme: &Theme, text: f32, demo: Demo) -> Div {
    let greeting = fl!(
        "about-marquee",
        publisher = super::PUBLISHER,
        name = crate::constants::WINDOW_TITLE
    );
    div()
        .debug_selector(|| "about-scene".into())
        .relative()
        .flex()
        .flex_col()
        .w_full()
        .overflow_hidden()
        .child(scene(Ink::of(theme), demo, text, "about-scene-paint"))
        .child(marquee(
            greeting,
            theme.muted,
            text,
            demo,
            "about-marquee-ride",
        ))
        .child(credits)
}

/// The canvas the demo paints into, one frame of the scene at a time. Its
/// paint closure cannot be rebuilt each frame, so the animator passes the
/// frame's time through a cell the closure reads, as a measured element
/// passes its height.
fn scene(ink: Ink, demo: Demo, text: f32, id: &'static str) -> impl IntoElement {
    let clock = Rc::new(Cell::new(0.));
    let reading = clock.clone();
    canvas(
        |_, _, _| (),
        move |bounds, _, window, _| {
            demos::paint(
                demo,
                &ink,
                mark_bounds(bounds, demo, text),
                bounds,
                reading.get(),
                window,
            );
        },
    )
    .absolute()
    .inset_0()
    .with_animation(id, animation(), move |layer, delta| {
        clock.set(time(delta));
        layer
    })
}

/// The square the credits' mark stands on: the column leaves [`margin`]
/// above it and gives it [`super::nfo::art_height`], so the stage paints the
/// mark's own square in the same place.
pub(super) fn mark_bounds(bounds: Bounds<Pixels>, demo: Demo, text: f32) -> Bounds<Pixels> {
    let side = super::nfo::mark_height(text);
    Bounds::new(
        point(
            bounds.origin.x + (bounds.size.width - px(side)) / 2.,
            bounds.origin.y + px(margin(demo, text)),
        ),
        size(px(side), px(side)),
    )
}

/// The glyphs of the marquee's line, without the characters that take no room
/// on screen. Fluent isolates the greeting's placeables in U+2068/U+2069 and a
/// pseudo-locale wraps the line again, so the line arrives carrying bidi
/// controls: given a slot each, they would leave gaps in the wave. They go by
/// the campaign's own class of untrusted characters, which `spaceport_truncate`
/// drops from every other line the app shows, so what a line loses is one
/// answer and not a second list here.
pub(super) fn letters(greeting: &str) -> Vec<char> {
    greeting
        .chars()
        .filter(|letter| !spaceport_client::text::unsafe_char(*letter))
        .collect()
}

/// The marquee riding the top band: one line for most demos, and the
/// scroller's two. Nothing here carries a name of its own, so a screen reader
/// reads the credits below instead.
fn marquee(
    greeting: String,
    ink: u32,
    text: f32,
    demo: Demo,
    id: &'static str,
) -> impl IntoElement {
    let riding = demo.rides();
    let glyphs = letters(&greeting);
    div()
        .debug_selector(|| "about-marquee".into())
        .absolute()
        .top(px(BAND_TOP))
        .left_0()
        .w_full()
        .h(px(demo.band(text)))
        .overflow_hidden()
        .text_color(rgb(ink))
        .with_animation(id, animation(), move |band, delta| {
            let t = time(delta);
            // Each line owns an equal share of the band, so a demo's second
            // line never overlaps its first.
            let share = 1. / riding.len() as f32;
            let height = demo.band(text) * share;
            band.children(
                riding
                    .iter()
                    .enumerate()
                    .map(|(index, ride)| line(&glyphs, ride, height, index, share, t))
                    .collect::<Vec<_>>(),
            )
        })
}

/// One line of the marquee, riding `ride`'s wave `height` pixels tall, the
/// `index`th of the band's `share`-sized shares.
fn line(glyphs: &[char], ride: &Ride, height: f32, index: usize, share: f32, t: f32) -> Div {
    let size = (height * ride.size).max(1.);
    let count = glyphs.len();
    // The line's middle, less half a glyph box, is where its text box starts.
    let baseline = (height - size) / 2.;
    div()
        .absolute()
        .left_0()
        .top(relative(index as f32 * share))
        .w_full()
        .h(px(height))
        .text_size(px(size))
        // A glyph's line box is its own size, so the panel's inherited line
        // height cannot push the baseline below the band and clip the crest.
        .line_height(px(size))
        .children(
            glyphs
                .iter()
                .enumerate()
                .filter_map(move |(index, letter)| {
                    let (x, lift) = scroller::glyph(ride.wave, index, count, t);
                    // A glyph past either edge of the stage is off screen.
                    (x > -scroller::ADVANCE && x < 1.).then(|| {
                        div()
                            .absolute()
                            .left(relative(x))
                            .top(px(baseline - lift))
                            .child(letter.to_string())
                    })
                }),
        )
}
