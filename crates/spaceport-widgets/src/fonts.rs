//! Applying a configured face to elements.
//!
//! Shell prompts draw powerline separators and Nerd Font icons from the Private
//! Use Area. No text face covers those codepoints and no platform default
//! cascade reaches an installed icon font, so every such cell shapes to the
//! missing-glyph box until the cascade names one. [`FontConfig`] carries that
//! cascade alongside the family; [`StyledFont`] applies both.

use gpui::{Font, Styled};
use spaceport_settings::FontConfig;

/// The shaping font for `face`: its family, weight and fallback cascade.
pub fn font_for(face: &FontConfig) -> Font {
    let mut font = gpui::font(face.family.clone());
    font.weight = gpui::FontWeight(face.weight.0);
    font.fallbacks = face
        .fallbacks
        .as_ref()
        .filter(|families| !families.is_empty())
        .map(|families| gpui::FontFallbacks::from_fonts(families.clone()));
    font
}

/// The configured face on an element, as every piece of chrome that draws text
/// in a configured font wants it.
pub trait StyledFont: Styled + Sized {
    /// Sets the family and its fallback cascade, and nothing else. `Styled::font`
    /// would also reset weight and style, which nested chrome inherits.
    fn text_font(mut self, config: &FontConfig) -> Self {
        let Font {
            family, fallbacks, ..
        } = font_for(config);
        let style = self.text_style();
        style.font_family = Some(family);
        style.font_fallbacks = fallbacks;
        self
    }
}

impl<E: Styled> StyledFont for E {}
