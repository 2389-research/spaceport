// ABOUTME: Checks native host routing, scoped repositories, and exact-window event delivery.
// ABOUTME: Shares routing checks between inactive and key-window qualification fixtures.
use crate::{
    NavigationTarget, SpaceportWindow, endpoint,
    sidebar::{
        self,
        layout_tests::PaintedProbes,
        native_tests::{HostArrows, Target},
    },
    smoke::*,
};
use anyhow::{Context as _, Result, bail};
use gpui::*;
use spaceport_client::ConnectTarget;
use std::time::{Duration, Instant};

#[path = "smoke_hosts/labels.rs"]
mod labels;

pub(super) const REMOTE: &str = "Synthetic host with a deliberately long label";

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Focus {
    DecoyKey,
    Unchanged,
}

/// Entry: native host routing under a decoy window the fixture never delivers to.
pub(crate) fn start(handle: WindowHandle<SpaceportWindow>, cx: &mut App) {
    cx.set_global(PaintedProbes::default());
    cx.spawn(async move |cx| {
        let result = async {
            with_root(handle, cx, |view, window, cx| -> Result<()> {
                view.update(cx, |view, cx| {
                    view.config.layout.mode =
                        crate::config::LayoutMode::from(crate::config::Density::Comfortable);
                    view.sidebar_scroll[0].set_offset(point(px(0.), px(-20.)));
                    cx.notify();
                });
                window.resize(fixture_size(480., 600.));
                draw(window, cx);
                Ok(())
            })?;
            checks(handle, Focus::Unchanged, cx).await?;
            cx.update(check_probes)?;
            Ok::<_, anyhow::Error>(())
        }
        .await;
        conclude(
            "SIDEBAR routing",
            "native host events, resized glyphs and independent scroll; key-window qualification excluded",
            result,
        );
    })
    .detach();
}

/// The hosts checks as the sidebar fixture runs them: an independently key
/// window holds the key, while every click here goes to the fixture's own window.
pub(super) async fn sidebar_hosts(
    handle: WindowHandle<SpaceportWindow>,
    cx: &mut AsyncApp,
) -> Result<()> {
    checks(handle, Focus::DecoyKey, cx).await
}

/// The fixture's own viewport height; the decoy-key run has room for its menus.
pub(super) fn height(focus: Focus) -> f32 {
    if focus == Focus::Unchanged { 600. } else { 780. }
}

async fn checks(
    handle: WindowHandle<SpaceportWindow>,
    focus: Focus,
    cx: &mut AsyncApp,
) -> Result<()> {
    let decoy = open_decoy(handle, focus, cx)?;
    await_activation(decoy, handle, focus, cx).await?;
    clicks(handle, decoy, focus, cx).await?;
    labels::resized(handle, focus, cx).await?;
    labels::scaled(handle, focus, cx).await?;
    status_group(handle, cx)?;
    independent_scroll(handle, cx).await?;
    close_decoy(decoy, cx)
}

/// Opens the decoy window and fills this window with the host, agent and
/// repository fixtures the clicks below target.
fn open_decoy(
    handle: WindowHandle<SpaceportWindow>,
    focus: Focus,
    cx: &mut AsyncApp,
) -> Result<WindowHandle<SpaceportWindow>> {
    let decoy = cx
        .update(|cx| {
            cx.open_window(
                WindowOptions {
                    focus: focus == Focus::DecoyKey,
                    ..WindowOptions::default()
                },
                |window, cx| {
                    cx.new(|cx| {
                        SpaceportWindow::new(
                            ConnectTarget::Socket("/unused-decoy.sock".into()),
                            window,
                            cx,
                            true,
                        )
                    })
                },
            )
        })
        .context("opening decoy window")?;
    handle
        .update(cx, |view, window, cx| {
            view.endpoints.clear();
            for (index, label) in ["Local", REMOTE, "Disabled"].into_iter().enumerate() {
                let mut endpoint = endpoint::Endpoint::new(
                    if index == 0 {
                        endpoint::LOCAL.into()
                    } else {
                        format!("fixture-{index}")
                    },
                    label.into(),
                    ConnectTarget::Socket(format!("/unused-sidebar-{index}.sock").into()),
                    index != 2,
                );
                let mut snapshot = sidebar::layout_tests::snapshot(6);
                snapshot.workspaces.drain(0..3);
                snapshot.workspaces[0].label = format!("repository-{index}");
                snapshot.workspaces[1].custom_label = true;
                snapshot.workspaces[1].label = format!("child-{index}");
                snapshot.workspaces.truncate(2);
                snapshot.agents.truncate(1);
                // This fixture tests the name fallback, not the shared fixture's
                // higher-priority "Claude Code" display name.
                snapshot.agents[0].display_agent = None;
                snapshot.agents[0].name =
                    Some(format!("agent-{index}-with-a-deliberately-long-label"));
                endpoint.live_mut().snapshot = Some(std::sync::Arc::new(snapshot));
                if let Ok(mut inbox) = endpoint.connection.inbox.lock() {
                    *inbox = endpoint.live().clone();
                }
                view.endpoints.push(endpoint);
            }
            view.selected_endpoint = 0;
            view.live = view.endpoints[0].live().clone();
            view.endpoints[0].collapsed_repos.clear();
            view.marked = "preserve collapse composition".into();
            window.resize(fixture_size(480., height(focus)));
            cx.notify();
        })
        .context("preparing sidebar host fixtures")?;
    if focus == Focus::DecoyKey {
        cx.update(|cx| cx.activate(true));
        decoy
            .update(cx, |_, window, _| window.activate_window())
            .context("activating decoy window")?;
    }
    Ok(decoy)
}

/// Waits for AppKit to make the decoy key and for the fixture's own resize to
/// settle, without guessing a delay.
async fn await_activation(
    decoy: WindowHandle<SpaceportWindow>,
    handle: WindowHandle<SpaceportWindow>,
    focus: Focus,
    cx: &mut AsyncApp,
) -> Result<()> {
    // GPUI schedules AppKit activation. Wait for its result, not a guessed delay
    // followed by a synchronous makeKeyWindow call on a possibly unordered window.
    let deadline = Instant::now() + Duration::from_secs(3);
    loop {
        let target = decoy
            .update(cx, |_, window, _| Target::acquire(window))
            .context("acquiring decoy target")??;
        let key = target.is_key();
        drop(target);
        let resized = handle
            .update(cx, |_, window, _| {
                window.viewport_size() == fixture_size(480., height(focus))
            })
            .context("checking sidebar resize")?;
        if (focus == Focus::Unchanged || key) && resized {
            break;
        }
        if Instant::now() >= deadline {
            let mtm = objc2::MainThreadMarker::new()
                .context("fixture activation requires main thread")?;
            let app = objc2_app_kit::NSApplication::sharedApplication(mtm);
            bail!(
                "decoy activation/resize deadline: key={key}, resized={resized}, app_active={}, app_hidden={}",
                app.isActive(),
                app.isHidden()
            );
        }
        cx.background_executor()
            .timer(Duration::from_millis(10))
            .await;
    }
    // The decoy is deliberately key. Neither acquisition nor delivery may use it.
    let target = decoy
        .update(cx, |_, window, _| Target::acquire(window))
        .context("acquiring decoy target")??;
    if focus == Focus::DecoyKey && !target.is_key() {
        bail!("decoy did not become key");
    }
    drop(target);
    let viewport = handle
        .update(cx, |_, window, _| window.viewport_size())
        .context("reading sidebar viewport")?;
    if viewport != fixture_size(480., height(focus)) {
        bail!("narrow resize not settled: {viewport:?}");
    }
    Ok(())
}

/// Clicks every host, agent and repository control the fixture shows, and holds
/// each one to the selection, composition and scope it is meant to change.
#[allow(clippy::too_many_lines, reason = "2026-11-27 baseline: 133 lines")]
async fn clicks(
    handle: WindowHandle<SpaceportWindow>,
    decoy: WindowHandle<SpaceportWindow>,
    focus: Focus,
    cx: &mut AsyncApp,
) -> Result<()> {
    for (step, label, expected) in [
        (0, REMOTE, 0), // collapse unselected host
        (1, REMOTE, 0), // expand
        (2, REMOTE, 1),
        (3, "Local", 0),
        (4, "Disabled", 0),
        (5, "child-1", 1),
        (6, "child-0", 0),
        (7, "agent-1-with-a-deliberately-long-label", 1),
        (8, "agent-0-with-a-deliberately-long-label", 0),
        (9, "repository-1", 0),
    ] {
        let decoy_target = decoy
            .update(cx, |_, window, _| Target::acquire(window))
            .context("checking decoy target")??;
        if focus == Focus::DecoyKey && !decoy_target.is_key() {
            bail!("decoy lost key status at step {step}");
        }
        drop(decoy_target);
        let (target, point) = with_root(handle, cx, |_, window, cx| -> Result<_> {
            clear_probes(cx);
            cx.default_global::<HostArrows>().0.clear();
            draw(window, cx);
            let probes = &cx.global::<PaintedProbes>().0;
            for (name, prefix) in [
                (REMOTE, "Synthetic host"),
                ("agent-1-with-a-deliberately-long-label", "agent-1-with-a"),
            ] {
                let p = probes
                    .get(name)
                    .with_context(|| format!("missing {name}"))?;
                if p.clipped
                    || p.glyph_text != p.cached
                    || !p.glyph_text.ends_with('\u{2026}')
                    || !p.glyph_text.starts_with(prefix)
                    || p.bounds.size.width < px(110.)
                    || p.width > p.bounds.size.width
                {
                    bail!("narrow native label: {name}: {p:?}");
                }
            }
            let p = probes
                .get(label)
                .with_context(|| format!("missing target {label}"))?;
            let mut point = p.bounds.center();
            if let Some(endpoint) = match step {
                0 | 2 => Some("fixture-1"),
                3 => Some(endpoint::LOCAL),
                4 => Some("fixture-2"),
                _ => None,
            } {
                // Text keys retain the clipped scrolling copy of a pinned
                // header. Its endpoint control records the visible copy last.
                point.y = cx
                    .global::<HostArrows>()
                    .0
                    .get(endpoint)
                    .context("missing painted host control")?
                    .center()
                    .y;
            }
            // The name folds the group. The trailing area selects the host.
            if (2..=4).contains(&step) {
                point.x = p.bounds.right() + px(spaceport_sidebar::metrics::HOST_GAP);
            }
            if step == 1 {
                point = cx
                    .global::<HostArrows>()
                    .0
                    .get("fixture-1")
                    .context("missing painted host collapse control")?
                    .center();
            }
            if step == 9 {
                // The title ends at the label column's edge, even with an avatar.
                point.x =
                    p.bounds.right() + px((spaceport_sidebar::metrics::LABEL_GAP + spaceport_sidebar::metrics::ARROW_RESERVE) / 2.);
            }
            Ok((Target::acquire(window)?, point))
        })
        .context("locating sidebar host click target")?;
        target.click(point.x.to_f64(), point.y.to_f64())?;
        drop(target);
        with_root(handle, cx, |view, window, cx| -> Result<()> {
            let delivered = window.mouse_position();
            clear_probes(cx);
            draw(window, cx);
            let view = view.read(cx);
            if view.selected_endpoint != expected {
                bail!(
                    "step {step}: selected {} expected {expected}; click={point:?} delivered={delivered:?}",
                    view.selected_endpoint
                );
            }
            if step < 2
                && (view.endpoints[1].collapsed != (step == 0)
                    || view.marked != "preserve collapse composition")
            {
                bail!("host collapse changed selection/composition");
            }
            if (5..=8).contains(&step) {
                let expected = if step < 7 {
                    NavigationTarget::Workspace("w4".to_owned())
                } else {
                    NavigationTarget::Pane("p0".to_owned())
                };
                if view.pending_navigation.as_ref() != Some(&expected) {
                    bail!("wrong duplicate-ID route at step {step}");
                }
            }
            if step == 9
                && (!view.endpoints[1]
                    .collapsed_repos
                    .contains("/fixture/agent-launcher/.git")
                    || !view.endpoints[0].collapsed_repos.is_empty())
            {
                bail!("repository collapse escaped endpoint scope");
            }
            let probes = &cx.global::<PaintedProbes>().0;
            if step == 0
                && (probes.contains_key("child-1")
                    || !probes.contains_key("agent-1-with-a-deliberately-long-label"))
            {
                bail!("host collapse hid agents or left workspace visible");
            }
            if step == 9 && (probes.contains_key("child-1") || !probes.contains_key("child-0"))
            {
                bail!("repository visibility not scoped");
            }
            Ok(())
        })
        .context("verifying sidebar host interaction")?;
        eprintln!("SIDEBAR native host step={step} verified");
    }
    Ok(())
}

/// Clicks a Done heading in the native window, then widens its icon-only agent row.
fn status_group(handle: WindowHandle<SpaceportWindow>, cx: &mut AsyncApp) -> Result<()> {
    with_root(handle, cx, |view, _, cx| {
        view.update(cx, |view, cx| -> Result<()> {
            view.agent_sort = crate::preferences::AgentSort::Priority;
            view.sidebar_width = Some(232.);
            view.sidebar_scroll[1].set_offset(Point::default());
            view.live.status = crate::state::ConnectionStatus::Connected;
            let snapshot = view.live.snapshot.as_mut().context("missing host snapshot")?;
            let snapshot = std::sync::Arc::make_mut(snapshot);
            let agent = snapshot.agents.first_mut().context("missing agent")?;
            agent.pane_id = "wAB:pY".into();
            agent.agent_status = spaceport_client::protocol::AgentStatus::Done;
            agent.display_agent = Some("Claude Code".into());
            agent.terminal_title_stripped = Some("Review changes".into());
            snapshot.panes.push(spaceport_client::protocol::ClientShellPane {
                pane_id: agent.pane_id.clone(),
                workspace_id: agent.workspace_id.clone(),
                tab_id: agent.tab_id.clone(),
                cwd: Some("/sandbox/review-dir".into()),
                foreground_cwd: None,
                focused: false,
                label: None,
                right_click_passthrough: false,
            });
            cx.notify();
            Ok(())
        })
    })?;
    for (visible, wide) in [(true, false), (false, false), (true, true)] {
        let (target, point) = with_root(handle, cx, |view, window, cx| -> Result<_> {
            if wide {
                view.update(cx, |view, cx| {
                    view.sidebar_width = Some(320.);
                    cx.notify();
                });
            }
            clear_probes(cx);
            draw(window, cx);
            let probes = &cx.global::<PaintedProbes>().0;
            if ["#1", "Review changes", "review-dir"].iter().any(|t| probes.contains_key(*t) != visible)
                || probes.contains_key("Claude Code") {
                bail!("unexpected native Done row: visible={visible}, wide={wide}");
            }
            if visible {
                let tab = probe(cx, "#1")?.bounds;
                let dir = probe(cx, "review-dir")?.bounds;
                let title = probe(cx, "Review changes")?.bounds;
                if tab.top() != dir.top() || tab.right() > dir.left() || title.bottom() > dir.top() {
                    bail!("terminal metadata should share a line below its title");
                }
            }
            Ok((Target::acquire(window)?, probe(cx, "Done")?.bounds.center()))
        })?;
        if !wide {
            target.click(point.x.to_f64(), point.y.to_f64())?;
        }
    }
    with_root(handle, cx, |view, window, cx| -> Result<()> {
        view.update(cx, |view, cx| -> Result<()> {
            let snapshot = view.live.snapshot.as_mut().context("missing host snapshot")?;
            let snapshot = std::sync::Arc::make_mut(snapshot);
            let agent = snapshot.agents.first_mut().context("missing agent")?;
            agent.agent_status = spaceport_client::protocol::AgentStatus::Idle;
            view.live.idle_tracker.observe(snapshot, spaceport_client::agent_idle::now_ms() - 7_500_000);
            cx.notify();
            Ok(())
        })?;
        clear_probes(cx);
        draw(window, cx);
        let probes = &cx.global::<PaintedProbes>().0;
        let age = crate::i18n::fl!("sidebar-idle-at-least", age = crate::i18n::fl!("sidebar-idle-age-hours", count = "2"));
        if !probes.contains_key(&age) || probes.keys().any(|text| text.starts_with("idle · ")) {
            bail!("native Idle row must show its observed age by itself");
        }
        Ok(())
    })?;
    Ok(())
}

/// The two sidebar lists scroll independently: moving one leaves the other where
/// it was.
async fn independent_scroll(
    handle: WindowHandle<SpaceportWindow>,
    cx: &mut AsyncApp,
) -> Result<()> {
    handle
        .update(cx, |view, _, cx| -> Result<()> {
            let snapshot = std::sync::Arc::make_mut(
                view.live
                    .snapshot
                    .as_mut()
                    .context("missing scroll snapshot")?,
            );
            let agent = snapshot.agents[0].clone();
            let workspace = snapshot.workspaces[0].clone();
            for index in 0..40 {
                let mut agent = agent.clone();
                agent.pane_id = format!("scroll-p{index}");
                snapshot.agents.push(agent);
                let mut workspace = workspace.clone();
                workspace.workspace_id = format!("scroll-w{index}");
                workspace.worktree = None;
                snapshot.workspaces.push(workspace);
            }
            cx.notify();
            Ok(())
        })
        .context("preparing sidebar scroll fixture")??;
    for list in 0..2 {
        with_root(handle, cx, |view, window, cx| -> Result<()> {
            draw(window, cx);
            let scroll = view.read(cx).sidebar_scroll.clone();
            let other = scroll[1 - list].offset();
            scroll[list].set_offset(point(px(0.), px(-80.)));
            draw(window, cx);
            if scroll[list].offset().y != px(-80.) || scroll[1 - list].offset() != other {
                bail!("scroll handles are not independent: list={list}");
            }
            Ok(())
        })
        .context("verifying independent sidebar scrolling")?;
    }
    Ok(())
}

/// The decoy received none of the fixture's clicks or navigation.
fn close_decoy(decoy: WindowHandle<SpaceportWindow>, cx: &mut AsyncApp) -> Result<()> {
    decoy
        .update(cx, |view, window, _| -> Result<()> {
            if view.selected_endpoint != 0
                || !view.endpoints[0].collapsed_repos.is_empty()
                || view.menu.page.is_some()
                || view.pending_navigation.is_some()
                || view.selection_epoch != 0
            {
                bail!("fixture click affected decoy");
            }
            window.remove_window();
            Ok(())
        })
        .context("verifying and closing decoy window")??;
    Ok(())
}
