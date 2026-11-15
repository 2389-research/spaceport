// ABOUTME: Pins which connection changes a pending file transfer survives.
// ABOUTME: A transfer finishes only on the selected connection it started on.

use super::*;
use crate::endpoint::fence::changes::{CHANGES, apply};

#[gpui::test]
fn a_transfer_holds_only_on_the_selected_connection_it_started_on(cx: &mut TestAppContext) {
    let (fixture, cx) = cx.add_window_view(fixture);
    let view = fixture.read_with(cx, |fixture, _| fixture.view.clone().unwrap());
    let seen = CHANGES.map(|change| {
        let peer = Peer::new();
        view.update(cx, |view, cx| {
            peer.prepare(view, cx);
            let transfer = pending(view, InputTarget::Pane("w1:p1".into()));
            assert!(view.file_transfer_current(&transfer), "{change:?}");
            apply(view, 0, change, cx);
            view.file_transfer_current(&transfer)
        })
    });
    // Reselect, SwitchAndReturn, ReconnectUnseen, Reconnect, Reboot, Removed.
    assert_eq!(seen, [true, false, false, false, false, false]);
}
