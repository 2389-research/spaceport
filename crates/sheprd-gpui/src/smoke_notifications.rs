//! Offline notification preview driver for the daemon-free sidebar fixture.
use crate::smoke::*;
use crate::{Command, SheprdWindow};
use anyhow::{Context as _, Result, bail};
use gpui::*;
use sheprd_client::protocol::*;
use std::time::Duration;

/// Entry: every offline toast preview drawn at a narrow and a wide size.
pub(crate) fn start(handle: WindowHandle<SheprdWindow>, cx: &mut App) {
    set_exit_code(1);
    let timer = cx.background_executor().clone();
    cx.spawn(async move |cx| {
        for (width, height) in [(360., 240.), (1200., 780.)] {
            let _ = handle.update(cx, |_, window, _| window.resize(size(px(width), px(height))));
            timer.timer(Duration::from_millis(100)).await;
            for kind in [
                SemanticNotificationKind::NeedsAttention,
                SemanticNotificationKind::Finished,
                SemanticNotificationKind::UpdateInstalled,
                SemanticNotificationKind::Custom,
            ] {
                let result = handle.update(cx, |view, window, cx| -> Result<()> {
                    view.menu.reset();
                    view.config.notifications.enabled = false;
                    view.config.notifications.delay_seconds = 3600;
                    window.focus(&view.focus, cx);
                    let selected = view.selected_endpoint;
                    let snapshot = view.live.snapshot.clone();
                    view.show_toast_preview(kind, cx);
                    let notice = &view.endpoints[selected].toasts.entries.back().context("missing toast preview")?.1;
                    if !notice.is_visible() || notice.kind != kind || view.endpoints.iter().any(|e| e.connection.handle.is_some()) {
                        bail!("preview did not bypass policy offline");
                    }
                    view.command(Command::OpenNotificationTarget, window, cx);
                    if view.selected_endpoint != selected || view.live.snapshot != snapshot || view.pending_navigation.is_some() || !view.focus.is_focused(window) {
                        bail!("offline notification command navigated or stole focus");
                    }
                    Ok(())
                });
                let painted = with_root(handle, cx, |_, window, cx| {
                    draw(window, cx);
                    Ok(())
                });
                if !matches!(result, Ok(Ok(()))) || painted.is_err() {
                    eprintln!("NOTIFICATIONS native FAIL: {result:?} {painted:?}");
                    std::process::exit(1);
                }
            }
        }
        eprintln!("NOTIFICATIONS native PASS: all four offline previews drawn at narrow/wide sizes; disabled/delayed policy bypass and inert offline command verified");
        std::process::exit(0);
    })
    .detach();
}
