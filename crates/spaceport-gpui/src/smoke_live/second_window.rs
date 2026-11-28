// ABOUTME: Checks that two windows are two clients of one daemon.
// ABOUTME: Keeps each window's focused space and connection its own.
use crate::smoke::*;
use crate::{LiveState, NavigationTarget, SpaceportWindow, open_window};
use anyhow::{Context as _, Result, bail};
use gpui::*;
use std::{
    sync::Arc,
    time::{Duration, Instant},
};

/// Two windows are two clients of one daemon. Each keeps its own focused space,
/// and neither one's navigation may move the other.
#[allow(clippy::too_many_lines, reason = "2026-11-27 baseline: 136 lines")]
pub(super) async fn verify(first: WindowHandle<SpaceportWindow>, cx: &mut AsyncApp) -> Result<()> {
    // Observe a window without leasing its root: drawing updates that entity.
    fn observe(
        handle: WindowHandle<SpaceportWindow>,
        cx: &mut AsyncApp,
        paint: bool,
    ) -> Result<(LiveState, Option<String>)> {
        with_root(handle, cx, |view, window, cx| {
            if paint {
                // A hidden window still needs a draw to publish its geometry.
                draw(window, cx);
            }
            let view = view.read(cx);
            Ok((view.live.clone(), view.notices.text().map(str::to_owned)))
        })
        .context("observing window")
    }

    fn healthy(live: &LiveState, local_error: Option<&str>) -> Result<Option<(String, String)>> {
        if let Some(error) = local_error.or(live.notices.text()) {
            bail!("window error: {error:.240}");
        }
        let (Some(snapshot), Some(surface)) = (&live.snapshot, &live.surface) else {
            return Ok(None);
        };
        if !live.status.is_connected()
            || snapshot.boot_id != surface.boot_id
            || snapshot.revision != surface.projection_revision
        {
            return Ok(None);
        }
        surface.frame.validate().context("invalid surface frame")?;
        let Some(workspace) = snapshot.focused_workspace_id.clone() else {
            return Ok(None);
        };
        Ok(Some((snapshot.boot_id.clone(), workspace)))
    }

    let (target, inbox) = first
        .update(cx, |view, _, _| {
            let endpoint = &view.endpoints[view.selected_endpoint];
            (
                endpoint.connection.target.clone(),
                endpoint.connection.inbox.clone(),
            )
        })
        .context("reading the first window's connection")?;
    let (live, local_error) = observe(first, cx, false)?;
    let (boot, first_workspace) =
        healthy(&live, local_error.as_deref())?.context("first window is not ready")?;
    let elsewhere = live
        .snapshot
        .as_ref()
        .context("missing first snapshot")?
        .workspaces
        .iter()
        .map(|workspace| workspace.workspace_id.clone())
        .find(|id| *id != first_workspace)
        .context("the daemon has only one workspace to show")?;
    let second = cx
        .update(|cx| open_window(target, cx, false))
        .context("opening a second window")?;
    if AnyWindowHandle::from(second) == AnyWindowHandle::from(first) {
        bail!("the second window replaced the first");
    }
    let deadline = Instant::now() + Duration::from_secs(30);
    let mut navigated = false;
    loop {
        cx.background_executor()
            .timer(Duration::from_millis(100))
            .await;
        let (second_live, second_error) = observe(second, cx, true)?;
        let (first_live, first_error) = observe(first, cx, false)?;
        let first_state = healthy(&first_live, first_error.as_deref())?;
        if first_state
            .as_ref()
            .map(|(_, workspace)| workspace.as_str())
            != Some(first_workspace.as_str())
        {
            bail!("the first window lost or changed its space: {first_state:?}");
        }
        if !Arc::ptr_eq(
            &inbox,
            &first
                .update(cx, |view, _, _| {
                    view.endpoints[view.selected_endpoint]
                        .connection
                        .inbox
                        .clone()
                })
                .context("re-reading the first window's inbox")?,
        ) {
            bail!("the first window's connection was replaced");
        }
        if let Some((second_boot, second_workspace)) =
            healthy(&second_live, second_error.as_deref())?
        {
            if second_boot != boot {
                bail!("the second window reached another daemon: {second_boot} != {boot}");
            }
            if !navigated {
                second
                    .update(cx, |view, _, cx| {
                        if view.input_ready() {
                            view.navigate(NavigationTarget::Workspace(&elsewhere), cx);
                            true
                        } else {
                            false
                        }
                    })
                    .context("navigating the second window")?
                    .then(|| navigated = true);
            } else if second_workspace == elsewhere {
                let panes = second_live
                    .surface
                    .as_ref()
                    .map_or(0, |surface| surface.panes.len());
                eprintln!(
                    "GUI second window verified: boot={boot} windows=2 first_space={first_workspace} second_space={second_workspace} second_panes={panes} separate_inbox=true"
                );
                return Ok(());
            }
        }
        if Instant::now() >= deadline {
            bail!(
                "second window deadline: navigated={navigated} first={first_state:?} second_status={:.160} second_snapshot={:?} second_surface={:?}",
                second_live.status,
                second_live
                    .snapshot
                    .as_ref()
                    .map(|s| (s.revision, s.focused_workspace_id.clone())),
                second_live.surface.as_ref().map(|s| (
                    s.projection_revision,
                    s.frame.width,
                    s.frame.height
                ))
            );
        }
    }
}
