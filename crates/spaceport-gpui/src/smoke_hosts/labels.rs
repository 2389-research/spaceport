// ABOUTME: Checks the native widths and sizes host and agent labels are drawn at.
// ABOUTME: Reads the painted rows the sidebar's own fixture filled in.
use super::*;

/// Exercise wider, narrower, then restored native allocations. This catches
/// stale truncated font runs as well as host labels left at the default width.
///
/// These baseline widths reserve the border, padding, arrow, gaps and status.
/// The host's device icon takes additional room measured from the painted row.
#[allow(clippy::too_many_lines, reason = "2026-11-27 baseline: 102 lines")]
pub(super) async fn resized(
    handle: WindowHandle<SpaceportWindow>,
    focus: Focus,
    cx: &mut AsyncApp,
) -> Result<()> {
    for (window_width, preferred, host_baseline, agent_width, host_prefix, agent_prefix) in [
        (
            800.,
            Some(400.),
            343.,
            343.,
            "Synthetic host",
            "agent-1-with-a",
        ),
        (360., None, 63., 63., "Syn", "agent-1"),
        (
            800.,
            Some(160.),
            103.,
            103.,
            "Synthetic",
            "agent-1-with\u{2026}",
        ),
        (
            800.,
            Some(480.),
            423.,
            423.,
            REMOTE,
            "agent-1-with-a-deliberately-long-label",
        ),
        (480., None, 175., 175., "Synthetic host", "agent-1-with-a"),
    ] {
        handle
            .update(cx, |view, window, cx| {
                view.sidebar_width = preferred;
                window.resize(fixture_size(window_width, height(focus)));
                cx.notify();
            })
            .context("resizing sidebar fixture")?;
        let deadline = Instant::now() + Duration::from_secs(2);
        loop {
            let settled = with_root(handle, cx, |_, window, cx| -> Result<bool> {
                if window.viewport_size() != fixture_size(window_width, height(focus)) {
                    return Ok(false);
                }
                clear_probes(cx);
                cx.default_global::<HostArrows>().0.clear();
                draw(window, cx);
                let host = cx
                    .global::<PaintedProbes>()
                    .0
                    .get(REMOTE)
                    .context("missing resized host label")?;
                let arrow = cx
                    .global::<HostArrows>()
                    .0
                    .get("fixture-1")
                    .context("missing resized host collapse control")?;
                let icon_room = host.bounds.left() - arrow.right() - px(spaceport_sidebar::metrics::HOST_GAP);
                let host_width = host_baseline - f32::from(icon_room);
                for (name, prefix, expected_width) in [
                    (REMOTE, host_prefix, host_width),
                    (
                        "agent-1-with-a-deliberately-long-label",
                        agent_prefix,
                        agent_width,
                    ),
                ] {
                    let p = cx
                        .global::<PaintedProbes>()
                        .0
                        .get(name)
                        .context("missing narrow label")?;
                    if p.clipped
                        || p.glyph_text != p.cached
                        || (p.glyph_text != name && !p.glyph_text.ends_with('\u{2026}'))
                        || !p.glyph_text.starts_with(prefix)
                        || p.bounds.size.width != px(expected_width)
                        || p.mask.size.width != px(expected_width)
                        || p.width > p.bounds.size.width
                    {
                        bail!(
                            "resized native label window={window_width} preferred={preferred:?}: {p:?}"
                        );
                    }
                }
                Ok(true)
            })
            .context("verifying resized sidebar labels")?;
            if settled {
                break;
            }
            if Instant::now() >= deadline {
                bail!("{window_width}px resize timed out");
            }
            cx.background_executor()
                .timer(Duration::from_millis(16))
                .await;
        }
        eprintln!(
            "SIDEBAR native resized labels verified: window={window_width} preferred={preferred:?} host_baseline={host_baseline} agent={agent_width}"
        );
    }
    Ok(())
}

/// Scaled text keeps its own line height and stays inside its box.
pub(super) async fn scaled(
    handle: WindowHandle<SpaceportWindow>,
    focus: Focus,
    cx: &mut AsyncApp,
) -> Result<()> {
    for font_size in [16., 20., 12.] {
        with_root(handle, cx, |view, window, cx| -> Result<()> {
            view.update(cx, |view, _| -> Result<()> {
                view.config.sidebar.size = font_size;
                if focus == Focus::Unchanged {
                    // Larger text needs its host scrolled into the compact viewport.
                    view.sidebar_scroll[0].set_offset(point(px(0.), px(-80.)));
                }
                view.config.theme = if font_size == 12. { "Default" } else { "Nord" }.into();
                view.theme = view.config.theme(false)?;
                Ok(())
            })?;
            clear_probes(cx);
            draw(window, cx);
            for name in [REMOTE, "agent-1-with-a-deliberately-long-label"] {
                let probe = cx
                    .global::<PaintedProbes>()
                    .0
                    .get(name)
                    .context("missing scaled label")?;
                if probe.clipped
                    || probe.glyph_text != probe.cached
                    || !probe.glyph_text.ends_with('\u{2026}')
                    || probe.width > probe.bounds.size.width
                    // Native layout snaps fractional line heights to device pixels.
                    || (probe.bounds.size.height - px(font_size * 4. / 3.)).abs()
                        > px(1. / window.scale_factor())
                {
                    bail!("scaled native label size={font_size}: {probe:?}");
                }
            }
            eprintln!("SIDEBAR native themed font labels verified: size={font_size}");
            Ok(())
        })
        .context("verifying scaled sidebar labels")?;
    }
    Ok(())
}
