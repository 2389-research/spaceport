// ABOUTME: Checks native glyph shaping, truncation and row layouts in the sidebar.
// ABOUTME: Reads paint probes from frames the real text system shaped.
use crate::config::{Density, LayoutMode, Style};
use crate::sidebar::layout_tests::PaintedProbes;
use crate::smoke::*;
use crate::{SpaceportWindow, sidebar};
use anyhow::{Context as _, Result, bail};
use gpui::*;
use std::time::Duration;

/// Native check that the icon cascade reaches an installed Nerd Font. Prompts
/// draw powerline separators and icons from the Private Use Area, which no text
/// face and no platform default cascade covers, so without the cascade every
/// such cell shapes to the platform's missing-glyph box. Headless shaping
/// cannot show this: the test text system reports no installed families and
/// gives every glyph the same fixed advance.
pub(super) fn symbol_cascade(window: &mut Window, cx: &mut App) -> Result<&'static str> {
    use spaceport_settings::fonts::symbol_fallbacks;
    let detected = symbol_fallbacks(cx.text_system().all_font_names());
    if detected.is_empty() {
        // An installed icon font is an external resource, like a daemon binary.
        return Ok("symbol cascade skipped (no Nerd Font installed)");
    }
    let font = crate::config::font_for(&crate::config::FontConfig {
        family: "Menlo".into(),
        size: crate::terminal::FONT_SIZE,
        fallbacks: Some(detected.clone()),
        line_height_multiple: None,
        weight: crate::config::Weight::NORMAL,
    });
    // Fallback faces never enter `get_font_for_id`, and a cascade gives the
    // same family a new font id, so coverage is read from the glyphs: anything
    // no font carries shapes to the platform's missing-glyph box, and a covered
    // codepoint must not land on that same glyph.
    let glyphs = |symbol: &str, window: &mut Window| {
        window
            .text_system()
            .shape_line(
                symbol.to_owned().into(),
                px(crate::terminal::FONT_SIZE),
                &[TextRun {
                    len: symbol.len(),
                    font: font.clone(),
                    color: rgb(0xffffff).into(),
                    background_color: None,
                    underline: None,
                    strikethrough: None,
                }],
                None,
            )
            .runs
            .iter()
            .flat_map(|run| run.glyphs.iter().map(|glyph| glyph.id))
            .collect::<Vec<_>>()
    };
    // Plane 15 is private and unassigned by every shipped font, including the
    // Nerd Font patches, so it names the missing-glyph box for this machine.
    let missing = glyphs("\u{f0000}", window);
    if missing.is_empty() {
        bail!("no missing-glyph baseline to compare icons against");
    }
    // A separator, a branch and a clock: three prompt icons from three ranges.
    for symbol in ["\u{e0b0}", "\u{e0a0}", "\u{f017}"] {
        if glyphs(symbol, window) == missing {
            bail!("icon {symbol:?} stayed a missing-glyph box under cascade {detected:?}");
        }
    }
    if glyphs("A", window) == missing {
        bail!("cascade lost the configured face for plain text");
    }
    Ok("symbol cascade reaches installed Nerd Fonts")
}

/// Draws the Menlo fixture in flat and rounded normal, compact and comfortable
/// layouts at four viewport sizes, and answers how many frames fitted the
/// display.
#[allow(clippy::too_many_lines, reason = "2026-11-27 baseline: 112 lines")]
pub(super) async fn frames(handle: WindowHandle<SpaceportWindow>, cx: &mut AsyncApp) -> Result<u64> {
    let mut drawn = 0;
    for frame in 0..72 {
        cx.background_executor()
            .timer(Duration::from_millis(100))
            .await;
        let painted = with_root(handle, cx, |view, window, cx| -> Result<bool> {
            let (w, h) =
                [(1200., 780.), (640., 400.), (1000., 650.), (800., 600.)][(frame % 12) / 3];
            // Retain the glyph fixture's list viewport while reserving the fixed
            // device footer below both scrollable sections.
            let density = [Density::Comfortable, Density::Normal, Density::Compact][frame / 12 % 3];
            let style = [Style::Flat, Style::Rounded][frame / 36];
            // Rounded rows are taller; grow both sections so the probed rows
            // stay inside their lists at the smallest size too. The activity
            // bar's Workspaces title sits over the sidebar, so its height is
            // added back too.
            let title = view.read(cx).activity_title_height();
            let h = h + sidebar::DEVICE_FOOTER_HEIGHT + title + if style == Style::Rounded { 160. } else { 0. };
            let mode = LayoutMode::new(density, style);
            let compact = density == Density::Compact;
            if frame % 3 == 0 {
                window.resize(fixture_size(w, h));
            } else if window.viewport_size() != fixture_size(w, h) {
                // The platform keeps a window within its screen, so a size
                // larger than the display (a 1024x768 CI runner) is skipped and
                // counted, never passed as drawn.
                let size = fixture_size(w, h);
                let fits = window.display(cx).is_none_or(|display| {
                    let screen = display.visible_bounds().size;
                    size.width <= screen.width && size.height <= screen.height
                });
                if !fits {
                    eprintln!("SIDEBAR skipped frame={frame}: {size:?} does not fit the display");
                    return Ok(false);
                }
                bail!("native resize did not settle: {:?}", window.viewport_size());
            }
            clear_probes(cx);
            view.update(cx, |view, cx| {
                view.config.layout.mode = mode;
                cx.notify();
            });
            draw(window, cx);
            check_probes(cx)?;
            let probes = &cx.default_global::<PaintedProbes>().0;
            let mut failed = false;
            for input in [
                "herdr",
                "main",
                "1256789",
                "herdr-gpui-sidebar-rendering-regression-investigation",
                "fix/sidebar-label-width-and-overflow-regression",
            ] {
                if compact && matches!(input, "main" | "1256789" | "fix/sidebar-label-width-and-overflow-regression") {
                    if probes.contains_key(input) {
                        bail!("compact layout painted branch: {input}");
                    }
                    continue;
                }
                let p = probes
                    .get(input)
                    .with_context(|| format!("missing paint: {input}"))?;
                if frame == 0 {
                    eprintln!("SIDEBAR frame={frame} input={input:?} {p:?}");
                }
                let expected_short = input.len() < 20;
                let title_icon = matches!(input, "herdr" | "herdr-gpui-sidebar-rendering-regression-investigation");
                // Rounded rows give up the highlight's inset, the density's
                // gap, on both edges.
                let extra_width = match density {
                    Density::Comfortable => 0.,
                    Density::Normal => 10.,
                    Density::Compact => 16.,
                } - match (style, density) {
                    (Style::Flat, _) => 0.,
                    (Style::Rounded, Density::Comfortable) => 16.,
                    (Style::Rounded, Density::Normal) => 12.,
                    (Style::Rounded, Density::Compact) => 8.,
                };
                let icon_reserve = if title_icon {
                    spaceport_sidebar::metrics::ICON_RESERVE
                } else {
                    0.
                };
                let expected_width = px(spaceport_sidebar::metrics::LABEL_WIDTH + extra_width - icon_reserve);
                if p.glyph_text != p.cached
                    || (expected_short && p.glyph_text != input)
                    || (!expected_short
                        && (p.width < px(150.) || !p.glyph_text.ends_with('\u{2026}')))
                    || p.clipped
                    || !sidebar::layout_tests::within_device_pixel(p.bounds.size.width, expected_width, window.scale_factor())
                    || p.mask.size.width != p.bounds.size.width
                    // Native glyph advances can exceed a device-snapped box
                    // by a fraction of a pixel without clipping.
                    || (p.width > p.bounds.size.width
                        && !sidebar::layout_tests::within_device_pixel(
                            p.width,
                            p.bounds.size.width,
                            window.scale_factor(),
                        ))
                    || p.bounds.size.height != px(16.)
                {
                    eprintln!("SIDEBAR bad paint frame={frame} input={input:?} {p:?}");
                    failed = true;
                }
            }
            if failed {
                bail!("incomplete/cropped native glyph output");
            }
            eprintln!(
                "SIDEBAR verified frame={frame} mode={mode:?} viewport={:?} clipped=0",
                window.viewport_size()
            );
            Ok(true)
        })?;
        if painted {
            drawn += 1;
        }
    }
    if drawn == 0 {
        bail!("no fixture size fits the display");
    }
    Ok(drawn)
}

/// The other row layouts place text by their own geometry, so they are held to
/// what every layout owes: shaped glyphs match the text they were given, and
/// none is cropped or wider than its box.
pub(super) async fn other_layouts(
    handle: WindowHandle<SpaceportWindow>,
    cx: &mut AsyncApp,
) -> Result<()> {
    // The other layouts are not the fixed-width child fixture.
    cx.update(|cx| verify_child_geometry(cx, false));
    for mode in [LayoutMode::Superset, LayoutMode::Orca, LayoutMode::Minimal] {
        cx.background_executor()
            .timer(Duration::from_millis(100))
            .await;
        with_root(handle, cx, |view, window, cx| -> Result<()> {
            clear_probes(cx);
            view.update(cx, |view, cx| {
                view.config.layout.mode = mode;
                cx.notify();
            });
            draw(window, cx);
            check_probes(cx)?;
            let probes = &cx.default_global::<PaintedProbes>().0;
            // The long name must be cut short in every layout, which proves
            // ellipsizing natively.
            const LONG: &str = "herdr-gpui-sidebar-rendering-regression-investigation";
            if !probes
                .get(LONG)
                .is_some_and(|p| p.glyph_text.ends_with('\u{2026}'))
            {
                bail!("{mode}: {LONG:?} was not ellipsized");
            }
            for input in ["herdr", LONG] {
                let p = probes
                    .get(input)
                    .with_context(|| format!("missing paint: {input}"))?;
                // Orca shares one line between an agent and its place, so a
                // name may end in an ellipsis.
                let shown = p.glyph_text == input
                    || p.glyph_text
                        .strip_suffix('\u{2026}')
                        .is_some_and(|kept| input.starts_with(kept.trim_end()));
                if p.glyph_text != p.cached
                    || !shown
                    || p.clipped
                    || (p.width > p.bounds.size.width
                        && !sidebar::layout_tests::within_device_pixel(
                            p.width,
                            p.bounds.size.width,
                            window.scale_factor(),
                        ))
                {
                    bail!("{mode}: bad paint {input:?} {p:?}");
                }
            }
            eprintln!("SIDEBAR verified layout={mode}");
            Ok(())
        })?;
    }
    Ok(())
}
