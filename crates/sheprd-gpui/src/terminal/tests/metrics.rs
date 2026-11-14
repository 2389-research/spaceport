//! `CellMetrics` against the expressions it replaced, written out here as they
//! stood at each call site, so a helper that reorders a float operation fails.

use super::*;
use gpui::{Size, size};

const WIDTHS: &[f32] = &[
    0.1,
    1.,
    6.6,
    7.224_609_4,
    8.,
    8.4,
    8.5,
    9.,
    9.601_562,
    10.3,
    13.333_333,
    33.7,
];
const HEIGHTS: &[f32] = &[0.3, 1., 14.7, 17., 18.2, 20., 21.25, 30.5, 47.9];
const CELLS: &[u16] = &[
    0, 1, 2, 3, 7, 23, 24, 79, 80, 131, 255, 256, 999, 4095, 4096, 32_767, 65_535,
];

fn each(mut check: impl FnMut(CellMetrics, u16, u16)) {
    for &width in WIDTHS {
        for &height in HEIGHTS {
            for &column in CELLS {
                for &row in CELLS {
                    check(CellMetrics::new(width, height), column, row);
                }
            }
        }
    }
}

fn bits(point: Point<Pixels>) -> (u32, u32) {
    (f32::from(point.x).to_bits(), f32::from(point.y).to_bits())
}

fn size_bits(size: Size<Pixels>) -> (u32, u32) {
    (
        f32::from(size.width).to_bits(),
        f32::from(size.height).to_bits(),
    )
}

#[test]
fn origin_and_extent_match_the_hand_built_products_bit_for_bit() {
    each(|metrics, column, row| {
        let (cell_width, cell_height) = (metrics.width, metrics.height);
        // `f32::from(u16)` and `u16 as f32` are both exact, so one helper
        // serves the sites that wrote either.
        let from = point(
            px(f32::from(column) * cell_width),
            px(f32::from(row) * cell_height),
        );
        let cast = point(px(column as f32 * cell_width), px(row as f32 * cell_height));
        assert_eq!(bits(metrics.origin(column, row)), bits(from));
        assert_eq!(bits(metrics.origin(column, row)), bits(cast));
        let extent = size(
            px(f32::from(column) * cell_width),
            px(f32::from(row) * cell_height),
        );
        assert_eq!(
            size_bits(metrics.extent(column, row)),
            size_bits(extent),
            "{metrics:?} {column} {row}"
        );
    });
}

#[test]
fn bounds_match_the_hand_built_rectangle_bit_for_bit() {
    each(|metrics, x, y| {
        let (cell_width, cell_height) = (metrics.width, metrics.height);
        for (width, height) in [(0, 0), (1, 1), (80, 24), (211, 67), (65_535, 65_535)] {
            let rect = SurfaceRect {
                x,
                y,
                width,
                height,
            };
            let old = Bounds::new(
                point(
                    px(f32::from(rect.x) * cell_width),
                    px(f32::from(rect.y) * cell_height),
                ),
                size(
                    px(f32::from(rect.width) * cell_width),
                    px(f32::from(rect.height) * cell_height),
                ),
            );
            let new = metrics.bounds(rect);
            assert_eq!(bits(new.origin), bits(old.origin));
            assert_eq!(size_bits(new.size), size_bits(old.size));
        }
    });
}

#[test]
fn cell_size_and_cell_at_match_the_hand_built_expressions() {
    for &width in WIDTHS {
        for &height in HEIGHTS {
            let metrics = CellMetrics::new(width, height);
            assert_eq!(
                size_bits(metrics.cell()),
                size_bits(size(px(width), px(height)))
            );
            for x in [0., 0.49, 8.4, 99.999, 1234.5, 70_000., 9.9e9, -3.] {
                for y in [0., 19.999, 20., 771.25, 3.0e7, -0.1] {
                    let old = ((x / width).floor() as u16, (y / height).floor() as u16);
                    assert_eq!(metrics.cell_at(x, y), old, "{metrics:?} {x} {y}");
                }
            }
        }
    }
}

#[test]
fn usable_refuses_what_the_hit_tests_refused() {
    for width in [8.5, 0., -1., f32::NAN, f32::INFINITY, f32::MIN_POSITIVE] {
        for height in [20., 0., -1., f32::NAN, f32::INFINITY] {
            let (cell_width, cell_height) = (width, height);
            let refused = !cell_width.is_finite()
                || !cell_height.is_finite()
                || cell_width <= 0.
                || cell_height <= 0.;
            assert_eq!(CellMetrics::new(width, height).usable(), !refused);
        }
    }
}

/// A point inside a cell's painted bounds hit-tests to that cell.
#[test]
fn the_middle_of_a_cells_bounds_is_in_that_cell() {
    each(|metrics, column, row| {
        // Past f32's whole-number precision a pixel position cannot name a
        // single cell at all; no terminal is that large.
        if f32::from(column.max(row)) * metrics.width.max(metrics.height) > 4.0e6 {
            return;
        }
        let bounds = metrics.bounds(SurfaceRect {
            x: column,
            y: row,
            width: 1,
            height: 1,
        });
        let middle = bounds.center();
        assert_eq!(
            metrics.cell_at(f32::from(middle.x), f32::from(middle.y)),
            (column, row),
            "{metrics:?} {column} {row}"
        );
    });
}

/// The exact corner is the one place division can land a cell early: the
/// product rounds, and dividing it back may give a hair under the whole
/// number. This pins how far that goes (never more than the cell before) and
/// that the sizes terminals really use round-trip exactly.
#[test]
fn a_cells_own_corner_is_in_that_cell_or_the_one_before() {
    let mut early = 0_u32;
    each(|metrics, column, row| {
        let origin = metrics.origin(column, row);
        let (at_column, at_row) = metrics.cell_at(f32::from(origin.x), f32::from(origin.y));
        assert!(at_column == column || at_column + 1 == column);
        assert!(at_row == row || at_row + 1 == row);
        early += u32::from((at_column, at_row) != (column, row));
    });
    // The case is real, which is why hit tests never lean on a corner.
    assert!(early > 0);
    for (width, height) in [(8., 20.), (8.5, 20.), (9., 18.), (7., 17.)] {
        let metrics = CellMetrics::new(width, height);
        for cell in 0..=4096 {
            let origin = metrics.origin(cell, cell);
            assert_eq!(
                metrics.cell_at(f32::from(origin.x), f32::from(origin.y)),
                (cell, cell)
            );
        }
    }
}
