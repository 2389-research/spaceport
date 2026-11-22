// ABOUTME: Replays the captured Core fixture through the actual terminal emulator.
// ABOUTME: Checks its generated output and equality across fragmented input.
use super::core_output_capture::STREAM;
use super::*;

#[test]
fn captured_core_stream_exposes_the_authored_output_at_its_runtime_size() {
    let terminal = fed(112, 34, STREAM);
    let frame = terminal.frame();
    frame.validate().unwrap();
    assert!(line(&frame, 0).ends_with("echo SP_A11Y_7"));
    assert_eq!(line(&frame, 1), "SP_A11Y_7");
    assert!(terminal.modes().win32_input);
}

#[test]
fn captured_core_stream_matches_at_every_fragment_boundary() {
    let expected = fed(112, 34, STREAM).frame();
    for at in 1..STREAM.len() {
        let mut terminal = emulator(112, 34);
        terminal.feed(&STREAM[..at]);
        terminal.feed(&STREAM[at..]);
        assert_eq!(terminal.frame(), expected, "capture split at {at}");
    }
}
