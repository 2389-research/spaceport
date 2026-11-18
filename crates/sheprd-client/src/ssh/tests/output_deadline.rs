// ABOUTME: Holds a real pipe writer beyond a probe's read deadline.
// ABOUTME: Checks bounded waits and releases every fixture descriptor afterward.
use super::*;

#[test]
fn a_retained_writer_cannot_hold_a_probe_past_its_deadline() {
    held_writer(false);
}

#[test]
fn cancellation_does_not_wait_for_a_retained_pipe_writer() {
    held_writer(true);
}

fn held_writer(cancelled: bool) {
    let (pipe, writer) = io::pipe().unwrap();
    let reader = start_output(pipe).unwrap();
    let (sent, done) = std::sync::mpsc::sync_channel(1);
    let waiting = std::thread::spawn(move || {
        let result = finish_output(reader, Instant::now(), || cancelled);
        sent.send(result).unwrap();
    });
    let result = done.recv_timeout(Duration::from_secs(1));
    // Release the real writer before assertions, including the failing baseline.
    drop(writer);
    waiting.join().unwrap();
    let error = match result {
        Ok(Err(error)) => error,
        other => panic!("probe read ignored its deadline or cancellation: {other:?}"),
    };
    match error {
        crate::Error::Ssh(Error::Cancelled) if cancelled => {}
        crate::Error::Ssh(Error::Timeout) if !cancelled => {}
        other => panic!("unexpected pipe failure: {other:?}"),
    }
}

#[test]
fn a_timed_out_probe_releases_its_reader_without_waiting_for_the_writer() {
    let (pipe, mut writer) = io::pipe().unwrap();
    let reader = start_output(pipe).unwrap();
    assert!(finish_output(reader, Instant::now(), || false).is_err());
    let written = writer.write(b"reader must be closed");
    drop(writer);
    assert!(
        matches!(written, Err(error) if error.kind() == io::ErrorKind::BrokenPipe),
        "the timed-out probe retained its reader"
    );
}

#[test]
fn a_completed_pipe_returns_its_actual_bytes() {
    let (pipe, mut writer) = io::pipe().unwrap();
    let reader = start_output(pipe).unwrap();
    writer.write_all(b"probe output\n").unwrap();
    drop(writer);
    let bytes = finish_output(reader, Instant::now() + Duration::from_secs(1), || false).unwrap();
    assert_eq!(bytes, b"probe output\n");
}

#[test]
fn a_pipe_read_failure_retains_its_io_error() {
    let (_pipe, writer) = io::pipe().unwrap();
    let error = match start_output(writer) {
        Ok(reader) => {
            finish_output(reader, Instant::now() + Duration::from_secs(1), || false).unwrap_err()
        }
        Err(error) => error.into(),
    };
    assert!(matches!(error, crate::Error::Io(_)), "{error:?}");
}
