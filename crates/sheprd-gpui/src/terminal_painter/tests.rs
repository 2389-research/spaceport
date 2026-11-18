// ABOUTME: Shares frame fixtures for the actual terminal painter and its caches.
// ABOUTME: Registers scoped rendering, geometry, and image lifecycle regressions.
use super::*;
use core::prelude::v1::test;
use std::sync::Arc;

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
