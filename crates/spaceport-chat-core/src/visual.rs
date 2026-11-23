//! What a chat's pictures and diagrams are drawn from: a visual's identity,
//! its source, the bounds that admit it, and the colors and font a diagram is
//! drawn in. The app turns these into textures; nothing here paints.
//!
//! Both the images and the Mermaid fences come from the program and are
//! untrusted, so the source lengths and the visuals one chat admits are
//! bounded here rather than where the textures are made.

use spaceport_chat_protocol::Image;
use spaceport_settings::{FontConfig, Theme};
use spaceport_web_url::WebUrl;
use std::{
    hash::{DefaultHasher, Hash, Hasher},
    sync::Arc,
};

#[cfg(test)]
mod tests;

/// Bytes of Mermaid source drawn; a longer fence stays code.
pub const MAX_DIAGRAM_SOURCE: usize = 16 * 1024;
/// Visuals one chat prepares at once; the rest wait their turn.
pub const MAX_RUNNING: usize = 2;
/// Rich entries admitted per pane, including queued work and failures.
pub const MAX_RICH_VISUALS: usize = 128;
/// Retained formula textures per pane; images/diagrams keep their own limits.
pub const MAX_MATH_TEXTURE_BYTES: usize = 64 * 1024 * 1024;

/// How a formula is typeset: within a line, or set apart as its own display.
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub enum MathMode {
    Inline,
    Display,
}

/// A visual's identity: a hash of what it draws.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct VisualId(u64);

impl VisualId {
    pub fn of(content: impl Hash) -> Self {
        let mut hasher = DefaultHasher::new();
        content.hash(&mut hasher);
        Self(hasher.finish())
    }
}

/// The budget a visual counts against.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Code,
    Math,
    Other,
}

/// What a visual is drawn from.
#[derive(Clone, Debug)]
pub enum Source {
    Image(Image),
    Code {
        language: String,
        source: Arc<str>,
    },
    Math {
        source: Arc<str>,
        mode: MathMode,
    },
    Diagram(Arc<str>),
    /// A picture on the web, asked for by the user.
    Remote(WebUrl),
}

impl Source {
    /// Code and math count against the rich budget; math also against the
    /// texture budget.
    pub fn kind(&self) -> Kind {
        match self {
            Self::Code { .. } => Kind::Code,
            Self::Math { .. } => Kind::Math,
            Self::Image(_) | Self::Diagram(_) | Self::Remote(_) => Kind::Other,
        }
    }
}

/// The theme a diagram is drawn in, as the colors and font it uses.
#[derive(Clone, Debug, PartialEq)]
pub struct DiagramStyle {
    /// The page behind the diagram.
    pub background: u32,
    /// Cluster and actor outlines.
    pub border: u32,
    /// Whether the background is dark, which chooses the renderer's preset.
    pub dark: bool,
    /// The UI font family the diagram's labels are set in.
    pub family: String,
    /// Node, actor and cluster fills.
    pub fill: u32,
    /// Edges and sequence actor lines.
    pub line: u32,
    /// Notes and active steps.
    pub note: u32,
    /// Label size in points.
    pub size: f32,
    /// Label color.
    pub text: u32,
}

impl DiagramStyle {
    pub fn of(theme: &Theme, font: &FontConfig) -> Self {
        Self {
            background: theme.background,
            border: theme.muted,
            dark: luminance(theme.background) < 0.5,
            family: font.family.clone(),
            fill: theme.surface,
            line: theme.subtext(),
            note: theme.active,
            size: font.size,
            text: theme.foreground,
        }
    }
}

fn luminance(color: u32) -> f32 {
    let channel = |shift: u32| ((color >> shift) & 0xff) as f32 / 255.;
    0.2126 * channel(16) + 0.7152 * channel(8) + 0.0722 * channel(0)
}
