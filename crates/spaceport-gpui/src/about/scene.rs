//! The About box's stage: the scene painted behind the credits, the marquee
//! that rides over them, and the clock both read.
//!
//! The credits themselves stay the plain column of messages [`about`] words;
//! the stage only paints under them, as decoration. Its time comes from the
//! app's motion clock through an animation element, so QA > Slow Animations
//! slows the scene tenfold like every other motion and reduced motion holds
//! it still, and both elements read one phase-locked animation, so the
//! marquee always rides the frame the canvas paints behind it.
//!
//! [`about`]: super

use super::effects::{LOOP, copper, plasma, ramp::mix, rotozoom, scroller, starfield};
use crate::config::Theme;
use crate::i18n::fl;
use gpui::{prelude::*, *};
use spaceport_motion::duration;
use std::{cell::Cell, ffi::OsStr, rc::Rc, time::Duration};

/// The variable that asks for the plain panel, for QA of the box without the
/// scene. Development and test builds only: a release build always shows the
/// scene.
pub(super) const CLASSIC: &str = "SPACEPORT_ABOUT_CLASSIC";
/// The stage's room above the mark, which the marquee rides in.
pub(super) const MARK_TOP: f32 = 20.;
/// How big the mark is drawn.
pub(super) const MARK_SIZE: f32 = 96.;
/// The band the marquee rides at the top of the stage.
pub(super) const BAND: f32 = 18.;
/// The stage's room below the credits.
pub(super) const ROOM_BOTTOM: f32 = 12.;
/// How far the rotozoomed field reaches past the mark.
const FRAME: f32 = 40.;
/// The most frames a second the scene asks for. A scene wants smoother
/// motion than a status mark, and the About box is the one place allowed to
/// burn a frame: the cost is a window scene rebuild per frame, about what a
/// working agent's mark costs at [`spaceport_motion::MARK_MAX_FPS`], and it
/// stops the moment the box closes, because the scene goes with it.
pub(super) const FPS: f32 = 30.;
/// How big the marquee's glyphs are, as a share of the window's text.
const GLYPH: f32 = 0.82;

/// The colours the stage paints with, taken from the window's theme so the
/// credits keep the contrast they have anywhere else: the ground is the
/// theme's background, the layers its ANSI brights, and a star the theme's
/// own ink.
#[derive(Clone, Copy, Debug)]
pub(super) struct Ink {
    pub ground: u32,
    pub field: [u32; 3],
    pub copper: [u32; 3],
    pub tile: [u32; 3],
    pub spark: u32,
}

impl Ink {
    pub(super) fn of(theme: &Theme) -> Self {
        let ground = theme.background;
        Self {
            ground,
            field: [theme.palette[12], theme.palette[14], theme.palette[13]],
            // A bar's dark end is the stage's own ground tinted copper, so it
            // grows out of the field instead of sitting on it.
            copper: [
                mix(ground, theme.palette[9], 0.35),
                theme.palette[9],
                theme.palette[11],
            ],
            tile: [
                mix(ground, theme.palette[12], 0.3),
                theme.palette[12],
                theme.palette[14],
            ],
            spark: theme.foreground,
        }
    }
}

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
pub(super) fn panel(credits: Div, theme: &Theme, text: f32, classic: bool) -> Div {
    if classic {
        return credits;
    }
    stage(credits, theme, text)
}

/// The credits laid over the scene, which paints behind them as decoration.
pub(super) fn stage(credits: Div, theme: &Theme, text: f32) -> Div {
    let ink = Ink::of(theme);
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
        .child(scene(ink, "about-scene-paint"))
        .child(marquee(greeting, theme.muted, text, "about-marquee-ride"))
        .child(credits)
}

/// The canvas the layers paint into, one frame of the scene at a time. Its
/// paint closure cannot be rebuilt each frame, so the animator passes the
/// frame's time through a cell the closure reads, as a measured element
/// passes its height.
fn scene(ink: Ink, id: &'static str) -> impl IntoElement {
    let clock = Rc::new(Cell::new(0.));
    let reading = clock.clone();
    canvas(
        |_, _, _| (),
        move |bounds, _, window, _| paint(ink, bounds, reading.get(), window),
    )
    .absolute()
    .inset_0()
    .with_animation(id, animation(), move |layer, delta| {
        clock.set(time(delta));
        layer
    })
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

/// The marquee riding the top band: the greeting as one element a glyph, so
/// each can take its own height from the wave. Nothing here carries a name of
/// its own, so a screen reader reads the credits below instead.
fn marquee(greeting: String, ink: u32, text: f32, id: &'static str) -> impl IntoElement {
    // A larger UI font keeps the line inside its band, so the crest of the
    // wave is never cut off.
    let size = (text * GLYPH).min(BAND - 2. * scroller::LIFT);
    let glyphs = letters(&greeting);
    let count = glyphs.len();
    div()
        .debug_selector(|| "about-marquee".into())
        .absolute()
        .top_0()
        .left_0()
        .w_full()
        .h(px(BAND))
        .overflow_hidden()
        .text_size(px(size))
        // A glyph's line box is its own size, so the panel's inherited line
        // height cannot push the baseline below the band and clip the crest.
        .line_height(px(size))
        .text_color(rgb(ink))
        .with_animation(id, animation(), move |band, delta| {
            let t = time(delta);
            band.children(
                glyphs
                    .iter()
                    .enumerate()
                    .filter_map(move |(index, letter)| {
                        let (x, lift) = scroller::glyph(index, count, t);
                        // A glyph past either edge of the stage is off screen.
                        (x > -scroller::ADVANCE && x < 1.).then(|| {
                            div()
                                .absolute()
                                .left(relative(x))
                                .top(px((BAND - size) / 2. - lift))
                                .child(letter.to_string())
                        })
                    }),
            )
        })
}

/// Paints one frame of the scene over `bounds`: the ground, the field, the
/// stars, the copper bars, and the rotozoomed field around the mark.
pub(super) fn paint(ink: Ink, bounds: Bounds<Pixels>, t: f32, window: &mut Window) {
    window.paint_quad(fill(bounds, rgb(ink.ground)));
    plasma::paint(&ink.field, bounds, t, window);
    starfield::paint(ink.spark, bounds, t, window);
    copper::paint(&ink.copper, bounds, t, window);
    rotozoom::paint(&ink.tile, framed(bounds), t, window);
}

/// The square the rotozoomed field fills: it stands on the mark's centre,
/// which the stage leaves [`MARK_TOP`] above, and reaches [`FRAME`] past each
/// of the mark's edges.
pub(super) fn framed(bounds: Bounds<Pixels>) -> Bounds<Pixels> {
    let side = MARK_SIZE + 2. * FRAME;
    let centre = point(
        bounds.origin.x + bounds.size.width / 2.,
        bounds.origin.y + px(MARK_TOP + MARK_SIZE / 2.),
    );
    Bounds::new(
        centre - point(px(side / 2.), px(side / 2.)),
        size(px(side), px(side)),
    )
}
