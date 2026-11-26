// ABOUTME: Draws each sidebar agent in its product colors with a theme-aware monochrome fallback.
// ABOUTME: Registers full-color marks with the same embedded asset source as other icons.
use crate::AgentIcon;
use gpui::{AnyElement, ObjectFit, div, img, prelude::*, px};
use spaceport_settings::Theme;

macro_rules! artwork {
    ($($agent:ident => $file:literal,)+) => {
        const ARTWORK: &[(AgentIcon, &str, &[u8])] = &[$((
            AgentIcon::$agent,
            concat!("icons/brand/", $file),
            include_bytes!(concat!("../../../assets/icons/brand/", $file)),
        ),)+];
    };
}

artwork! {
    // keep-sorted start
    Antigravity => "agy.svg",
    Codex => "codex.svg",
    Devin => "devin.svg",
    Gemini => "gemini.svg",
    Kimi => "kimi.svg",
    Kiro => "kiro.svg",
    Maki => "maki.png",
    Muse => "muse.svg",
    Omp => "omp.svg",
    Qodercli => "qodercli.svg",
    // keep-sorted end
}

pub(super) fn assets() -> impl Iterator<Item = (&'static str, &'static [u8])> {
    ARTWORK.iter().map(|&(_, path, bytes)| (path, bytes))
}

impl AgentIcon {
    /// The brand mark for an agent row. Neutral artwork follows the row's ink.
    pub fn sidebar_element(self, size: f32, neutral: u32, theme: &Theme) -> AnyElement {
        if let Some(&(_, path, _)) = ARTWORK.iter().find(|(agent, _, _)| *agent == self) {
            let image = img(path)
                .size(px(size))
                .flex_none()
                .object_fit(ObjectFit::Contain);
            // These marks have one colored detail over a shape drawn in currentColor.
            if matches!(self, Self::Kimi | Self::Qodercli) {
                return div()
                    .relative()
                    .size(px(size))
                    .flex_none()
                    .child(self.element(size, neutral))
                    .child(div().absolute().inset_0().child(image))
                    .into_any_element();
            }
            return image.into_any_element();
        }
        let color = match self {
            Self::Amp => 0xf34e3f,
            Self::Claude => 0xd97757,
            Self::Cline => 0x323b43,
            Self::Kilo => 0xf8f676,
            Self::Letta => 0x8c8cf9,
            Self::Qwen => 0x082dff,
            Self::Droid => return crate::icon("icons/providers/factory.svg", size, neutral),
            _ => return self.element(size, neutral),
        };
        self.element(size, theme.ink(color))
    }
}
