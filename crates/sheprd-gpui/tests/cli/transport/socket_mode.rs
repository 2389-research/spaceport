// ABOUTME: Checks the real socket mode used by scripted CLI peers.
// ABOUTME: Preserves deadline reads when an accepted stream starts nonblocking.
use super::{Duration, UnixStream, prepare_peer};
use rustix::fs::{OFlags, fcntl_getfl};

#[test]
fn a_scripted_peer_reads_with_a_deadline_after_inheriting_nonblocking_mode() {
    let (stream, _client) = UnixStream::pair().unwrap();
    stream.set_nonblocking(true).unwrap();
    prepare_peer(&stream);
    assert!(!fcntl_getfl(&stream).unwrap().contains(OFlags::NONBLOCK));
    assert_eq!(stream.read_timeout().unwrap(), Some(Duration::from_secs(5)));
}
