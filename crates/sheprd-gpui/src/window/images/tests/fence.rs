// ABOUTME: Pins which connection changes a pending image paste survives.
// ABOUTME: A prepared image is sent only on the selected connection it started on.

use super::*;
use crate::{
    endpoint::fence::changes::{CHANGES, apply},
    sidebar::layout_tests::fixture_window,
    window::MockPeer,
};
use gpui::{Task, TestAppContext};
use sheprd_client::protocol::ClientClipboardImageTarget;

const PANE: &str = "w1:p1";

fn pending(view: &SheprdWindow, peer: &MockPeer) -> PendingImage {
    let boot = view.live.snapshot.as_ref().unwrap().boot_id.clone();
    let upload = peer
        .client
        .handle
        .reserve_clipboard_image(&boot, ClientClipboardImageTarget::Pane(PANE.into()))
        .unwrap();
    PendingImage {
        token: Arc::new(()),
        kind: PreparationKind::Image,
        fence: view.input_fence().unwrap(),
        remote: view.selected_is_remote(),
        target: InputTarget::Pane(PANE.into()),
        cancellation: upload.cancellation_handle(),
        preparing: true,
        _task: Task::ready(()),
    }
}

#[gpui::test]
fn an_image_holds_only_on_the_selected_connection_it_started_on(cx: &mut TestAppContext) {
    let (view, cx) = cx.add_window_view(fixture_window);
    let seen = CHANGES.map(|change| {
        let peer = MockPeer::new();
        view.update(cx, |view, cx| {
            peer.prepare(view, cx);
            let image = pending(view, &peer);
            assert!(view.image_target_current(&image), "{change:?}");
            apply(view, 0, change, cx);
            view.image_target_current(&image)
        })
    });
    // Reselect, SwitchAndReturn, ReconnectUnseen, Reconnect, Reboot, Removed.
    assert_eq!(seen, [true, false, false, false, false, false]);
}
