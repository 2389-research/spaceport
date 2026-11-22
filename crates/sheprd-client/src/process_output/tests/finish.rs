// ABOUTME: Checks that finishing a capture keeps what arrived when a writer stays open.
// ABOUTME: Uses real kernel pipes and releases each fixture writer before asserting.
use super::*;

#[test]
fn finish_keeps_the_captured_tail_while_another_writer_holds_the_pipe() {
    let (pipe, mut writer) = io::pipe().unwrap();
    let capture = capture(pipe, Retain::Tail(3)).unwrap();
    writer.write_all(b"abcdefgh").unwrap();
    let kept = capture.finish(Duration::ZERO);
    drop(writer);
    assert_eq!(kept.unwrap(), b"fgh");
}

#[test]
fn finish_returns_the_whole_output_once_the_pipe_closes() {
    let (pipe, mut writer) = io::pipe().unwrap();
    let capture = capture(pipe, Retain::Prefix(64)).unwrap();
    writer.write_all(b"complete").unwrap();
    drop(writer);
    assert_eq!(capture.finish(Duration::from_secs(1)).unwrap(), b"complete");
}

#[test]
fn a_finished_capture_closes_its_reader() {
    let (pipe, mut writer) = io::pipe().unwrap();
    let capture = capture(pipe, Retain::Tail(8)).unwrap();
    capture.finish(Duration::ZERO).unwrap();
    let written = writer.write(b"reader must be closed");
    drop(writer);
    assert!(
        matches!(written, Err(error) if error.kind() == io::ErrorKind::BrokenPipe),
        "the finished capture retained its reader"
    );
}
