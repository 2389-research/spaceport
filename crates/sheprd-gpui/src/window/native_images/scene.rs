// ABOUTME: Recognizes only the authored real-daemon image scene and its paired assets.
// ABOUTME: Checks native raster samples against source colors and independent viewport bounds.
use super::{FramePaint, ImagePaint};
use crate::{presentation::Picture, terminal::CellMetrics};
use anyhow::{Result, ensure};
use gpui::{Bounds, EntityId, Pixels, Size};
use image::RgbaImage;
use sheprd_client::protocol::{
    PaneSurfaceFrame, SurfaceGraphicsFormat, SurfaceGraphicsSource, SurfaceGraphicsTarget,
};
use std::sync::Arc;

pub(super) struct Scene {
    pub(super) frame: Arc<PaneSurfaceFrame>,
    pub(super) metrics: CellMetrics,
    pub(super) regions: Vec<EntityId>,
    pub(super) serials: Vec<u64>,
}

pub(super) struct Shot {
    pub(super) paint: FramePaint,
    pub(super) pane: Bounds<Pixels>,
    pub(super) pixels: RgbaImage,
    pub(super) scale: f32,
    pub(super) scene: Scene,
    pub(super) viewport: Size<Pixels>,
}

pub(super) fn authored(picture: &Picture, token: uuid::Uuid) -> Option<Vec<u64>> {
    let frame = &picture.frame;
    let [pane] = frame.panes.as_slice() else {
        return None;
    };
    if pane.alternate_screen_active
        || pane.mouse_reporting
        || frame.popup.is_some()
        || frame.frame.width > 200
        || frame.frame.height > 100
        || frame
            .frame
            .cursor
            .as_ref()
            .is_some_and(|cursor| cursor.visible)
        || !frame
            .frame
            .cells
            .iter()
            .map(|cell| cell.symbol.as_str())
            .collect::<String>()
            .contains(&format!("SP_IMAGE_NATIVE_{token}"))
    {
        return None;
    }
    let mut serials = Vec::new();
    for (id, width, height, data, z) in [
        (7, 1, 2, &[255, 0, 0, 255, 0, 0, 255, 255][..], -1),
        (8, 1, 1, &[0, 255, 0, 128][..], 2),
        (9, 1, 1, &[255, 255, 0, 255][..], 3),
        (10, 1, 1, &[0, 255, 255, 255][..], 4),
    ] {
        let placed = frame.graphics.placements.iter().find(|placed| {
            matches!(&placed.asset.source, SurfaceGraphicsSource::Terminal {
                target: SurfaceGraphicsTarget::Pane { pane_id }, image_id
            } if pane_id == &pane.pane_id && *image_id == id)
        })?;
        let asset = picture.images.get(&placed.asset)?;
        if placed.z != z
            || placed.asset.format != SurfaceGraphicsFormat::Rgba
            || placed.asset.image_width != width
            || placed.asset.image_height != height
            || asset.data().as_ref() != data
        {
            return None;
        }
        serials.push(asset.serial());
    }
    Some(serials)
}

pub(super) fn image(paint: &FramePaint, z: i32) -> Result<&ImagePaint> {
    paint
        .images
        .iter()
        .find(|image| image.z == z && image.painted)
        .ok_or_else(|| anyhow::anyhow!("authored image z={z} has no successful paint submission"))
}

fn color(shot: &Shot, x: f32, y: f32) -> Result<[u8; 4]> {
    super::raster::sample(&shot.pixels, shot.scale, x, y)
}

pub(super) fn validate_pixels(shot: &Shot) -> Result<()> {
    ensure!(
        !shot.paint.overflow,
        "native image paint observations exceed bound"
    );
    for z in [-1, 2, 3, 4] {
        image(&shot.paint, z)?;
    }
    let below = image(&shot.paint, -1)?;
    ensure!(
        below.image.size == below.visible.size
            && below.visible.intersect(&below.mask).size == below.visible.size,
        "native source mapping or central-image clipping changed"
    );
    let middle = image(&shot.paint, 2)?;
    let x = f32::from(below.image.left() + below.image.size.width / 2.);
    // The green overlay touches both source-row centers; sample just outside its snapped edges.
    let red_y = f32::from(below.image.top() + below.image.size.height / 4.) - 1. / shot.scale;
    let blue_y = f32::from(below.image.top() + below.image.size.height * 0.75) + 1. / shot.scale;
    ensure!(
        red_y >= f32::from(below.image.top())
            && red_y < f32::from(middle.visible.top())
            && blue_y >= f32::from(middle.visible.bottom())
            && blue_y < f32::from(below.image.bottom()),
        "native source-row witnesses overlap the green image or leave their source mapping"
    );
    let top = color(shot, x, red_y)?;
    let bottom = color(shot, x, blue_y)?;
    ensure!(
        top[0] > 200 && top[2] < 40 && bottom[2] > 200 && bottom[0] < 40,
        "native image source rows are not red then blue: {top:?} / {bottom:?}"
    );
    let above = image(&shot.paint, 3)?;
    let yellow = color(
        shot,
        f32::from(above.image.left() + above.image.size.width / 2.),
        f32::from(above.image.top() + above.image.size.height / 2.),
    )?;
    ensure!(
        yellow[0] > 200 && yellow[1] > 200 && yellow[2] < 40,
        "native above-text image is missing: {yellow:?}"
    );
    let mixed = color(
        shot,
        f32::from(middle.image.left() + middle.image.size.width / 2.),
        f32::from(middle.image.top() + middle.image.size.height / 2.),
    )?;
    ensure!(
        mixed[0].abs_diff(64) <= 8 && mixed[1].abs_diff(128) <= 8 && mixed[2].abs_diff(64) <= 8,
        "native transparent overlap does not blend green once over the red/blue boundary: {mixed:?}"
    );
    let white = |image: &ImagePaint| -> Result<usize> {
        let x = f32::from(image.visible.left()) + shot.scene.metrics.width;
        let y = f32::from(image.visible.top()) + shot.scene.metrics.height;
        let mut count = 0;
        for row in 0..shot.scene.metrics.height.ceil() as u32 {
            for col in 0..(5. * shot.scene.metrics.width).ceil() as u32 {
                let pixel = color(shot, x + col as f32, y + row as f32)?;
                count += usize::from(pixel[0] > 170 && pixel[1] > 170 && pixel[2] > 170);
            }
        }
        Ok(count)
    };
    ensure!(
        white(below)? > 5,
        "native below-text image hides its authored glyphs"
    );
    ensure!(
        white(above)? == 0,
        "native above-text image leaves authored glyphs uncovered"
    );
    let edge = shot
        .paint
        .images
        .iter()
        .find(|image| {
            image.z == 4
                && image.painted
                && f32::from(image.visible.bottom() - shot.pane.bottom()) >= -1. / shot.scale
        })
        .ok_or_else(|| anyhow::anyhow!("native cyan edge does not reach its fixed viewport"))?;
    super::raster::clipping(
        &shot.pixels,
        shot.scale,
        shot.pane,
        edge.visible,
        edge.image,
        edge.mask,
        f32::from(edge.visible.bottom() - shot.pane.bottom()) > 1. / shot.scale,
    )
}

pub(super) fn validate_motion(base: &Shot, moving: &Shot, offset: f32) -> Result<()> {
    ensure!(
        base.viewport == moving.viewport && base.pane == moving.pane,
        "native image viewport moved during wheel input"
    );
    ensure!(
        base.scene.serials == moving.scene.serials,
        "native image assets changed during the gesture"
    );
    let before = image(&base.paint, -1)?;
    let after = image(&moving.paint, -1)?;
    ensure!(
        before.id == after.id && before.visible.size == after.visible.size,
        "native image identity or extent changed during scrolling"
    );
    let placement = |frame: &PaneSurfaceFrame| {
        frame
            .graphics
            .placements
            .iter()
            .find(|p| p.z == -1)
            .map(|p| p.y)
    };
    let a = placement(&base.scene.frame)
        .ok_or_else(|| anyhow::anyhow!("initial image placement missing"))?;
    let b = placement(&moving.scene.frame)
        .ok_or_else(|| anyhow::anyhow!("moving image placement missing"))?;
    let shift = (offset * moving.scene.metrics.height * moving.scale).round() / moving.scale;
    let expected = (f32::from(b) - f32::from(a)) * moving.scene.metrics.height + shift;
    let actual = f32::from(after.visible.top() - before.visible.top());
    ensure!(
        actual.abs() > 0. && (actual - expected).abs() <= 1. / moving.scale,
        "native image does not share its fractional cell translation: actual={actual}, expected={expected}"
    );
    let crossed = moving
        .paint
        .images
        .iter()
        .find(|image| {
            image.z == 4 && image.painted && image.visible.bottom() > moving.pane.bottom()
        })
        .ok_or_else(|| {
            anyhow::anyhow!("native fractional image has no oversized historical mapping")
        })?;
    super::raster::clipping(
        &moving.pixels,
        moving.scale,
        moving.pane,
        crossed.visible,
        crossed.image,
        crossed.mask,
        true,
    )?;
    validate_pixels(moving)
}
