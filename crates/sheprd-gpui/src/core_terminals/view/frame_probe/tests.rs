// ABOUTME: Checks frame-observer text and concealment bounds without process environment changes.
// ABOUTME: Keeps diagnostic sampling finite even when a cell contains excessive combining text.
use super::sampled_rows;
use sheprd_backend::FrameData;
use sheprd_client::protocol::CellData;

#[test]
fn captured_core_frame_samples_bound_rows_cells_and_characters() {
    let cell = CellData {
        symbol: "é".repeat(512),
        fg: 0,
        bg: 0,
        modifier: crate::terminal::HIDDEN,
        skip: false,
        hyperlink: None,
    };
    let frame = FrameData {
        cells: vec![cell; 130 * 4],
        width: 130,
        height: 4,
        cursor: None,
        hyperlinks: Vec::new(),
        graphics: Vec::new(),
    };
    let sampled = sampled_rows(&frame);
    assert_eq!(sampled.len(), 3);
    for (text, hidden) in sampled {
        assert_eq!(text.chars().count(), 96);
        assert_eq!(hidden, 96);
    }
}
