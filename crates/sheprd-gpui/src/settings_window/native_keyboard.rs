// ABOUTME: Observes real Windows keyboard focus in the private Settings fixture.
// ABOUTME: Captures the same switch row before and after focus leaves without moving focus.
use super::{Section, SettingsWindow, search};
use anyhow::{Result, ensure};
use base64::Engine as _;
use gpui::*;
use std::{
    io::Cursor,
    time::{Duration, Instant},
};

const ROW: &str = "general-smooth-scroll";

mod scale;

struct Capture {
    image: image::RgbaImage,
    row: Bounds<Pixels>,
    viewport: Size<Pixels>,
    color: u32,
    smooth_scroll: bool,
}

pub(super) fn observe(handle: WindowHandle<SettingsWindow>, cx: &mut App) {
    let Some(mode) = std::env::var_os("HERDR_TEST_SETTINGS_KEYBOARD") else {
        return;
    };
    cx.spawn(async move |cx| {
        if mode == "scale" {
            if let Err(error) = scale::verify(handle, cx).await {
                eprintln!("SETTINGS_SCALE_ERROR {error:#}");
            }
        } else if let Err(error) = verify(handle, cx).await {
            eprintln!("SETTINGS_RING_ERROR {error:#}");
        }
    })
    .detach();
}

fn capture(
    handle: WindowHandle<SettingsWindow>,
    focused: bool,
    cx: &mut AsyncApp,
) -> Result<Option<Capture>> {
    AnyWindowHandle::from(handle).update(cx, |root, window, cx| -> Result<_> {
        let view = root
            .downcast::<SettingsWindow>()
            .map_err(|_| anyhow::anyhow!("unexpected Settings capture root"))?;
        let (ready, color, smooth_scroll) = {
            let view = view.read(cx);
            (
                view.section == Section::General
                    && !view.busy()
                    && view.search_results.is_none()
                    && view.dropdown.is_none(),
                view.theme.ink(view.theme.primary()),
                view.config.smooth_scroll,
            )
        };
        let row_focused = window
            .context_stack()
            .iter()
            .rev()
            .find_map(|context| context.get("settings-row"))
            .is_some_and(|row| row == ROW);
        if !ready
            || !window.is_window_active()
            || !window.last_input_was_keyboard()
            || window.focused(cx).is_none()
            || row_focused != focused
        {
            return Ok(None);
        }
        window.refresh();
        window.draw(cx).clear(cx);
        let Some(row) = cx
            .try_global::<search::RowBounds>()
            .and_then(|bounds| bounds.0.get(ROW))
            .copied()
        else {
            return Ok(None);
        };
        let viewport = window.viewport_size();
        ensure!(
            row.left() >= px(0.)
                && row.top() >= px(0.)
                && row.right() <= viewport.width
                && row.bottom() <= viewport.height,
            "native Settings switch row is clipped"
        );
        let image = window.render_to_image()?;
        ensure!(
            u64::from(image.width()) * u64::from(image.height()) <= 4_000_000,
            "native Settings image exceeds bound"
        );
        Ok(Some(Capture {
            image,
            row,
            viewport,
            color,
            smooth_scroll,
        }))
    })?
}

fn encode(capture: Capture) -> Result<(usize, Vec<u8>)> {
    let scale_x = capture.image.width() as f32 / f32::from(capture.viewport.width);
    let scale_y = capture.image.height() as f32 / f32::from(capture.viewport.height);
    let x = (f32::from(capture.row.left()) * scale_x).floor() as u32;
    let y = (f32::from(capture.row.top()) * scale_y).floor() as u32;
    let right = (f32::from(capture.row.right()) * scale_x).ceil() as u32;
    let bottom = (f32::from(capture.row.bottom()) * scale_y).ceil() as u32;
    ensure!(
        right > x
            && bottom > y
            && right <= capture.image.width()
            && bottom <= capture.image.height(),
        "native Settings crop escaped its image"
    );
    let crop = image::imageops::crop_imm(&capture.image, x, y, right - x, bottom - y).to_image();
    let rgb = [
        (capture.color >> 16) as u8,
        (capture.color >> 8) as u8,
        capture.color as u8,
    ];
    let start = crop.width().saturating_sub((92. * scale_x).ceil() as u32);
    let matching = crop
        .enumerate_pixels()
        .filter(|(x, _, pixel)| {
            *x >= start
                && pixel.0[3] > 240
                && (0..3).all(|channel| pixel.0[channel].abs_diff(rgb[channel]) <= 3)
        })
        .count();
    let mut png = Cursor::new(Vec::new());
    crop.write_to(&mut png, image::ImageFormat::Png)?;
    let png = png.into_inner();
    ensure!(png.len() <= 131_072, "native Settings crop exceeds 128 KiB");
    Ok((matching, png))
}

async fn wait_capture(
    handle: WindowHandle<SettingsWindow>,
    focused: bool,
    deadline: Instant,
    cx: &mut AsyncApp,
) -> Result<Capture> {
    let mut polls = 0;
    loop {
        if let Some(capture) = capture(handle, focused, cx)? {
            return Ok(capture);
        }
        if matches!(polls, 0 | 60 | 300) {
            eprintln!(
                "SETTINGS_CAPTURE_WAIT focused={focused} {}",
                capture_status(handle, cx)?
            );
        }
        polls += 1;
        ensure!(
            Instant::now() < deadline,
            "native Settings ring transition timed out"
        );
        cx.background_executor()
            .timer(Duration::from_millis(16))
            .await;
    }
}

fn capture_status(handle: WindowHandle<SettingsWindow>, cx: &mut AsyncApp) -> Result<String> {
    handle.update(cx, |view, window, cx| {
        let row_focused = window
            .context_stack()
            .iter()
            .rev()
            .find_map(|context| context.get("settings-row"))
            .is_some_and(|row| row == ROW);
        format!(
            "viewport={:?} active={} keyboard={} focus={} row={} general={} busy={} search={} dropdown={}",
            window.viewport_size(),
            window.is_window_active(),
            window.last_input_was_keyboard(),
            window.focused(cx).is_some(),
            row_focused,
            view.section == Section::General,
            view.busy(),
            view.search_results.is_some(),
            view.dropdown.is_some(),
        )
    })
}

async fn verify(handle: WindowHandle<SettingsWindow>, cx: &mut AsyncApp) -> Result<()> {
    let deadline = Instant::now() + Duration::from_secs(90);
    let focused = wait_capture(handle, true, deadline, cx).await?;
    let baseline = (
        focused.row,
        focused.viewport,
        focused.color,
        focused.smooth_scroll,
    );
    let (focused_ink, png) = cx.background_spawn(async move { encode(focused) }).await?;
    eprintln!(
        "SETTINGS_RING_PNG focused {}",
        base64::engine::general_purpose::STANDARD.encode(png)
    );
    eprintln!("SETTINGS_RING_READY focused");
    let unfocused = wait_capture(handle, false, deadline, cx).await?;
    ensure!(
        (
            unfocused.row,
            unfocused.viewport,
            unfocused.color,
            unfocused.smooth_scroll
        ) == baseline,
        "native Settings ring comparison changed geometry, color, or switch state"
    );
    let (unfocused_ink, png) = cx
        .background_spawn(async move { encode(unfocused) })
        .await?;
    eprintln!(
        "SETTINGS_RING_PNG unfocused {}",
        base64::engine::general_purpose::STANDARD.encode(png)
    );
    ensure!(
        focused_ink > unfocused_ink,
        "native Settings focus ring was not painted: {focused_ink} <= {unfocused_ink}"
    );
    eprintln!("SETTINGS_RING_PASS focused={focused_ink} unfocused={unfocused_ink}");
    Ok(())
}
