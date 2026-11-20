// ABOUTME: Replays the byte-exact private Core stream captured during native text corruption.
// ABOUTME: Checks real emulator output and records bounded resize interleavings for diagnosis.
use super::*;

#[path = "../../../test-support/core_output_capture.rs"]
mod capture;
use capture::STREAM;

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

#[test]
fn captured_core_stream_records_resize_interleavings_for_diagnosis() {
    let mode = b"\x1b[?9001h";
    let first_live = STREAM
        .windows(mode.len())
        .position(|part| part == mode)
        .unwrap()
        + mode.len();
    for (initial, current, at) in [
        (112, 112, 0),
        (64, 64, 0),
        (112, 64, first_live),
        (112, 64, STREAM.len()),
        (64, 112, first_live),
    ] {
        let mut terminal = emulator(initial, 34);
        terminal.feed(&STREAM[..at]);
        terminal.resize(size(current, 34));
        terminal.feed(&STREAM[at..]);
        let frame = terminal.frame();
        frame.validate().unwrap();
        let rows: Vec<_> = (0..4).map(|row| line(&frame, row)).collect();
        eprintln!("CORE_REPLAY initial={initial} current={current} resize_at={at} rows={rows:?}");
    }
}
