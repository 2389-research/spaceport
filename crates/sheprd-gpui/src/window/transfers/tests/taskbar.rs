use super::*;
use crate::progress::Progress;

#[gpui::test]
fn taskbar_follows_the_copy_until_it_is_cancelled(cx: &mut TestAppContext) {
    let (fixture, cx) = cx.add_window_view(fixture);
    let view = fixture.read_with(cx, |fixture, _| fixture.view.clone().unwrap());
    let mut peer = Peer::new();
    view.update(cx, |view, cx| {
        peer.prepare(view, cx);
        assert_eq!(view.taskbar_progress(), None);
        let transfer = pending(view, InputTarget::Pane("w1:p1".into()));
        let (sent, total, cancelled) = (
            transfer.sent.clone(),
            transfer.total.clone(),
            transfer.cancelled.clone(),
        );
        view.file_transfer = Some(transfer);
        view.poll_file_transfer(cx);
        // The size is not known yet.
        assert_eq!(view.taskbar_progress(), Some(Progress::Busy));
        total.store(400, Ordering::Relaxed);
        sent.store(100, Ordering::Relaxed);
        view.poll_file_transfer(cx);
        assert_eq!(view.taskbar_progress(), Some(Progress::Fraction(0.25)));
        cancelled.store(true, Ordering::Release);
        view.poll_file_transfer(cx);
        assert_eq!(view.taskbar_progress(), None);
    });
    peer.sentinel(&view, cx);
}
