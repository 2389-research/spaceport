# The About box's demoscene stage

What issue [#245] asked for: an About box with the energy of a cracktro's
ABOUT key — old-school effects, real-time, unmistakably 2389 Research — while
staying an About box: same version, same links, same credits, same Escape-to-
close.

The owner's ruling (`notes-owner-decisions-2026-11-25.md`, item 4) is
**shipped in every build, no sound**. That is what this change does: the scene
is in release builds, and nothing in it makes a noise.

## What the box is now

`about.rs` is still the panel: `render_about` builds the same credits column it
always built — icon, name, version, publisher, repository link, summary,
upstream credits, the unaffiliated line, a worktree build's branch and PR, the
team links, the modifications and original-copyright lines, the license and
notices links, OK — with every word unchanged. `render_about` hands that
column to `scene::panel`, which lays it over `scene::stage` so the scene paints
behind it, and hands it back untouched when the plain panel is asked for.

The credits are ordinary text elements, laid out exactly as before: the scene
paints under them and never stands in for them. Nothing in the scene carries an
accessible name, so a screen reader still reaches the dialog's role and title
and the credits' text, and hears no decoration.

## The layers, one module each

All of `crates/spaceport-gpui/src/about/effects/`. Each effect is a pure
function of the scene's time (seconds into a 12 s loop) plus its own constants:
no clock, no state, nothing allocated. That is what the tests pin — projection,
sine phases, palette selection at `t`, the rotozoom matrix at `t` — rather than
pixels.

| Layer | File | What it is |
| --- | --- | --- |
| Plasma | `effects/plasma.rs` | Four crossing sine waves read through the theme's cool ANSI ramp, one colour a 14 px cell. The field everything else sits on. |
| Starfield | `effects/starfield.rs` | 96 stars from a fixed hash, each a point projected from its own depth: the far plane reads its scattered place, and a star brightens and flies outward as its depth shrinks. |
| Copper bars | `effects/copper.rs` | Five warm columns riding across the stage in a sine, each lit across its width from a copper ramp, so each reads as a lit tube. |
| Rotozoomer | `effects/rotozoom.rs` | A checkered texture with a bright ring, read backwards through a matrix that turns twice and breathes three times a loop, in a square framing the mark. |
| Sine scroller | `effects/scroller.rs` | The greeting, one glyph an element, each lifted by a sine of where it is across the stage, so the line crosses as a wave. |
| Ramps | `effects/ramp.rs` | Three-stop colour mixing, so a layer's palette is a value its maths returns. |

`effects.rs` holds the loop length, the cell grid the painted layers share, and
the layer order. `scene.rs` holds the stage: the palette, the animation, the
marquee, and one canvas that paints the layers in order.

## Composition

Back to front, all inside one canvas: the stage's ground, the plasma field, the
stars, the copper columns, then the rotozoomed field around the mark. The
marquee rides a band at the top of the stage, over the field, and the credits
column paints over all of it.

The palette comes from the window's theme, not from fixed brand colours: the
ground is the theme's background, the layers the theme's ANSI brights mixed
toward that ground, and a star the theme's ink. So the box reads on a dark
theme and on a light one — a dark plasma and warm beams behind light text, a
pale wash behind dark text — and the credits keep the contrast they have
anywhere else. The identity comes from the mark and the words; there is no
second palette to drift from the theme.

## The clock

Both stage elements read one `Animation` in `crate::motion`'s terms:

- `Animation::new(motion::duration(LOOP))` with `repeat_synced()`, so the
  canvas and the marquee share the app's phase-locked clock and the scene
  repeats without a seam. Under QA > Slow Animations `duration` stretches the
  loop tenfold, so the scene slows like every other motion.
- `.with_max_fps(30)`, so it re-renders at most 30 times a second instead of
  burning every frame the window draws.
- Nothing reads a wall clock. Under reduced motion GPUI holds a repeating
  animation on its first frame and schedules nothing, so the scene becomes a
  still picture.
- Teardown is the element: the scene is a child of the About page, so closing
  the box drops it and the frame requests with it. There is no timer to stop
  and no state on the window.

A frame is a bounded amount of work: at the narrowest panel (326×568) the field
is 984 quads, the rotozoomed square 400, the stars 96, the copper 40 — about
1,500 quads, and the marquee is one text element a visible glyph. The paint
closure allocates nothing: palettes are mixed from `Copy` colours, the star
table is `LazyLock`ed once, and the frame's time reaches the closure through a
cell the animator writes, not through a rebuilt closure.

The About box is the one place the issue allows to burn a frame. The cost of
one frame is the window scene rebuild plus those quads, and it is paid only
while the box is open.

## Reaching the plain panel

For QA of the box without the scene:

```sh
SPACEPORT_ABOUT_CLASSIC=1 just run-debug
```

`scene::classic()` honours the variable in development and test builds
(`cfg!(debug_assertions)`) only, so every shipped build shows the scene and no
user finds a second About box behind a variable. `scene::panel` is the branch
itself, so a test draws both panels through it and finds the stage and marquee
only in the one that is shipped. The plain panel keeps the same credits
column, laid out by the same code, with no canvas and no marquee behind it.

## What this change does not do

- **No sound.** Per the owner's ruling. Sound stays in the notification path;
  nothing here plays anything.
- **No video and no new dependency.** Every effect is GPUI quads and text.
- **The mark does not spin.** A squircle rotated inside its 96 px box shows
  clipped corners, and the box has no room to grow it, so the rotozoomed field
  turns and breathes around a still mark instead.
- **No settings toggle.** The issue offered "an env var or a settings debug
  toggle"; a QA-only variable does not need a row in Settings.
- **No native screenshots in this change.** The layers were composed and
  colour-checked against the dark and light themes before they were painted in
  GPUI, but the real window was not captured here: `just test-gui` needs an
  active desktop and an installed Herdr. `just test-gui /path/to/herdr` with
  `HERDR_TEST_LOCAL_CONTROLS_CAPTURE=<dir>` writes `about-{theme}-{width}.png`
  from the native About smoke, which is the check to run on a desktop.

[#245]: https://github.com/2389-research/spaceport/issues/245
