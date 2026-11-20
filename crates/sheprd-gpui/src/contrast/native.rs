// ABOUTME: Qualifies shipped secondary text colors and glyphs through native Windows painting.
// ABOUTME: Captures only owned synthetic palette scenes and measures opaque native color samples.
use super::{Contrast, ratio};
use crate::{
    config::{Config, FontConfig, Theme},
    fonts::StyledFont,
};
use anyhow::{Result, ensure};
use base64::Engine as _;
use gpui::{prelude::*, *};
use std::{io::Cursor, sync::Arc};

const HEADER: f32 = 36.;
const ROW: f32 = 88.;
// i18n: data
const PALETTES: &[(&str, &str)] = &[
    ("github-light", "GitHub Light Default"),
    ("atom-one-light", "Atom One Light"),
    ("gruvbox-light", "Gruvbox Light"),
    ("one-half-light", "One Half Light"),
];

#[derive(Clone)]
struct Sample {
    row: usize,
    column: usize,
    role: &'static str,
    fill: &'static str,
    foreground: u32,
    background: u32,
}

struct Scene {
    font: FontConfig,
    name: &'static str,
    samples: Arc<Vec<Sample>>,
    theme: Theme,
}

impl Render for Scene {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div()
            .size_full()
            .flex()
            .flex_col()
            .bg(rgb(self.theme.background))
            .child(
                div()
                    .h(px(HEADER))
                    .flex_none()
                    .flex()
                    .items_center()
                    .px(px(16.))
                    .text_font(&self.font)
                    .text_size(px(12.))
                    .text_color(rgb(self.theme.foreground))
                    .child(self.name),
            )
            .children(self.samples.chunks(2).map(|row| {
                div()
                    .w_full()
                    .h(px(ROW))
                    .flex_none()
                    .flex()
                    .children(row.iter().map(|sample| {
                        let label = format!("{} on {} · AaMmIl 012345", sample.role, sample.fill); // i18n: data
                        div()
                            .h_full()
                            .flex_1()
                            .min_w_0()
                            .relative()
                            .bg(rgb(sample.background))
                            .children([9., 12.].into_iter().enumerate().map(|(line, size)| {
                                div()
                                    .absolute()
                                    .left(px(16.))
                                    .top(px(12. + line as f32 * 24.))
                                    .text_font(&self.font)
                                    .text_size(px(size))
                                    .line_height(px(size * 1.4))
                                    .text_color(rgb(sample.foreground))
                                    .whitespace_nowrap()
                                    .child(label.clone())
                            }))
                            .child(
                                div()
                                    .absolute()
                                    .left(px(16.))
                                    .top(px(66.))
                                    .w(px(16.))
                                    .h(px(12.))
                                    .bg(rgb(sample.foreground)),
                            )
                    }))
            }))
    }
}

struct Capture {
    image: image::RgbaImage,
    width: f32,
    height: f32,
    samples: Arc<Vec<Sample>>,
}

#[derive(serde::Serialize)]
struct Measurement {
    role: &'static str,
    fill: &'static str,
    expected_foreground: u32,
    foreground: u32,
    expected_background: u32,
    background: u32,
    ratio: f32,
    glyph_pixels: Vec<usize>,
}

fn capture(name: &'static str, theme: Theme, font: FontConfig, cx: &mut App) -> Result<Capture> {
    ensure!(
        theme.contrast == Contrast::Standard,
        "native standard-text qualification requires standard OS contrast"
    );
    let samples = Arc::new(
        [
            ("background", theme.background),
            ("surface", theme.surface),
            ("hover", theme.active), // i18n: data
        ]
        .into_iter()
        .enumerate()
        .flat_map(|(row, (fill, background))| {
            [("muted", theme.muted), ("subtext", theme.subtext())] // i18n: data
                .into_iter()
                .enumerate()
                .map(move |(column, (role, foreground))| Sample {
                    row,
                    column,
                    role,
                    fill,
                    foreground,
                    background,
                })
        })
        .collect::<Vec<_>>(),
    );
    let expected = samples.clone();
    let handle = cx.open_window(
        WindowOptions {
            focus: false,
            titlebar: None,
            window_bounds: Some(WindowBounds::Windowed(Bounds::new(
                point(px(0.), px(0.)),
                size(px(800.), px(340.)),
            ))),
            ..Default::default()
        },
        |_, cx| {
            cx.new(|_| Scene {
                font,
                name,
                samples,
                theme,
            })
        },
    )?;
    let pixels = AnyWindowHandle::from(handle).update(cx, |_, window, cx| -> Result<_> {
        window.refresh();
        window.draw(cx).clear(cx);
        let size = window.viewport_size();
        Ok((
            window.render_to_image()?,
            f32::from(size.width),
            f32::from(size.height),
        ))
    });
    let removed = handle.update(cx, |_, window, _| window.remove_window());
    let (image, width, height) = pixels??;
    removed?;
    ensure!(
        width >= 700. && height >= HEADER + ROW * 3.,
        "native palette viewport clipped its samples"
    );
    ensure!(
        u64::from(image.width()) * u64::from(image.height()) <= 2_000_000,
        "native palette image exceeds bound"
    );
    Ok(Capture {
        image,
        width,
        height,
        samples: expected,
    })
}

fn pixel(capture: &Capture, x: f32, y: f32) -> Result<u32> {
    let x = (x * capture.image.width() as f32 / capture.width) as u32;
    let y = (y * capture.image.height() as f32 / capture.height) as u32;
    ensure!(
        x < capture.image.width() && y < capture.image.height(),
        "native sample escaped its image"
    );
    let pixel = capture.image.get_pixel(x, y).0;
    ensure!(pixel[3] >= 250, "native palette sample is not opaque");
    Ok((u32::from(pixel[0]) << 16) | (u32::from(pixel[1]) << 8) | u32::from(pixel[2]))
}

fn distance(a: u32, b: u32) -> u32 {
    [0, 8, 16]
        .into_iter()
        .map(|shift| ((a >> shift) & 255).abs_diff((b >> shift) & 255))
        .sum()
}

fn measure(capture: &Capture) -> Result<Vec<Measurement>> {
    capture
        .samples
        .iter()
        .map(|sample| {
            let left = sample.column as f32 * capture.width / 2.;
            let top = HEADER + sample.row as f32 * ROW;
            let foreground = pixel(capture, left + 24., top + 72.)?;
            let background = pixel(capture, left + capture.width / 2. - 16., top + 78.)?;
            let glyph_pixels = [12., 36.]
                .into_iter()
                .map(|line| -> Result<_> {
                    let mut ink = 0;
                    for y in 0..18 {
                        for x in 16..320 {
                            let color = pixel(capture, left + x as f32, top + line + y as f32)?;
                            ink += usize::from(
                                distance(color, foreground) < distance(color, background),
                            );
                        }
                    }
                    Ok(ink)
                })
                .collect::<Result<Vec<_>>>()?;
            Ok(Measurement {
                role: sample.role,
                fill: sample.fill,
                expected_foreground: sample.foreground,
                foreground,
                expected_background: sample.background,
                background,
                ratio: ratio(foreground, background),
                glyph_pixels,
            })
        })
        .collect()
}

pub(crate) async fn verify_secondary_text(cx: &mut AsyncApp) -> Result<()> {
    let (font, palettes) = cx
        .background_spawn(async {
            let config = Config::default();
            let palettes = PALETTES
                .iter()
                .map(|&(id, name)| Ok((id, name, config.theme_named(name, true)?)))
                .collect::<Result<Vec<_>>>()?;
            Ok::<_, anyhow::Error>((config.sidebar.clone(), palettes))
        })
        .await?;
    for (id, name, theme) in palettes {
        let capture = cx.update(|cx| capture(name, theme, font.clone(), cx))?;
        let (png, measured) = cx
            .background_spawn(async move {
                let mut png = Cursor::new(Vec::new());
                capture.image.write_to(&mut png, image::ImageFormat::Png)?;
                let png = png.into_inner();
                ensure!(png.len() <= 131_072, "native palette PNG exceeds 128 KiB");
                Ok::<_, anyhow::Error>((png, measure(&capture)))
            })
            .await?;
        eprintln!(
            "SECONDARY_TEXT_PNG {id} {}",
            base64::engine::general_purpose::STANDARD.encode(png)
        );
        for measured in measured? {
            eprintln!("SECONDARY_TEXT {id} {}", serde_json::to_string(&measured)?);
            ensure!(
                distance(measured.foreground, measured.expected_foreground) <= 3
                    && distance(measured.background, measured.expected_background) <= 3,
                "native palette RGB differs from its configured colors"
            );
            ensure!(
                measured.ratio >= 4.5,
                "native secondary text contrast is below 4.5:1"
            );
            ensure!(
                measured.glyph_pixels.iter().all(|count| *count > 8),
                "native small text glyphs are missing"
            );
        }
    }
    eprintln!(
        "SECONDARY TEXT native PASS: shipped light palettes, muted/subtext, three backgrounds, 9/12px glyphs"
    ); // i18n: data
    Ok(())
}
