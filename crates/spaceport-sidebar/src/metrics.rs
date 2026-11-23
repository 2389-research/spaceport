//! Sidebar geometry. The same numbers feed layout, hit testing, and the
//! label budgets, so a row never paints wider than it was measured.

use spaceport_settings::FontConfig;
use std::time::Duration;

pub const SIDEBAR_WIDTH: f32 = 232.;
/// How long the pointer rests on a workspace row before its menu opens, so the
/// same actions a right click offers are reachable without one.
pub const HOVER_MENU_DELAY: Duration = Duration::from_millis(600);
/// Pointer drift, in pixels, that still counts as resting on the row.
pub const HOVER_MENU_SLOP: f32 = 3.;
pub const ROW_PADDING: f32 = 12.;
/// A status dot's diameter at the default size; [`scaled`] sizes it for the
/// configured font.
pub const STATUS_WIDTH: f32 = 8.;
// Unknown stays a smaller dot so it reads as "no reported status" next to the full ones.
pub const STATUS_DOT_UNKNOWN: f32 = 3.;
pub const LABEL_GAP: f32 = 8.;
// Children clear the gutter their tree lines run in, which starts at the
// parent's label column so the trunk lines up under the parent's branch.
pub const TREE_GUTTER: f32 = ROW_PADDING + STATUS_WIDTH + LABEL_GAP;
pub const CHILD_INDENT: f32 = TREE_GUTTER - ROW_PADDING + 12.;
pub const ARROW_RESERVE: f32 = 18.;
pub const HOST_ARROW_WIDTH: f32 = 12.;
pub const HOST_GAP: f32 = 6.;
pub const ICON_RESERVE: f32 = 18.;

/// What a row's label has left to fill. Only a layout test reads it here; the
/// app gates its own re-export to the builds that measure with it, which is
/// why this crate carries no build-time gate of its own.
pub const LABEL_WIDTH: f32 = SIDEBAR_WIDTH - 1. - 2. * ROW_PADDING - STATUS_WIDTH - LABEL_GAP;

pub fn line_height(font: &FontConfig) -> f32 {
    font.size * 4. / 3.
}

/// The `[sidebar] size` the marks' pixel sizes here and in the rows are
/// drawn for: its default.
const MARK_FONT_SIZE: f32 = 12.;

/// `size`, a mark's size at the default sidebar font, at `font` instead, so
/// dots, icons, and chevrons grow with `[sidebar] size` as the text beside
/// them does. Whole pixels keep their edges crisp, and the default size
/// keeps every mark as it was.
pub fn scaled(font: &FontConfig, size: f32) -> f32 {
    (size * font.size / MARK_FONT_SIZE).round()
}

/// The unknown status's dot beside full dots `diameter` across: as much
/// smaller as it is at the default size.
pub fn unknown_dot(diameter: f32) -> f32 {
    (diameter * STATUS_DOT_UNKNOWN / STATUS_WIDTH).round()
}

pub fn sidebar_width(preferred: Option<f32>, window_width: f32) -> f32 {
    // Keep useful label space and reserve at least 240 logical pixels for the terminal.
    preferred
        .unwrap_or(SIDEBAR_WIDTH)
        .clamp(160., 480.)
        .min((window_width - 240.).max(0.))
}

/// Sidebar labels are monospace by default and digits are near-uniform
/// elsewhere, so an em-fraction bounds a glyph well enough to divide a line.
pub fn glyph_width(font: &FontConfig) -> f32 {
    font.size * 0.62
}

/// Widths in glyphs for segments sharing one line, as upstream shares them:
/// every segment keeps one glyph, then they grow in turn until the line is
/// full, so a long name cannot crowd a short one out entirely.
pub fn segment_budgets(lengths: &[usize], available: usize) -> Vec<usize> {
    let mut budgets: Vec<usize> = lengths
        .iter()
        .map(|length| usize::from(*length > 0))
        .collect();
    let mut remaining = available.saturating_sub(budgets.iter().sum());
    while remaining > 0 {
        let mut grew = false;
        for (budget, length) in budgets.iter_mut().zip(lengths) {
            if *budget > 0 && *budget < *length {
                *budget += 1;
                remaining -= 1;
                grew = true;
                if remaining == 0 {
                    break;
                }
            }
        }
        if !grew {
            break;
        }
    }
    budgets
}
