// ABOUTME: Checks shared read admission and native-worker lease ownership.
// ABOUTME: Uses private app state and an exactly owned fixture thread.
#![allow(clippy::unwrap_used)]
use super::super::pending::{MAX_READS, reserve};
use super::*;

#[gpui::test]
fn cancellation_does_not_free_a_running_read_permit(cx: &mut gpui::TestAppContext) {
    let mut permits = cx.update(|cx| {
        (0..MAX_READS)
            .map(|_| reserve(cx).unwrap())
            .collect::<Vec<_>>()
    });
    let permit = permits.pop().unwrap();
    let (release, released) = std::sync::mpsc::sync_channel(1);
    let worker = std::thread::spawn(move || {
        let _ = released.recv_timeout(std::time::Duration::from_secs(5));
        drop(permit);
    });
    assert!(cx.update(|cx| matches!(reserve(cx), Err(crate::Error::Clipboard(Error::Busy)))));
    release.send(()).unwrap();
    worker.join().unwrap();
    assert!(cx.update(|cx| reserve(cx).is_ok()));
    drop(permits);
}
