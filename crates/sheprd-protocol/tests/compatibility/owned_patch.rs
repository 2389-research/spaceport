// ABOUTME: Checks allocation transfer when accepted patches consume owned cells.
// ABOUTME: Preserves complete surface values and ordered overlapping row updates.
use super::*;

#[test]
fn accepted_patch_moves_owned_cells_and_keeps_the_last_overlapping_row() {
    let mut frame = surface();
    frame.frame.width = 2;
    frame.frame.cells.push(frame.frame.cells[0].clone());
    let cell = |symbol: &str| CellData {
        symbol: symbol.repeat(8),
        ..frame.frame.cells[0].clone()
    };
    let first = cell("owned first cell");
    let overwritten = cell("owned overwritten cell");
    let last = cell("owned last cell");
    let first_allocation = first.symbol.as_ptr();
    let last_allocation = last.symbol.as_ptr();
    let mut expected = frame.clone();
    expected.frame.cells = vec![first.clone(), last.clone()];
    expected.frame.cursor = None;
    expected.surface_revision = 2;

    frame
        .apply_patch(PaneSurfacePatch {
            boot_id: frame.boot_id.clone(),
            projection_revision: frame.projection_revision,
            base_surface_revision: frame.surface_revision,
            surface_revision: 2,
            rows: vec![
                PaneSurfacePatchRow {
                    x: 0,
                    y: 0,
                    cells: vec![first, overwritten],
                },
                PaneSurfacePatchRow {
                    x: 1,
                    y: 0,
                    cells: vec![last],
                },
            ],
            panes: vec![],
            cursor: None,
        })
        .unwrap();

    assert_eq!(frame, expected);
    assert_eq!(frame.frame.cells[0].symbol.as_ptr(), first_allocation);
    assert_eq!(frame.frame.cells[1].symbol.as_ptr(), last_allocation);
}
