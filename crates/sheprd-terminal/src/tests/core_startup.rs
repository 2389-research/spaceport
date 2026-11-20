// ABOUTME: Replays the startup height change observed before corrupted native Core text.
// ABOUTME: Keeps the first prompt and later absolute cursor output on their authoritative rows.
use super::*;

#[path = "../../../test-support/core_output_capture.rs"]
mod capture;

#[test]
fn captured_core_startup_height_change_preserves_the_first_prompt() {
    let stream = capture::STREAM;
    let prompt = stream.windows(3).position(|part| part == b"PS ").unwrap();
    let expected = fed(112, 34, stream).frame();
    let mut terminal = emulator(80, 24);
    terminal.feed(&stream[..prompt]);
    terminal.resize(size(112, 34));
    terminal.feed(&stream[prompt..]);
    let frame = terminal.frame();
    frame.validate().unwrap();
    let rows: Vec<_> = (0..3).map(|row| line(&frame, row)).collect();
    eprintln!("CORE_STARTUP_REPLAY rows={rows:?}");
    assert_eq!(line(&frame, 0), line(&expected, 0));
    assert_eq!(line(&frame, 1), "SP_A11Y_7");
    assert_eq!(line(&frame, 2), line(&expected, 2));
}
