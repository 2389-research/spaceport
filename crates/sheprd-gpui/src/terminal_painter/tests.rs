// ABOUTME: Shares frame fixtures for the actual terminal painter and its caches.
// ABOUTME: Registers scoped rendering, geometry, and image lifecycle regressions.
use super::*;

use crate::terminal::*;
#[cfg(feature = "integration-test")]
use crate::terminal_painter::area::whole;
use crate::terminal_painter::backgrounds::background_spans;
use crate::terminal_painter::composition::{byte_index, composition_origin};
use crate::terminal_painter::decorations::decoration_offsets;
use crate::terminal_painter::diagnostics::{REPORT_INTERVAL, SLOW_PAINT};
use crate::terminal_painter::grid::{background_extent, grid_corners, last_row_has_separator};
use crate::terminal_painter::links::link_bounds;
use core::prelude::v1::test;
use gpui::*;
use sheprd_client::protocol::{CellData, FrameData, SurfaceRect};
use std::{
    sync::Arc,
    time::{Duration, Instant},
};

mod backdrop;
mod cells;
mod composition;
mod diagnostics;
mod font_weight;
mod glyph_cache;
mod glyph_eviction;
mod grid_edges;
// Paint counts are recorded only with the integration-test feature.
#[cfg(feature = "integration-test")]
mod layers;
mod link_underline;
mod placed_images;
mod prompt_geometry;
mod theme_cache;

fn cell(symbol: &str) -> CellData {
    CellData {
        symbol: symbol.into(),
        fg: 0,
        bg: 0,
        modifier: 0,
        skip: false,
        hyperlink: None,
    }
}
