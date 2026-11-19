// ABOUTME: Validates native image evidence against actual decoded screenshot pixels.
// ABOUTME: Requires bounded samples rather than accepting absent imagery or skipped viewport checks.
use anyhow::{Result, ensure};
use gpui::{Bounds, Pixels};
use image::RgbaImage;

#[cfg(test)]
mod tests;

pub(super) fn sample(pixels: &RgbaImage, scale: f32, x: f32, y: f32) -> Result<[u8; 4]> {
    let x = (x * scale).floor();
    let y = (y * scale).floor();
    ensure!(
        x >= 0. && y >= 0. && x < pixels.width() as f32 && y < pixels.height() as f32,
        "native image sample escapes its owned frame"
    );
    Ok(pixels.get_pixel(x as u32, y as u32).0)
}

pub(super) fn clipping(
    pixels: &RgbaImage,
    scale: f32,
    pane: Bounds<Pixels>,
    bounds: Bounds<Pixels>,
    mask: Bounds<Pixels>,
    crossing: bool,
) -> Result<()> {
    ensure!(
        scale.is_finite() && scale > 0.,
        "native image evidence has invalid scale"
    );
    let tolerance = 1. / scale;
    if crossing {
        ensure!(
            bounds.bottom() > pane.bottom(),
            "fractional image mapping does not cross the fixed pane bottom"
        );
        ensure!(
            f32::from(mask.bottom() - pane.bottom()) <= tolerance,
            "fractional image mask extends past the fixed pane bottom"
        );
    } else {
        ensure!(
            f32::from(bounds.bottom() - pane.bottom()).abs() <= tolerance,
            "authored edge image does not reach the fixed pane bottom"
        );
    }
    let visible = bounds.intersect(&mask).intersect(&pane);
    ensure!(
        f32::from(visible.size.width) * scale >= 1. && f32::from(visible.size.height) * scale >= 1.,
        "authored edge image has no visible interior pixels"
    );
    let x = f32::from(visible.left()) + f32::from(visible.size.width) / 2.;
    let inside = sample(
        pixels,
        scale,
        x,
        f32::from(visible.top()) + f32::from(visible.size.height) / 2.,
    )?;
    ensure!(
        inside[1] > 200 && inside[2] > 200 && inside[0] < 40,
        "authored cyan edge image is absent inside the pane: {inside:?}"
    );
    let y = f32::from(pane.bottom()) + 2.;
    ensure!(
        y * scale < pixels.height() as f32,
        "native clipping evidence has no outside sample"
    );
    let outside = sample(pixels, scale, x, y)?;
    ensure!(
        !(outside[1] > 200 && outside[2] > 200 && outside[0] < 40),
        "native image escaped the fixed terminal viewport: {outside:?}"
    );
    Ok(())
}
