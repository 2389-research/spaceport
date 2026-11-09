//! The trailing badges a row reserves room for: local change counts, dirty,
//! note, muted, and upstream drift marks.

use super::label_text;
use crate::{
    config::{FontConfig, Theme},
    sidebar::{glyph_width, layout::SidebarMetrics, line_height},
};
use gpui::{prelude::*, *};

/// Local change counts and marks on the right edge of a workspace row.
pub(in crate::sidebar) struct RowBadge {
    pub(in crate::sidebar) counts: Option<Counts>,
    pub(in crate::sidebar) dirty: bool,
    /// The user keeps a note on this checkout. Card layouts leave it to the
    /// note's own line under the row, which leads with the same icon, rather
    /// than squeeze the workspace name in a narrow card.
    pub(in crate::sidebar) noted: bool,
    /// The user muted the workspace's notifications.
    pub(in crate::sidebar) muted: bool,
}

impl RowBadge {
    /// Whether it says anything about the checkout, beyond the workspace's
    /// mute, which card layouts put on the name's line instead.
    pub(in crate::sidebar) fn checkout_marks(&self) -> bool {
        self.counts.is_some() || self.dirty || self.noted
    }

    pub(in crate::sidebar) fn lines(&self, layout: &SidebarMetrics) -> usize {
        1 + usize::from(self.counts.is_some() && layout.count_line())
    }

    /// Nothing to draw is nothing to reserve, so a row with none keeps its
    /// full label width.
    pub(in crate::sidebar) fn new(
        counts: Option<Counts>,
        dirty: bool,
        noted: bool,
        muted: bool,
    ) -> Option<Self> {
        (counts.is_some() || dirty || noted || muted).then_some(Self {
            counts,
            dirty,
            noted,
            muted,
        })
    }

    /// The whole column's width: its first line or the counts line under
    /// it, whichever is wider.
    pub(in crate::sidebar) fn width(&self, font: &FontConfig, layout: &SidebarMetrics) -> f32 {
        self.first_width(font, layout)
            .max(self.counts_width(font, layout))
    }

    /// The counts line's width, or nothing when it is not drawn.
    pub(in crate::sidebar) fn counts_width(
        &self,
        font: &FontConfig,
        layout: &SidebarMetrics,
    ) -> f32 {
        let glyph = glyph_width(font);
        let counts = self.counts.as_ref().filter(|_| layout.count_line());
        counts.map_or(0., |counts| counts.chip_width(glyph, chip_icon(font)))
    }

    /// The first line's mark width. The label's own first line only yields
    /// this much, not the counts' width.
    pub(in crate::sidebar) fn first_width(
        &self,
        font: &FontConfig,
        layout: &SidebarMetrics,
    ) -> f32 {
        let glyph = glyph_width(font);
        let mark = line_height(font) + glyph;
        let marks = [self.dirty && !self.chipped(layout), self.noted, self.muted]
            .into_iter()
            .filter(|mark| *mark)
            .count();
        mark * marks as f32
    }

    /// Local counts take the dirty mark's place in a chip of its hue.
    fn chipped(&self, layout: &SidebarMetrics) -> bool {
        layout.count_line() && self.counts.is_some()
    }

    /// The badge column at a row's trailing edge, `width` wide: the marks on
    /// the first line, with local counts underneath.
    pub(in crate::sidebar) fn element(
        self,
        key: &str,
        width: f32,
        font: &FontConfig,
        theme: &Theme,
        layout: &SidebarMetrics,
    ) -> Div {
        let chipped = self.chipped(layout);
        let Self {
            counts,
            dirty,
            noted,
            muted,
        } = self;
        div()
            .debug_selector(|| format!("badge-{key}"))
            .w(px(width))
            .flex_none()
            .flex()
            .flex_col()
            .items_end()
            .overflow_hidden()
            .child(
                div()
                    .debug_selector(|| format!("badge-line-{key}"))
                    .h(px(line_height(font)))
                    .flex_none()
                    .flex()
                    .items_center()
                    .gap(px(glyph_width(font)))
                    .overflow_hidden()
                    .when(muted, |line| {
                        line.child(
                            crate::icons::muted(theme, (line_height(font) * 0.75).round())
                                .debug_selector(|| format!("muted-{key}")),
                        )
                    })
                    .when(noted, |line| {
                        line.child(
                            crate::icons::note(theme, (line_height(font) * 0.75).round())
                                .debug_selector(|| format!("note-{key}")),
                        )
                    })
                    // Uncommitted work uses the titlebar button's mark.
                    .when(dirty && !chipped, |line| {
                        line.child(
                            // Well under the line height, so marks on
                            // neighbouring rows keep a visible gap.
                            crate::icons::uncommitted(theme, (line_height(font) * 0.75).round())
                                .debug_selector(|| format!("dirty-{key}")),
                        )
                    }),
            )
            .when_some(counts.filter(|_| layout.count_line()), |column, badge| {
                let colors = (theme.ink(theme.palette[2]), theme.ink(theme.palette[1]));
                let chip = badge.chip(key, glyph_width(font), chip_icon(font), colors, theme);
                column.child(chip.h(px(line_height(font))))
            })
    }
}

/// `+additions` and `-deletions` for the local working tree.
pub(in crate::sidebar) struct Counts {
    pub(in crate::sidebar) additions: String,
    pub(in crate::sidebar) deletions: String,
}

/// Room inside a local counts chip: padding on each side, its one-pixel
/// border, and the gap after the pencil.
const CHIP_PADDING: f32 = 3.;

impl Counts {
    pub(in crate::sidebar) fn new(additions: u64, deletions: u64) -> Self {
        Self {
            additions: format!("+{}", compact(additions)),
            deletions: format!("-{}", compact(deletions)),
        }
    }

    /// A local chip's width at `glyph`, with an `icon`-wide pencil.
    pub(in crate::sidebar) fn chip_width(&self, glyph: f32, icon: f32) -> f32 {
        (glyph * self.glyphs() as f32).ceil() + icon + CHIP_PADDING * 3. + 2.
    }

    /// The pencil and the counts in the dirty mark's hue: it stands for the
    /// mark, so it carries the mark's selector and the counts the badge's.
    pub(in crate::sidebar) fn chip(
        &self,
        key: &str,
        glyph: f32,
        icon: f32,
        (additions, deletions): (u32, u32),
        theme: &Theme,
    ) -> Div {
        let hue = theme.palette[3];
        div()
            .debug_selector(|| format!("dirty-{key}"))
            .flex_none()
            .flex()
            .items_center()
            .gap(px(CHIP_PADDING))
            .px(px(CHIP_PADDING))
            .overflow_hidden()
            .rounded(px(crate::config::corners::SMALL))
            .bg(rgba((hue << 8) | 0x24))
            .border_1()
            .border_color(rgba((hue << 8) | 0x70))
            .child(
                svg()
                    .flex_none()
                    .size(px(icon))
                    .path("icons/pencil.svg")
                    .text_color(rgb(theme.ink(hue))),
            )
            .child(
                div()
                    .debug_selector(|| format!("counts-{key}"))
                    .flex()
                    .flex_none()
                    .w(px((glyph * self.glyphs() as f32).ceil()))
                    .child(
                        div()
                            .text_color(rgb(additions))
                            .child(label_text(&self.additions)),
                    )
                    .child(div().text_color(rgb(theme.muted)).child(label_text("/")))
                    .child(
                        div()
                            .text_color(rgb(deletions))
                            .child(label_text(&self.deletions)),
                    ),
            )
    }

    /// Both counts and the separator between them. Sidebar labels are
    /// monospace by default and digits are near-uniform elsewhere, so an
    /// em-fraction per glyph bounds the line; a wider face truncates the
    /// counts rather than eating the label.
    pub(in crate::sidebar) fn glyphs(&self) -> usize {
        self.additions.chars().count() + self.deletions.chars().count() + 1
    }
}

/// The pencil inside a local counts chip, a little smaller than the bare
/// mark's so the chip keeps a line's height.
pub(in crate::sidebar) fn chip_icon(font: &FontConfig) -> f32 {
    (line_height(font) * 0.6).round()
}

/// How far the checked-out branch has drifted from its upstream, as the
/// daemon's `git_status` token reports it: `↑` commits to push in green, `↓`
/// commits to pull in red, painted the way the TUI paints them.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::sidebar) struct Upstream {
    ahead: usize,
    behind: usize,
}

impl Upstream {
    /// Nothing while the branch is in sync, has no upstream, or the daemon's
    /// sidebar config leaves `git_status` out and so never computes it.
    pub(in crate::sidebar) fn new(counts: Option<(usize, usize)>) -> Option<Self> {
        let (ahead, behind) = counts?;
        (ahead > 0 || behind > 0).then_some(Self { ahead, behind })
    }

    /// `↑ahead` and `↓behind`, each only when nonzero.
    fn parts(self) -> impl Iterator<Item = (String, bool)> {
        [
            (self.ahead, '\u{2191}', true),
            (self.behind, '\u{2193}', false),
        ]
        .into_iter()
        .filter(|(count, _, _)| *count > 0)
        .map(|(count, arrow, ahead)| (format!("{arrow}{count}"), ahead))
    }

    /// Width at `glyph`, the two counts a glyph apart as in `↑2 ↓18`.
    pub(in crate::sidebar) fn width(self, glyph: f32) -> f32 {
        let (glyphs, parts) = self.parts().fold((0, 0), |(glyphs, parts), (text, _)| {
            (glyphs + text.chars().count(), parts + 1)
        });
        ((glyphs + parts - 1) as f32 * glyph).ceil()
    }

    pub(in crate::sidebar) fn element(self, key: &str, glyph: f32, theme: &Theme) -> Div {
        self.render(key, glyph, theme, true)
    }

    /// Inline counts take their natural shaped width, leaving the branch to
    /// shrink before a fallback arrow or digit is clipped.
    pub(in crate::sidebar) fn inline_element(self, key: &str, glyph: f32, theme: &Theme) -> Div {
        self.render(key, glyph, theme, false)
    }

    fn render(self, key: &str, glyph: f32, theme: &Theme, fixed_width: bool) -> Div {
        let (ahead, behind) = (theme.ink(theme.palette[2]), theme.ink(theme.palette[1]));
        div()
            .debug_selector(|| format!("upstream-{key}"))
            .when(fixed_width, |element| {
                element.w(px(self.width(glyph))).overflow_hidden()
            })
            .flex_none()
            .flex()
            .gap(px(glyph))
            .children(self.parts().map(|(text, is_ahead)| {
                div()
                    .flex_none()
                    .text_color(rgb(if is_ahead { ahead } else { behind }))
                    .child(label_text(&text))
            }))
    }
}

/// Four digits of churn is already a big diff; abbreviate past that so the
/// column stays narrow enough to leave the branch readable.
pub(crate) fn compact(lines: u64) -> String {
    match lines {
        0..=9999 => lines.to_string(),
        _ => format!("{}k", lines / 1000),
    }
}
