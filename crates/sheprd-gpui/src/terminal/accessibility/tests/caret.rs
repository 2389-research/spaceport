// ABOUTME: Checks terminal cursor positions exposed as accessibility carets.
// ABOUTME: Preserves real selections and wide-cell character boundaries.
use super::*;
use sheprd_client::protocol::CursorState;

#[test]
fn an_unselected_terminal_exposes_the_programs_cursor() {
    let mut screen = surface(&["ab   right", "cd"]);
    screen.frame.cursor = Some(CursorState {
        x: 4,
        y: 1,
        visible: true,
        shape: 0,
    });
    let transcript = read(&screen, None);
    let caret = Caret {
        line: 1,
        character: 2,
    };
    assert_eq!(transcript.selection, Some((caret, caret)));
}

#[test]
fn a_real_selection_takes_precedence_over_the_programs_cursor() {
    let mut screen = surface(&["abcd right"]);
    screen.frame.cursor = Some(CursorState {
        x: 0,
        y: 0,
        visible: true,
        shape: 0,
    });
    let selection = drag(&screen, (16., 1.), (39., 1.));
    assert_eq!(selected(&read(&screen, Some(&selection))), "cd");
}

#[test]
fn a_hidden_or_outside_cursor_does_not_claim_a_caret() {
    let mut screen = surface(&["abcd right"]);
    for (x, visible) in [(8, true), (1, false)] {
        screen.frame.cursor = Some(CursorState {
            x,
            y: 0,
            visible,
            shape: 0,
        });
        assert_eq!(read(&screen, None).selection, None);
    }
}

#[test]
fn full_grid_reading_exposes_unicode_caret_and_polite_output() {
    let mut frame = frame(&["a界", "done"], 5);
    frame.cursor = Some(CursorState {
        x: 4,
        y: 1,
        visible: true,
        shape: 0,
    });
    let transcript =
        Transcript::read_frame(&frame, [], CELL_WIDTH, CELL_HEIGHT, (50., 40.)).unwrap();
    let ids = [NodeId(1), NodeId(2)];
    let mut node = Node::new(Role::Terminal);
    transcript.decorate_parent(&mut node, &ids);
    assert_eq!(node.live(), Some(Live::Polite));
    let selection = node.text_selection().unwrap();
    assert_eq!(selection.anchor, selection.focus);
    assert_eq!(selection.focus.node, ids[1]);
    assert_eq!(selection.focus.character_index, 4);
    assert_eq!(
        transcript
            .runs((0., 0.), 1.)
            .map(|node| node.value().unwrap().to_string())
            .collect::<Vec<_>>(),
        ["a界\n", "done"]
    );
    let selected =
        Transcript::read_frame(&frame, [(0, 0..1)], CELL_WIDTH, CELL_HEIGHT, (50., 40.)).unwrap();
    assert_eq!(
        selected.selection,
        Some((
            Caret {
                line: 0,
                character: 0
            },
            Caret {
                line: 0,
                character: 1
            }
        ))
    );
    frame.cells.clear();
    assert!(Transcript::read_frame(&frame, [], CELL_WIDTH, CELL_HEIGHT, (50., 40.)).is_none());
}
