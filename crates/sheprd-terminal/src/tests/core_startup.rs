// ABOUTME: Replays the startup height change observed before corrupted native Core text.
// ABOUTME: Keeps the first prompt and later absolute cursor output on their authoritative rows.
use super::core_output_capture::STREAM;
use super::*;

#[test]
fn captured_core_startup_height_change_preserves_the_first_prompt() {
    let prompt = STREAM.windows(3).position(|part| part == b"PS ").unwrap();
    let expected = fed(112, 34, STREAM).frame();
    let mut terminal = emulator(80, 24);
    terminal.feed(&STREAM[..prompt]);
    terminal.resize(size(112, 34));
    terminal.feed(&STREAM[prompt..]);
    let frame = terminal.frame();
    frame.validate().unwrap();
    assert_eq!(line(&frame, 0), line(&expected, 0));
    assert_eq!(line(&frame, 1), "SP_A11Y_7");
    assert_eq!(line(&frame, 2), line(&expected, 2));
}
