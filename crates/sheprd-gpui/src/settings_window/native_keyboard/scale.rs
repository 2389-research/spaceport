// ABOUTME: Captures enlarged Settings text through the real native Windows renderer.
// ABOUTME: Resizes only its private window and keeps GPU encoding off the UI thread.
use super::{SettingsWindow, capture, wait_capture};
use anyhow::{Result, ensure};
use base64::Engine as _;
use gpui::{AppContext as _, AsyncApp, WindowHandle, px, size};
use std::time::{Duration, Instant};

pub(super) async fn verify(handle: WindowHandle<SettingsWindow>, cx: &mut AsyncApp) -> Result<()> {
    let deadline = Instant::now() + Duration::from_secs(90);
    // Native focus must reach the actual row before changing its window size.
    drop(wait_capture(handle, true, deadline, cx).await?);
    handle.update(cx, |view, _, _| -> Result<()> {
        ensure!(
            view.config.ui.size == 24.,
            "native Settings did not load the private 24-point UI config"
        );
        Ok(())
    })??;
    for (name, expected) in [
        ("wide", size(px(1100.), px(780.))),
        ("narrow", size(px(680.), px(780.))),
    ] {
        handle.update(cx, |_, window, _| window.resize(expected))?;
        let captured = loop {
            if let Some(captured) = capture(handle, true, cx)?
                && captured.viewport == expected
            {
                break captured;
            }
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
        let png = cx
            .background_spawn(async move {
                use image::codecs::png::{CompressionType, FilterType, PngEncoder};
                let mut png = Vec::new();
                captured
                    .image
                    .write_with_encoder(PngEncoder::new_with_quality(
                        &mut png,
                        CompressionType::Fast,
                        FilterType::Sub,
                    ))?;
                ensure!(
                    png.len() <= 196_608,
                    "native Settings scale PNG exceeds 192 KiB"
                );
                anyhow::Ok(png)
            })
            .await?;
        eprintln!(
            "SETTINGS_SCALE_PNG {name} {}",
            base64::engine::general_purpose::STANDARD.encode(png)
        );
        eprintln!("SETTINGS_SCALE_FRAME {name} ui=24 image={dimensions:?} row={row:?}");
    }
    eprintln!("SETTINGS_SCALE_PASS ui=24 wide=1100x780 narrow=680x780");
    Ok(())
}
