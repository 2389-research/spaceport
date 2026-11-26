//! The About box's demo pool: one demo a launch, each owning the whole stage.
//!
//! A demo is a paint arrangement over the stage and the marquee treatment
//! that goes with it, so five opens of the box are five different scenes
//! rather than one scene in five colours. The pick is made once, when the box
//! opens, and kept in the window, so a redraw never changes the demo under
//! the person reading it.
//!
//! `SPACEPORT_ABOUT_DEMO` pins one of them by name for QA and tests; without
//! it the pick comes from [`RandomState`]'s own seed, which is std's source
//! of process randomness and adds no dependency.

use super::effects::scroller::{COUNTER, RIDE, Wave};
use super::effects::{copper, plasma, rotozoom, starfield};
use crate::config::Theme;
use gpui::{Bounds, Pixels, Window};
use std::{
    collections::hash_map::RandomState,
    ffi::OsStr,
    hash::{BuildHasher, Hasher},
};

/// The variable that pins the demo, for QA and tests.
pub(super) const PIN: &str = "SPACEPORT_ABOUT_DEMO";

// keep-sorted start
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Demo {
    CopperBars,
    Plasma,
    Rotozoom,
    SineScroller,
    Starfield,
}
// keep-sorted end

/// Every demo the pool offers, in the order the pick walks them.
pub(super) const POOL: [Demo; 5] = [
    Demo::CopperBars,
    Demo::Plasma,
    Demo::Rotozoom,
    Demo::SineScroller,
    Demo::Starfield,
];

impl Demo {
    /// The demo `name` names, as [`PIN`] spells it.
    pub(super) fn named(name: &str) -> Option<Self> {
        let name = name.trim().to_ascii_lowercase();
        POOL.into_iter().find(|demo| demo.name() == name)
    }

    /// The name [`PIN`] gives this demo.
    pub(super) fn name(self) -> &'static str {
        match self {
            Self::CopperBars => "copper",
            Self::Plasma => "plasma",
            Self::Rotozoom => "rotozoom",
            Self::SineScroller => "scroller",
            Self::Starfield => "starfield",
        }
    }

    /// The room the marquee rides above the mark, in whole pixels at a window
    /// text size of `text`: the tallest band is the two-line scroller's. A
    /// whole number of pixels keeps the band and the mark below it on the
    /// pixel grid, so a test can name them exactly.
    pub(super) fn band(self, text: f32) -> f32 {
        let share = match self {
            Self::CopperBars => 1.6,
            Self::Plasma => 1.75,
            Self::Rotozoom => 2.2,
            Self::SineScroller => 3.6,
            Self::Starfield => 1.45,
        };
        (text * share).round()
    }

    /// The lines this demo's marquee is made of, top to bottom.
    pub(super) fn rides(self) -> &'static [Ride] {
        match self {
            Self::SineScroller => &COUNTER_MARQUEE,
            _ => &SINGLE_MARQUEE,
        }
    }
}

/// One line of a marquee: the wave it rides, and the size of its glyphs.
#[derive(Clone, Copy, Debug)]
pub(super) struct Ride {
    pub(super) wave: &'static Wave,
    /// The glyphs' size, as a share of the line's own height in the band.
    pub(super) size: f32,
}

/// The marquee most demos ride: one line, one wave.
const SINGLE_MARQUEE: [Ride; 1] = [Ride {
    wave: &RIDE,
    size: 0.55,
}];

/// The scroller's marquee: two lines crossing against each other, the
/// greeting itself the demo rather than a decoration over it.
const COUNTER_MARQUEE: [Ride; 2] = [
    Ride {
        wave: &RIDE,
        size: 0.45,
    },
    Ride {
        wave: &COUNTER,
        size: 0.45,
    },
];

/// The demo this launch shows: the one [`PIN`] names, and otherwise one of
/// the pool picked from `entropy`.
pub(super) fn choose_with(pinned: Option<&OsStr>, entropy: u64) -> Demo {
    pinned
        .and_then(OsStr::to_str)
        .and_then(Demo::named)
        .unwrap_or_else(|| POOL[(entropy % POOL.len() as u64) as usize])
}

/// The demo this launch shows.
pub(super) fn choose() -> Demo {
    choose_with(std::env::var_os(PIN).as_deref(), entropy())
}

/// A number that differs between processes: `RandomState` is seeded from the
/// process's own randomness, and hashing a constant still mixes that seed in.
fn entropy() -> u64 {
    let mut hasher = RandomState::new().build_hasher();
    hasher.write_u8(0);
    hasher.finish()
}

/// The colours the stage paints with, taken from the window's theme so the
/// credits keep the contrast they have anywhere else: the ground is the
/// theme's background, the layers its ANSI brights, and a star the theme's
/// own ink.
#[derive(Clone, Copy, Debug)]
pub(super) struct Ink {
    pub(super) ground: u32,
    pub(super) field: [u32; 3],
    pub(super) copper: [u32; 3],
    pub(super) tile: [u32; 3],
    pub(super) spark: u32,
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
                super::effects::ramp::mix(ground, theme.palette[9], 0.35),
                theme.palette[9],
                theme.palette[11],
            ],
            tile: [
                super::effects::ramp::mix(ground, theme.palette[12], 0.3),
                theme.palette[12],
                theme.palette[14],
            ],
            spark: theme.foreground,
        }
    }
}

/// Paints one frame of `demo` over `bounds`, `t` seconds into the loop. Each
/// demo paints the whole stage itself: its own layers over the ground, so no
/// two of them compose the same scene.
pub(super) fn paint(demo: Demo, ink: &Ink, bounds: Bounds<Pixels>, t: f32, window: &mut Window) {
    window.paint_quad(gpui::fill(bounds, gpui::rgb(ink.ground)));
    match demo {
        Demo::CopperBars => copper::paint(&ink.copper, bounds, t, window),
        Demo::Plasma => plasma::paint(&ink.field, bounds, t, window),
        Demo::Rotozoom => rotozoom::paint(&ink.tile, bounds, t, window),
        Demo::SineScroller => {
            // The quietest composition of the pool: the two-line marquee is
            // the demo, and the stage behind it only has to keep moving.
            plasma::paint(&ink.field, bounds, t, window);
            starfield::paint(ink.spark, bounds, t, window);
        }
        Demo::Starfield => starfield::paint(ink.spark, bounds, t, window),
    }
}
