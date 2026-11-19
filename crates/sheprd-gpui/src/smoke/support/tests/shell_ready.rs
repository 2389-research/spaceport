// ABOUTME: Check that a step types only once the shell reached its own prompt.
// ABOUTME: Keep typed-ahead input out of a shell that is still starting.
use crate::smoke::shell_ready;
use sheprd_client::protocol::{CellData, CursorState, FrameData};

#[test]
fn a_shell_is_ready_once_its_prompt_precedes_the_cursor() {
    let frame = |text: &str, x: u16| FrameData {
        width: 12,
        height: 1,
        cells: format!("{text:<12}")
            .chars()
            .map(|ch| CellData {
                symbol: ch.to_string(),
                skip: false,
                fg: 0,
                bg: 0,
                modifier: 0,
                hyperlink: None,
            })
            .collect(),
        cursor: Some(CursorState {
            x,
            y: 0,
            visible: true,
            shape: 0,
        }),
        hyperlinks: vec![],
        graphics: vec![],
    };
    assert!(shell_ready(&frame("LIVE> ", 6)));
    assert!(shell_ready(&frame("PS C:\\h> ", 9)));
    assert!(!shell_ready(&frame("", 0)));
    assert!(!shell_ready(&frame("Loading", 7)));
    // Text after the cursor is not the prompt.
    assert!(!shell_ready(&frame("ab LIVE>", 2)));
}
