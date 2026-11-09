//! The note and mute marks at a workspace row's trailing edge.

use crate::{
    config::{FontConfig, Theme},
    sidebar::{glyph_width, line_height},
};
use gpui::{prelude::*, *};

/// User-owned marks that need room beside a workspace label.
pub(in crate::sidebar) struct RowBadge {
    pub(in crate::sidebar) noted: bool,
    pub(in crate::sidebar) muted: bool,
}

impl RowBadge {
    pub(in crate::sidebar) fn lines(&self) -> usize {
        1
    }

    pub(in crate::sidebar) fn new(noted: bool, muted: bool) -> Option<Self> {
        (noted || muted).then_some(Self { noted, muted })
    }

    pub(in crate::sidebar) fn width(&self, font: &FontConfig) -> f32 {
        let marks = usize::from(self.noted) + usize::from(self.muted);
        (line_height(font) + glyph_width(font)) * marks as f32
    }

    pub(in crate::sidebar) fn element(
        self,
        key: &str,
        width: f32,
        font: &FontConfig,
        theme: &Theme,
    ) -> Div {
        div()
            .debug_selector(|| format!("badge-{key}"))
            .w(px(width))
            .flex_none()
            .flex()
            .items_center()
            .justify_end()
            .gap(px(glyph_width(font)))
            .overflow_hidden()
            .when(self.muted, |line| {
                line.child(
                    crate::icons::muted(theme, (line_height(font) * 0.75).round())
                        .debug_selector(|| format!("muted-{key}")),
                )
            })
            .when(self.noted, |line| {
                line.child(
                    crate::icons::note(theme, (line_height(font) * 0.75).round())
                        .debug_selector(|| format!("note-{key}")),
                )
            })
    }
}
