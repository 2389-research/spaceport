// ABOUTME: Keeps retained Core grids inside the current accessibility viewport during resize.
// ABOUTME: Checks text, character geometry, selection, and caret boundaries before the runtime catches up.
use super::*;
use sheprd_client::protocol::CursorState;

#[test]
fn a_retained_grid_reads_only_cells_inside_the_current_canvas() {
    let screen = frame(&["abcdefghij", "klmnopqrst", "uvwxyz"], 10);
    let transcript =
        Transcript::read_frame(&screen, [], CELL_WIDTH, CELL_HEIGHT, (45., 45.)).unwrap();
    assert_eq!(text(&transcript), "abcd\nklmn");
    let nodes: Vec<_> = transcript.runs((100., 40.), 1.5).collect();
    assert_eq!(nodes.len(), 2);
    for node in nodes {
        let bounds = node.bounds().unwrap();
        assert!(bounds.x0 >= 150. && bounds.y0 >= 60.);
        assert!(bounds.x1 <= 217.5 && bounds.y1 <= 127.5, "{bounds:?}");
        for (&position, &width) in node
            .character_positions()
            .unwrap()
            .iter()
            .zip(node.character_widths().unwrap())
        {
            assert!(f64::from(position + width) <= bounds.width());
        }
    }
    let restored =
        Transcript::read_frame(&screen, [], CELL_WIDTH, CELL_HEIGHT, (100., 60.)).unwrap();
    assert_eq!(text(&restored), "abcdefghij\nklmnopqrst\nuvwxyz");
}

#[test]
fn retained_grid_selection_and_caret_stay_on_visible_cells() {
    let mut screen = frame(&["abcdefghij", "klmnopqrst", "uvwxyz"], 10);
    let transcript = Transcript::read_frame(
        &screen,
        [(0, 2..8), (2, 0..3)],
        CELL_WIDTH,
        CELL_HEIGHT,
        (45., 45.),
    )
    .unwrap();
    assert_eq!(selected(&transcript), "cd");
    for (x, y, expected) in [
        (8, 0, None),
        (2, 2, None),
        (
            2,
            1,
            Some(Caret {
                line: 1,
                character: 2,
            }),
        ),
    ] {
        screen.cursor = Some(CursorState {
            x,
            y,
            visible: true,
            shape: 0,
        });
        let transcript =
            Transcript::read_frame(&screen, [], CELL_WIDTH, CELL_HEIGHT, (45., 45.)).unwrap();
        assert_eq!(transcript.selection, expected.map(|caret| (caret, caret)));
    }
}

#[test]
fn a_canvas_without_a_complete_cell_exposes_no_grid() {
    let screen = frame(&["abc"], 3);
    for viewport in [(0., 20.), (9., 20.), (30., 19.), (f32::NAN, 20.)] {
        assert!(Transcript::read_frame(&screen, [], CELL_WIDTH, CELL_HEIGHT, viewport).is_none());
    }
}

#[test]
fn a_wholly_hidden_selection_does_not_replace_the_visible_cursor() {
    let mut screen = frame(&["abcdefghij", "klmnopqrst"], 10);
    screen.cursor = Some(CursorState {
        x: 2,
        y: 1,
        visible: true,
        shape: 0,
    });
    let transcript =
        Transcript::read_frame(&screen, [(0, 6..8)], CELL_WIDTH, CELL_HEIGHT, (45., 45.)).unwrap();
    let caret = Caret {
        line: 1,
        character: 2,
    };
    assert_eq!(transcript.selection, Some((caret, caret)));
}

#[test]
fn a_wide_character_at_the_visible_edge_has_bounded_character_geometry() {
    let mut screen = frame(&["abc界 xxxx"], 10);
    screen.cells[4] = cell(" ");
    let transcript =
        Transcript::read_frame(&screen, [], CELL_WIDTH, CELL_HEIGHT, (45., 20.)).unwrap();
    let node = transcript.runs((0., 0.), 1.).next().unwrap();
    assert_eq!(node.value(), Some("abc界"));
    assert_eq!(node.bounds().unwrap().x1, 40.);
    assert_eq!(node.character_positions().unwrap().last(), Some(&30.));
    assert_eq!(node.character_widths().unwrap().last(), Some(&10.));
}
