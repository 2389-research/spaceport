// ABOUTME: Captures enlarged Settings text through the real native Windows renderer.
// ABOUTME: Resizes only its private window and keeps PNG encoding off the UI thread.
use super::super::native_png;
use super::{SettingsWindow, capture, capture_status, wait_capture};
use anyhow::{Result, ensure};
use base64::Engine as _;
use gpui::{AppContext as _, AsyncApp, WindowHandle, px, size};
use std::time::{Duration, Instant};

pub(super) async fn verify(handle: WindowHandle<SettingsWindow>, cx: &mut AsyncApp) -> Result<()> {
    let started = Instant::now();
    let deadline = started + Duration::from_secs(90);
    eprintln!("SETTINGS_SCALE_STAGE begin");
    // Native focus must reach the actual row before changing its window size.
    drop(wait_capture(handle, true, deadline, cx).await?);
    eprintln!(
        "SETTINGS_SCALE_STAGE initial_focus elapsed_ms={}",
        started.elapsed().as_millis()
    );
    handle.update(cx, |view, _, _| -> Result<()> {
        ensure!(
            view.config.ui.size == 24.,
            "native Settings did not load the private 24px UI config"
        );
        Ok(())
    })??;
    for (name, expected) in [
        ("wide", size(px(1100.), px(780.))),
        ("narrow", size(px(680.), px(780.))),
    ] {
        handle.update(cx, |_, window, _| window.resize(expected))?;
        eprintln!("SETTINGS_SCALE_STAGE resize {name} expected={expected:?}");
        let mut polls = 0;
        let mut keyboard_requested = false;
        let captured = loop {
            if let Some(captured) = capture(handle, true, cx)?
                && captured.viewport == expected
            {
                break captured;
            }
            if !keyboard_requested
                && handle.update(cx, |_, window, _| {
                    window.viewport_size() == expected && !window.last_input_was_keyboard()
                })?
            {
                eprintln!("SETTINGS_SCALE_KEYBOARD {name}");
                keyboard_requested = true;
            }
            if matches!(polls, 0 | 60 | 300) {
                eprintln!("SETTINGS_SCALE_WAIT {name} {}", capture_status(handle, cx)?);
            }
            polls += 1;
            ensure!(
                Instant::now() < deadline,
                "native Settings scale resize timed out"
            );
            cx.background_executor()
                .timer(Duration::from_millis(16))
                .await;
        };
        let dimensions = (captured.image.width(), captured.image.height());
        let row = captured.row;
        eprintln!(
            "SETTINGS_SCALE_STAGE encode {name} image={dimensions:?} elapsed_ms={}",
            started.elapsed().as_millis()
        );
        let png = cx
            .background_spawn(async move { native_png::encode(captured.image) })
            .await?;
        eprintln!(
            "SETTINGS_SCALE_STAGE encoded {name} bytes={} elapsed_ms={}",
            png.len(),
            started.elapsed().as_millis()
        );
        eprintln!(
            "SETTINGS_SCALE_PNG {name} {}",
            base64::engine::general_purpose::STANDARD.encode(png)
        );
        eprintln!("SETTINGS_SCALE_FRAME {name} ui=24 image={dimensions:?} row={row:?}");
    }
    eprintln!("SETTINGS_SCALE_PASS ui=24 wide=1100x780 narrow=680x780");
    Ok(())
}
