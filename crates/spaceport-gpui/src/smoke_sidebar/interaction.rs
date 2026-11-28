// ABOUTME: Checks native sidebar clicks, menu isolation and workspace dialogs.
// ABOUTME: Drives the same AppKit events a user's click produces.
use crate::{
    Command, ConnectionStatus, RunCommand, SpaceportWindow, menu,
    sidebar::native_tests::Target,
    smoke::*,
};
use anyhow::{Context as _, Result, bail};
use gpui::*;
use std::{sync::Arc, time::Duration};

/// Collapses and expands a host, opens the menu from the footer, and proves the
/// open menu does not pass its keys on to the terminal.
#[allow(clippy::too_many_lines, reason = "2026-11-27 baseline: 112 lines")]
pub(super) async fn menu_isolation(
    handle: WindowHandle<SpaceportWindow>,
    cx: &mut AsyncApp,
) -> Result<()> {
    for step in 0..4 {
        let (target, point) = with_root(handle, cx, |view, window, cx| -> Result<_> {
            view.update(cx, |view, _| {
                if step == 0 {
                    if let Some(snapshot) = view.live.snapshot.as_mut() {
                        let snapshot = Arc::make_mut(snapshot);
                        snapshot.focused_workspace_id = Some("w4".into());
                        for workspace in &mut snapshot.workspaces {
                            workspace.focused = workspace.workspace_id == "w4";
                        }
                    }
                    view.marked = "preserve active child".into();
                }
            });
            clear_probes(cx);
            draw(window, cx);
            let label = match step {
                0 => "\u{25be}",
                1 => "\u{25b8}",
                _ => "menu",
            };
            let point = probe(cx, label)
                .context("missing native click target")?
                .bounds
                .center();
            Ok((Target::acquire(window)?, point))
        })
        .context("updating native click target")?;
        let result = target.click(point.x.to_f64(), point.y.to_f64());
        cx.background_executor()
            .timer(Duration::from_millis(50))
            .await;
        let result = if step == 3 {
            result.and_then(|()| {
                let (target, point) = with_root(handle, cx, |view, window, cx| -> Result<_> {
                    // Paint the reopened menu before targeting its outside hitbox.
                    window.draw(cx).clear(cx);
                    let state = view.read(cx);
                    if state.menu.page != Some(menu::Page::Menu) {
                        bail!("native footer click did not reopen menu");
                    }
                    let menu::Cover::Panel(panel) = state.menu.cover.get() else {
                        bail!("reopened native menu has no painted panel bounds");
                    };
                    let viewport = Bounds::new(Point::default(), window.viewport_size());
                    let x = if viewport.right() - panel.right() >= panel.left() {
                        (panel.right() + viewport.right()) / 2.
                    } else {
                        panel.left() / 2.
                    };
                    let point = gpui::point(x, viewport.center().y);
                    if !viewport.contains(&point) || panel.contains(&point) {
                        bail!("no outside click target: panel={panel:?} viewport={viewport:?}");
                    }
                    eprintln!(
                        "SIDEBAR native outside click: panel={panel:?} viewport={viewport:?} target={point:?}"
                    );
                    Ok((Target::acquire(window)?, point))
                })
                .context("acquiring outside-click target")?;
                target.click(point.x.to_f64(), point.y.to_f64())
            })
        } else {
            result
        };
        with_root(handle, cx, |view, window, cx| -> Result<()> {
            result?;
            let state = view.read(cx);
            if step < 2 {
                if state.endpoints[0]
                    .collapsed_repos
                    .contains("/fixture/agent-launcher/.git")
                    != (step == 0)
                    || state.marked != "preserve active child"
                    || state.live.snapshot.as_ref().is_none_or(|s| {
                        s.workspaces.len() != 40
                            || s.focused_workspace_id.as_deref() != Some("w4")
                    })
                {
                    bail!("collapse navigated, removed rows, or lost selection");
                }
            } else if step == 2 {
                if state.menu.page != Some(menu::Page::Menu) {
                    bail!("native footer click did not open menu");
                }
                let before = state.input_probe;
                window.dispatch_action(Box::new(RunCommand { command: Command::Tab }), cx);
                for key in ["down", "down", "enter", "x", "escape"] {
                    window.dispatch_keystroke(
                        Keystroke {
                            key: key.into(),
                            ..Default::default()
                        },
                        cx,
                    );
                }
                let state = view.read(cx);
                if state.menu.page.is_some()
                    || state.input_probe.text != before.text
                    || state.input_probe.keys != before.keys
                    || state.input_probe.actions != before.actions
                {
                    bail!("menu keyboard handling leaked terminal input");
                }
            } else if state.menu.page.is_some() {
                bail!("outside click did not dismiss native menu");
            }
            Ok(())
        })?;
    }
    Ok(())
}

/// Opens the workspace menu's dialogs by native right click at two sizes and
/// proves each one keeps the terminal's input and the window's focus.
pub(super) async fn dialogs(handle: WindowHandle<SpaceportWindow>, cx: &mut AsyncApp) -> Result<()> {
    for (width, height) in [(640., 400.), (1200., 780.)] {
        let _ = handle.update(cx, |_, window, _| window.resize(size(px(width), px(height))));
        cx.background_executor()
            .timer(Duration::from_millis(100))
            .await;
        for action in [menu::WorkspaceAction::Rename] {
            handle
                .update(cx, |view, window, cx| {
                    view.live.status = ConnectionStatus::Connected;
                    // Keep the clicked row inside the scroll viewport below the native titlebar.
                    view.sidebar_scroll[0].set_offset(point(px(0.), px(-80.)));
                    clear_probes(cx);
                    cx.notify();
                    window.refresh();
                })
                .context("preparing native dialog")?;
            let point = with_root(handle, cx, |_, window, cx| -> Result<_> {
                window.draw(cx).clear(cx);
                probe(cx, "agent-launcher").map(|probe| {
                    eprintln!(
                        "DIALOG native {action:?} viewport={:?}",
                        window.viewport_size()
                    );
                    probe.bounds.center()
                })
            })?;
            let target = handle
                .update(cx, |_, window, _| Target::acquire(window))
                .context("acquiring native dialog target")??;
            let clicked = target.right_click(point.x.to_f64(), point.y.to_f64());
            cx.background_executor()
                .timer(Duration::from_millis(50))
                .await;
            let opened = with_root(handle, cx, |view, window, cx| -> Result<_> {
                clicked?;
                if view.read(cx).menu.page != Some(menu::Page::Workspace) {
                    bail!("right click did not open workspace menu");
                }
                let before = view.read(cx).input_probe;
                let steps = view
                    .read(cx)
                    .native_workspace_dialog_steps(action)
                    .context("workspace dialog is absent from its menu")?;
                for key in std::iter::repeat_n("down", steps).chain(std::iter::once("enter")) {
                    window.dispatch_keystroke(spaceport_keymap::parse_keystroke(key)?, cx);
                }
                window.draw(cx).clear(cx);
                if view.read(cx).menu.page != Some(menu::Page::Dialog(action)) {
                    bail!("workspace menu opened wrong dialog at {width}x{height} after {steps} down keys: expected {:?}, actual {:?}", menu::Page::Dialog(action), view.read(cx).menu.page);
                }
                window.dispatch_action(Box::new(RunCommand { command: Command::Tab }), cx);
                Ok(before)
            })
            .context("opening native dialog")?;
            // Finish entity creation before inspecting the dialog's focused input.
            with_root(handle, cx, |view, window, cx| -> Result<()> {
                // Only the editable dialogs carry a text field; confirmations do not.
                if action == menu::WorkspaceAction::Rename {
                    window.dispatch_keystroke(Keystroke::parse("cmd-a")?, cx);
                    for ch in "long-label-\u{65e5}\u{672c}-\u{1f600}".repeat(4).chars() {
                        window.dispatch_keystroke(spaceport_keymap::parse_keystroke(&ch.to_string())?, cx);
                    }
                    window.draw(cx).clear(cx);
                    view.update(cx, |view, cx| -> Result<()> {
                        let input = view.menu.input.as_ref().context("missing native editor")?;
                        if input.text != "long-label-\u{65e5}\u{672c}-\u{1f600}".repeat(4) {
                            bail!("native editor lost Unicode text");
                        }
                        let end = input.text.encode_utf16().count();
                        let field = input.bounds;
                        let caret = view
                            .bounds_for_range(end..end, Bounds::default(), window, cx)
                            .context("missing native IME bounds")?;
                        if !field.contains(&caret.origin)
                            || field.right() > window.viewport_size().width
                            || field.bottom() > window.viewport_size().height
                        {
                            bail!("native dialog caret/field out of bounds");
                        }
                        Ok(())
                    })?;
                }
                window.dispatch_keystroke(Keystroke::parse("escape")?, cx);
                let state = view.read(cx);
                if state.menu.page.is_some()
                    || state.input_probe.text != opened.text
                    || state.input_probe.keys != opened.keys
                    || state.input_probe.actions != opened.actions
                    || !state.focus.is_focused(window)
                {
                    bail!("workspace dialog leaked input or lost focus");
                }
                Ok(())
            })?;
        }
    }
    Ok(())
}
