//! The painter's own state across frames: its appearance, the shape cache it
//! keys on the font, and the cell metrics it measures.

use super::TerminalPainter;
use super::glyphs;
use crate::config::Theme;
use crate::terminal::CellMetrics;
#[cfg(feature = "integration-test")]
use anyhow::Result;
use gpui::{App, Font, Hsla, ShapedLine, SharedString, TextRun, Window, px, rgb};

impl TerminalPainter {
    pub fn set_appearance(&mut self, font_size: f32, cell_height: f32, theme: Theme) {
        if self.font_size != font_size || self.cell_height != cell_height || self.theme != theme {
            self.font_size = font_size;
            self.cell_height = cell_height;
            self.theme = theme;
            self.glyphs.clear();
            self.cell_width = None;
        }
    }

    #[cfg(feature = "integration-test")]
    pub fn reset_cache(&mut self) {
        self.config = None;
        self.glyphs.clear();
        self.cell_width = None;
    }

    #[cfg(feature = "integration-test")]
    pub fn verify_native_cache(&self, window: &Window) -> Result<usize> {
        let Some(base) = &self.config else {
            anyhow::bail!("missing font config");
        };
        for (style, symbol, cached) in self.glyphs.iter() {
            let fresh = self.shape(base, style, &symbol, window);
            // Includes native glyph IDs/positions, font IDs and metrics.
            if format!("{fresh:?}") != format!("{cached:?}") {
                anyhow::bail!("cached glyph/style mismatch: {symbol:?}");
            }
        }
        Ok(self.glyphs.len())
    }

    /// Glyphs for one cell symbol. Color is left to `paint_glyphs`, so one
    /// shape serves every color the symbol is drawn in.
    pub(super) fn shape(
        &self,
        font: &Font,
        style: usize,
        symbol: &str,
        window: &Window,
    ) -> ShapedLine {
        window.text_system().shape_line(
            SharedString::from(symbol.to_owned()),
            px(self.font_size),
            &[TextRun {
                len: symbol.len(),
                font: glyphs::styled_font(font, style),
                color: Hsla::default(),
                background_color: None,
                underline: None,
                strikethrough: None,
            }],
            None,
        )
    }

    /// The painter's own cell height with the width its caller measured.
    pub(super) fn metrics(&self, cell_width: f32) -> CellMetrics {
        CellMetrics::new(cell_width, self.cell_height)
    }

    pub(super) fn configure(&mut self, font: &Font) {
        if self.config.as_ref() != Some(font) {
            self.glyphs.clear();
            self.cell_width = None;
            self.config = Some(font.clone());
        }
    }

    pub fn cell_width(&mut self, font: &Font, window: &Window, cx: &mut App) -> f32 {
        self.configure(font);
        if let Some(width) = self.cell_width {
            return width;
        }
        let width = window
            .text_system()
            .shape_line(
                "M".into(),
                px(self.font_size),
                &[TextRun {
                    len: 1,
                    font: font.clone(),
                    color: rgb(self.theme.foreground).into(),
                    background_color: None,
                    underline: None,
                    strikethrough: None,
                }],
                None,
            )
            .width
            .to_f64() as f32;
        #[cfg(feature = "integration-test")]
        {
            cx.default_global::<crate::performance::Counts>()
                .metric_shapes += 1;
        }
        #[cfg(not(feature = "integration-test"))]
        let _ = cx;
        self.cell_width = Some(width);
        width
    }

    /// Releases the textures of images no paint has placed for a while, for
    /// frames that paint without placing any.
    pub(crate) fn release_idle_images(&mut self, window: &mut Window) {
        self.images.release_idle(window);
    }
}
