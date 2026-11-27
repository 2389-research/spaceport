// ABOUTME: Runs daemon-free native sidebar, menu, and glyph qualification.
// ABOUTME: Keeps activation-dependent checks distinct from directly delivered native events.
use super::*;
use crate::{SpaceportWindow, sidebar};
#[cfg(target_os = "macos")]
use crate::app_icon;
use anyhow::{Context as _, Result, bail};
use gpui::*;

#[path = "smoke_sidebar/glyphs.rs"]
mod glyphs;
#[cfg(target_os = "macos")]
#[path = "smoke_sidebar/interaction.rs"]
mod interaction;

/// Entry: the sidebar glyph and layout qualification a `--sidebar-test` run
/// without a fixture variable asks for.
pub(crate) fn start(handle: WindowHandle<SpaceportWindow>, cx: &mut App) {
    set_exit_code(1);
    // AppKit may exit(0) inside cx.quit(), bypassing main's ExitCode. These
    // daemon-free fixtures exit explicitly, like the performance driver.
    #[cfg(target_os = "macos")]
    require(
        "ICON native",
        app_icon::verify_native().and_then(|()| crate::app_badge::verify_native()),
    );
    cx.set_global(sidebar::layout_tests::PaintedProbes::default());
    verify_child_geometry(cx, true);
    if std::env::var_os("HERDR_TEST_SIDEBAR_PROBE_FAILURE").is_some() {
        let _ = handle.update(cx, |view, _, cx| {
            // Exercise the real native paint failure path: a legitimately narrow
            // sidebar, held to the default column the probe once assumed for
            // every frame, so the next frame's child row is one it must reject.
            cx.set_global(sidebar::layout_tests::ExpectDefaultChildColumn(true));
            view.sidebar_width = Some(180.);
            cx.notify();
        });
    }
    cx.spawn(async move |cx| {
        let summary = match qualify(handle, cx).await {
            Ok(summary) => summary,
            Err(error) => conclude("SIDEBAR native", "", Err(error)),
        };
        conclude("SIDEBAR native", &summary, Ok(()));
    })
    .detach();
}

/// Runs every sidebar check in order, each reporting under its own label.
async fn qualify(handle: WindowHandle<SpaceportWindow>, cx: &mut AsyncApp) -> Result<String> {
    // Shaping first: the cascade is independent of every layout probe below.
    let cascade = require(
        "SIDEBAR native symbol cascade",
        with_root(handle, cx, |_, window, cx| glyphs::symbol_cascade(window, cx)),
    );
    eprintln!("SIDEBAR native symbol cascade: {cascade}");
    require("SIDEBAR native bidi", bidi_shaping(handle, cx).await);
    #[cfg(target_os = "macos")]
    require(
        "SIDEBAR native preferences",
        preferences::sidebar_preferences(handle, cx).await,
    );
    let drawn = require(
        "SIDEBAR native glyphs",
        glyphs::frames(handle, cx).await,
    );
    require(
        "SIDEBAR native layouts",
        glyphs::other_layouts(handle, cx).await,
    );
    // Later fixtures add PR badges and dialogs that change label budgets.
    handle
        .update(cx, |view, _, cx| {
            verify_child_geometry(cx, false);
            view.config.layout.mode =
                crate::config::LayoutMode::from(crate::config::Density::Comfortable);
            cx.notify();
        })
        .context("resetting the sidebar layout")?;
    #[cfg(target_os = "macos")]
    require(
        "SIDEBAR native interaction",
        interaction::menu_isolation(handle, cx).await,
    );
    require("SIDEBAR native visibility", agents_visibility(handle, cx).await);
    #[cfg(target_os = "macos")]
    require(
        "SIDEBAR native dialog",
        interaction::dialogs(handle, cx).await,
    );
    require(
        "SIDEBAR native local controls",
        local_controls::verify(handle, cx).await,
    );
    #[cfg(target_os = "macos")]
    require(
        "SIDEBAR native hosts",
        hosts::sidebar_hosts(handle, cx).await,
    );
    require("SIDEBAR native paint", cx.update(check_probes));
    Ok(format!(
        "{cascade}; {drawn}/72 Menlo draws (the rest larger than the display), flat and rounded normal/compact/comfortable layouts at 4 sizes, collapse/expand, menu isolation, profile popup, right-click dialogs and Unicode fields at 2 sizes; host routing, disabled selection, scoped repositories, resized host/agent glyphs, independent scroll and decoy key window"
    ))
}

/// The platform's own shaper draws Hebrew right to left and gives the
/// isolation marks Fluent writes around every placeable no room.
async fn bidi_shaping(handle: WindowHandle<SpaceportWindow>, cx: &mut AsyncApp) -> Result<()> {
    match with_root(handle, cx, |_, window, _| bidi::check(window)) {
        // CoreText draws right-to-left runs in visual order; another
        // platform's answer is reported for docs/i18n.md.
        Ok(report) if cfg!(target_os = "macos") && !report.visual_order => {
            bail!("Hebrew drawn in logical order");
        }
        Ok(report) => {
            eprintln!(
                "SIDEBAR native bidi: isolation marks take no room; Hebrew in {} order; en-XB {:?}",
                if report.visual_order { "visual" } else { "logical" },
                report.pseudo,
            );
            Ok(())
        }
        Err(error) => Err(error),
    }
}

/// The native Agents setting decides whether agent rows are painted, and it
/// leaves the workspace rows alone either way.
async fn agents_visibility(handle: WindowHandle<SpaceportWindow>, cx: &mut AsyncApp) -> Result<()> {
    for show_agents in [false, true] {
        with_root(handle, cx, |view, window, cx| -> Result<()> {
            view.update(cx, |view, cx| {
                view.config.show_agents = show_agents;
                cx.notify();
            });
            clear_probes(cx);
            draw(window, cx);
            let probes = &cx.default_global::<sidebar::layout_tests::PaintedProbes>().0;
            if probes.contains_key("tab 1") != show_agents
                || !probes.contains_key("herdr")
                || !probes.contains_key("menu")
            {
                bail!("native Agents visibility did not follow configuration");
            }
            Ok(())
        })?;
    }
    Ok(())
}
