//! The terminal's configured font weight, the heavier weight its bold cells
//! use, and whether the configured family has a face at that weight.
use crate::{Error, Result};

/// A font weight, as CSS names it: 100 (thin) to 900 (black). The app converts
/// it to its own renderer's type.
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub struct Weight(pub f32);

impl Weight {
    pub const THIN: Self = Self(100.);
    pub const EXTRA_LIGHT: Self = Self(200.);
    pub const LIGHT: Self = Self(300.);
    pub const NORMAL: Self = Self(400.);
    pub const MEDIUM: Self = Self(500.);
    pub const SEMIBOLD: Self = Self(600.);
    pub const BOLD: Self = Self(700.);
    pub const EXTRA_BOLD: Self = Self(800.);
    pub const BLACK: Self = Self(900.);
}

/// CSS weight names, lightest first. The first name for each weight is the
/// one diagnostics print.
const NAMED_WEIGHTS: [(&str, Weight); 14] = [
    ("thin", Weight::THIN),
    ("extralight", Weight::EXTRA_LIGHT),
    ("ultralight", Weight::EXTRA_LIGHT),
    ("light", Weight::LIGHT),
    ("normal", Weight::NORMAL),
    ("regular", Weight::NORMAL),
    ("medium", Weight::MEDIUM),
    ("semibold", Weight::SEMIBOLD),
    ("demibold", Weight::SEMIBOLD),
    ("bold", Weight::BOLD),
    ("extrabold", Weight::EXTRA_BOLD),
    ("ultrabold", Weight::EXTRA_BOLD),
    ("black", Weight::BLACK),
    ("heavy", Weight::BLACK),
];

/// How far bold cells sit above the configured weight: regular text's 400
/// becomes the conventional 700.
const BOLD_STEP: f32 = 300.0;

/// Reads a `weight` value: a CSS name such as `"medium"`, ignoring case,
/// spaces, hyphens and underscores, or a number from 100 to 900. Untyped
/// because only the terminal takes the key, so elsewhere its type must not
/// fail the whole file before it is reported as unknown.
pub fn parse(value: toml::Value, name: &'static str) -> Result<Weight> {
    let weight = match value {
        toml::Value::String(text) => {
            let normalized: String = text
                .chars()
                .filter(|c| !matches!(c, ' ' | '-' | '_'))
                .flat_map(char::to_lowercase)
                .collect();
            NAMED_WEIGHTS
                .iter()
                .find(|(weight_name, _)| *weight_name == normalized)
                .map(|(_, weight)| *weight)
        }
        toml::Value::Integer(value) => (100..=900).contains(&value).then_some(Weight(value as f32)),
        _ => None,
    };
    weight.ok_or(Error::InvalidFontWeight(name))
}

/// The weight bold cells draw at: a fixed step heavier than `weight`, never
/// lighter than bold, so bold stays visible over a medium or light base.
pub fn bold(weight: Weight) -> Weight {
    Weight((weight.0 + BOLD_STEP).clamp(Weight::BOLD.0, Weight::BLACK.0))
}

/// `weight` as configured: its CSS name with its number, or the number alone.
pub fn describe(weight: Weight) -> String {
    match NAMED_WEIGHTS.iter().find(|(_, named)| *named == weight) {
        Some((name, _)) => format!("{name} ({})", weight.0),
        None => format!("{}", weight.0),
    }
}

/// A configured weight the terminal's family has no face for, so the platform
/// draws the nearest weight it has instead.
#[derive(Clone, Debug, PartialEq)]
pub struct MissingWeight {
    pub family: String,
    pub weight: Weight,
}

/// Whether `family` has a face within half a step of `weight`. Core Text
/// reports weights through a piecewise mapping, so an exact match is rare.
/// Loads every face of the family, so it must stay off the UI thread.
#[cfg(target_os = "macos")]
pub fn platform_has_weight(family: &str, weight: Weight) -> bool {
    let Ok(handle) = font_kit::source::SystemSource::new().select_family_by_name(family) else {
        // A missing family is reported on its own.
        return true;
    };
    handle
        .fonts()
        .iter()
        .filter_map(|face| face.load().ok())
        .any(|face| (face.properties().weight.0 - weight.0).abs() < 50.0)
}

/// Only the macOS backend lists a family's faces by weight here, so other
/// platforms report nothing rather than guess.
#[cfg(not(target_os = "macos"))]
pub fn platform_has_weight(_family: &str, _weight: Weight) -> bool {
    true
}
