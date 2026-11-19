// ABOUTME: Qualifies default status shapes through owned native Windows render targets.
// ABOUTME: Emits only small monochrome marker scenes for native pixel inspection.
use super::{Indicators, status_indicator};
use crate::config::{Config, FontConfig, StatusIcons, Theme};
use anyhow::{Result, ensure};
use base64::Engine as _;
use gpui::{prelude::*, *};
use sheprd_client::protocol::AgentStatus;
use std::io::Cursor;

struct Mark {
    background: u32,
    font: FontConfig,
    indicators: Indicators,
    status: AgentStatus,
}

impl Render for Mark {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div()
            .size_full()
            .bg(rgb(self.background))
            .p(px(8.))
            .child(status_indicator(self.status, &self.font, self.indicators))
    }
}

pub(crate) fn verify_status_shapes(light: bool, cx: &mut App) -> Result<()> {
    let config = Config::default();
    ensure!(
        config.status_icons == StatusIcons::Dots,
        "default status style changed"
    );
    for font_size in [config.sidebar.size, config.sidebar.size * 2.] {
        let mut ink = Vec::new();
        for status in [AgentStatus::Done, AgentStatus::Blocked] {
            let mut font = config.sidebar.clone();
            font.size = font_size;
            let mut indicators =
                Indicators::new(None, config.status_icons, false, light, &Theme::default());
            indicators.colors.fill(if light { 0 } else { 0xffffff });
            let handle = cx.open_window(
                WindowOptions {
                    focus: false,
                    titlebar: None,
                    window_bounds: Some(WindowBounds::Windowed(Bounds::new(
                        point(px(0.), px(0.)),
                        size(px(80.), px(48.)),
                    ))),
                    ..Default::default()
                },
                |_, cx| {
                    cx.new(|_| Mark {
                        background: if light { 0xffffff } else { 0 },
                        font,
                        indicators,
                        status,
                    })
                },
            )?;
            let pixels = AnyWindowHandle::from(handle).update(cx, |_, window, cx| {
                window.refresh();
                window.draw(cx).clear(cx);
                window.render_to_image()
            });
            let removed = handle.update(cx, |_, window, _| window.remove_window());
            let pixels = pixels??;
            removed?;
            let amount: u64 = pixels
                .pixels()
                .map(|pixel| {
                    if light {
                        255 - u64::from(pixel[0])
                    } else {
                        u64::from(pixel[0])
                    }
                })
                .sum();
            ensure!(
                amount > 0,
                "native {status:?} mark is invisible at size {font_size}"
            );
            ink.push(amount);
            let mut png = Cursor::new(Vec::new());
            pixels.write_to(&mut png, image::ImageFormat::Png)?;
            let theme = if light { "light" } else { "dark" };
            eprintln!(
                "STATUS_SHAPE_PNG {theme} {font_size} {status:?} {}",
                base64::engine::general_purpose::STANDARD.encode(png.into_inner())
            );
        }
        ensure!(
            ink[1] > ink[0],
            "native Blocked square must fill more area than Done circle at size {font_size}: {ink:?}"
        );
    }
    // i18n: data
    eprintln!("STATUS SHAPES native PASS: monochrome default and doubled markers");
    Ok(())
}

#[cfg(test)]
mod tests;
