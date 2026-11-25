// ABOUTME: Holds new parked bridges until the primary has a serving snapshot.
// ABOUTME: Keeps reconciliation on owned explicit sockets while checking startup gates.
use super::*;
use gpui::Window;
use spaceport_client::ConnectTarget;
use std::path::Path;

pub(super) fn prepare_groups(
    view: &mut SpaceportWindow,
    window: &mut Window,
    cx: &mut Context<SpaceportWindow>,
    socket: &Path,
    peer: &MockPeer,
) -> GroupId {
    let mut connection = ConnectionBridge::new(ConnectTarget::Socket(socket.into()));
    connection.handle = Some(peer.client.handle.clone());
    view.endpoints[0].connection = connection;
    *view.endpoints[0].live_mut() = view.live.clone();
    view.command(Command::SplitEditor, window, cx);
    let slots = view.group_slots();
    assert_eq!(slots.len(), 2, "the fixture must show two terminal groups");
    let (left, right) = (slots[0].id, slots[1].id);
    terminals_side_by_side(view, left, right);
    assert_eq!(view.group_pick(left), Some(Pick::Herdr("t1".into())));
    assert_eq!(view.primary_group(), Some(right));
    assert!(view.group_terminals.parked.is_empty());
    assert!(view.group_terminals.retry.is_empty());
    assert!(
        !view.endpoints[0]
            .connection
            .handle
            .as_ref()
            .unwrap()
            .is_disconnected()
    );
    left
}

#[gpui::test]
fn new_parked_connections_wait_for_the_primary_snapshot(cx: &mut TestAppContext) {
    let root = tempfile::tempdir().unwrap();
    let socket = root.path().join("parked.sock");
    let peer = MockPeer::new();
    let (view, cx) = fixture_window_on_t0(cx);
    cx.update(|window, cx| {
        view.update(cx, |view, cx| {
            let left = prepare_groups(view, window, cx, &socket, &peer);
            let key = view.browser_key().unwrap();

            let snapshot = view.endpoints[0].live_mut().snapshot.take().unwrap();
            assert!(view.live.snapshot.is_some(), "the layout still needs t1");
            view.reconcile_group_terminals(cx);
            let created_before_snapshot = view
                .group_terminals
                .parked
                .iter()
                .any(|parked| parked.group == left);

            view.endpoints[0].live_mut().snapshot = Some(snapshot);
            view.reconcile_group_terminals(cx);
            let parked = view
                .group_terminals
                .parked
                .iter()
                .find(|parked| parked.group == left)
                .unwrap();
            assert_eq!(parked.workspace, key);
            assert!(matches!(
                &parked.connection.target,
                ConnectTarget::Socket(path) if path == &socket
            ));
            assert!(
                !created_before_snapshot,
                "a primary without a snapshot must not create parked bridges"
            );
        })
    });
}
