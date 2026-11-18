// ABOUTME: Exercises pipe retention through real kernel pipes and bounded waits.
// ABOUTME: Verifies capture completion releases its descriptor before the writer closes.
use super::*;
use std::io::Write;

fn retained(retain: Retain, bytes: &[u8]) -> Vec<u8> {
    let (pipe, mut writer) = io::pipe().unwrap();
    let capture = capture(pipe, retain).unwrap();
    writer.write_all(bytes).unwrap();
    drop(writer);
    capture
        .recv_timeout(Duration::from_secs(1))
        .unwrap()
        .unwrap()
}

#[test]
fn retention_keeps_the_requested_prefix_limit_or_tail() {
    assert_eq!(retained(Retain::Prefix(3), b"abcdefgh"), b"abc");
    assert_eq!(retained(Retain::Limited(3), b"abcdefgh"), b"abc");
    assert_eq!(retained(Retain::Tail(3), b"abcdefgh"), b"fgh");
    assert!(retained(Retain::Prefix(0), b"discarded").is_empty());
    assert!(retained(Retain::Tail(0), b"discarded").is_empty());
}

#[test]
fn prefix_and_tail_remain_bounded_across_multiple_reads() {
    let bytes: Vec<_> = (0..32_000).map(|n| (n % 251) as u8).collect();
    assert_eq!(retained(Retain::Prefix(17), &bytes), bytes[..17]);
    assert_eq!(
        retained(Retain::Tail(17), &bytes),
        bytes[bytes.len() - 17..]
    );
}

#[test]
fn dropping_a_capture_closes_a_reader_even_while_its_writer_stays_open() {
    let (pipe, mut writer) = io::pipe().unwrap();
    let capture = capture(pipe, Retain::Prefix(64)).unwrap();
    drop(capture);
    assert!(
        matches!(writer.write(b"closed"), Err(error) if error.kind() == io::ErrorKind::BrokenPipe)
    );
}

#[test]
fn a_zero_limit_finishes_without_waiting_for_eof() {
    let (pipe, writer) = io::pipe().unwrap();
    let capture = capture(pipe, Retain::Limited(0)).unwrap();
    assert!(
        capture
            .recv_timeout(Duration::from_secs(1))
            .unwrap()
            .unwrap()
            .is_empty()
    );
    drop(capture);
    drop(writer);
}
