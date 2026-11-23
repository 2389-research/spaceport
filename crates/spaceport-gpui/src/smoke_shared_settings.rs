// ABOUTME: Checks the shared settings file a newer Herdr may have written.
// ABOUTME: Proves one value this build cannot read keeps its default, not the file.
use crate::smoke::*;
use crate::{SpaceportWindow, herdr_settings::TabBarPosition};
use anyhow::{Context as _, Result, bail};
use gpui::*;
use std::time::{Duration, Instant};

/// A `view.toml` as a build that knows a third tab-bar position would write it:
/// one value this build cannot read beside one it must still follow.
const SEEDED: &str = "[ui]\ntab_bar_position = \"floating\"\nhide_tab_bar_when_single_tab = true\n";

/// Entry: the file a build that knows more values wrote still loads, so the
/// value this build cannot read keeps its default and every other value applies.
pub(crate) fn start(handle: WindowHandle<SpaceportWindow>, cx: &mut App) {
    cx.spawn(async move |cx| {
        conclude(
            "SHARED SETTINGS native",
            "one unreadable value kept its default beside a value that still applied",
            verify(handle, cx).await,
        );
    })
    .detach();
}

async fn verify(handle: WindowHandle<SpaceportWindow>, cx: &mut AsyncApp) -> Result<()> {
    // A `--sidebar-test` window does not run the load its normal startup would,
    // so this fixture asks for it: once to read the file as it stands, then a
    // second time after seeding, rather than racing a load already reading it.
    handle.update(cx, |view, _, cx| view.load_shared_settings(cx))?;
    settle(handle, cx, "the first shared settings load").await?;
    let path = crate::config::view_settings_path(spaceport_client::local_var);
    let seeded = path.clone();
    cx.background_spawn(async move {
        if let Some(parent) = seeded.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&seeded, SEEDED)
    })
    .await
    .with_context(|| format!("seeding shared settings at {}", path.display()))?;
    handle.update(cx, |view, _, cx| view.load_shared_settings(cx))?;
    settle(handle, cx, "the shared settings reload").await?;
    with_root(handle, cx, |view, window, cx| -> Result<()> {
        let state = view.read(cx);
        let shared = state
            .settings
            .shared
            .as_ref()
            .context("no shared settings after the reload")?;
        if shared.tab_bar_position != TabBarPosition::default() {
            bail!(
                "a tab-bar position this build cannot read was not defaulted: {:?}",
                shared.tab_bar_position
            );
        }
        if !shared.hide_tab_bar_when_single_tab {
            bail!("the value beside the unreadable one did not apply");
        }
        // The window paints from what it loaded, so the file did not stop it.
        draw(window, cx);
        Ok(())
    })?;
    Ok(())
}

/// Waits for a shared settings load to land, bounded so a load that never
/// finishes fails the fixture instead of hanging the run.
async fn settle(
    handle: WindowHandle<SpaceportWindow>,
    cx: &mut AsyncApp,
    label: &str,
) -> Result<()> {
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        let settled = handle.update(cx, |view, _, _| -> Result<bool> {
            if let Some(error) = &view.settings.error {
                bail!("{label}: {error}");
            }
            Ok(view.settings.loaded && view.settings.task.is_none())
        })??;
        if settled {
            return Ok(());
        }
        if Instant::now() >= deadline {
            let state = handle.update(cx, |view, _, _| {
                format!(
                    "loaded={} running={}",
                    view.settings.loaded,
                    view.settings.task.is_some()
                )
            })?;
            bail!("{label} timed out: {state}");
        }
        cx.background_executor()
            .timer(Duration::from_millis(10))
            .await;
    }
}
