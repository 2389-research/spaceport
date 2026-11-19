//! Runs of one cell background across a row, so a frame paints one quad per
//! run instead of one per cell.
//!
//! gpui-free: a future `sheprd-cells` crate takes this file as it stands.

use crate::config::Theme;
use crate::terminal::{cell_colors, on_default_background};
use sheprd_client::protocol::CellData;
use unicode_width::UnicodeWidthStr;

/// Runs of one background in `columns`: start, end, color, and whether it is
/// the theme's default rather than a color the program chose.
pub(super) fn background_spans<'a>(
    row: &'a [CellData],
    columns: std::ops::Range<usize>,
    theme: &'a Theme,
) -> impl Iterator<Item = (usize, usize, u32, bool)> + 'a {
    // A wide glyph's continuation cell shows the glyph's background, as a host
    // terminal does: Herdr's ANSI renderer never draws that cell, so its own
    // background is not meant to be seen. Like that renderer, a halfwidth
    // katakana with a (semi-)voiced mark counts as two columns.
    let wide = |symbol: &str| {
        let mut chars = symbol.chars();
        symbol.width() > 1
            || matches!(
                (chars.next(), chars.next(), chars.next()),
                (
                    Some('\u{ff66}'..='\u{ff9d}'),
                    Some('\u{ff9e}' | '\u{ff9f}'),
                    None
                )
            )
    };
    let bg = move |x: usize| {
        let x = if x > 0 && wide(&row[x - 1].symbol) {
            x - 1
        } else {
            x
        };
        (
            cell_colors(&row[x], theme).1,
            on_default_background(&row[x]),
        )
    };
    let mut start = columns.start;
    let stop = columns.end.min(row.len());
    std::iter::from_fn(move || {
        (start < stop).then_some(())?;
        let (color, default) = bg(start);
        let mut end = start + 1;
        while end < stop && bg(end) == (color, default) {
            end += 1;
        }
        let span = (start, end, color, default);
        start = end;
        Some(span)
    })
}
