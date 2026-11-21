// ABOUTME: Checks surface patch accounting through the real framed client worker.
// ABOUTME: Exercises ownership of published, unpublished, and weakly held baselines.
use super::*;

mod operations;

fn connected_client() -> (Client, Stream) {
    connected_client_with_welcome(WELCOME)
}

fn connected_client_with_welcome(welcome: &str) -> (Client, Stream) {
    let (stream, mut server) = Stream::pair().unwrap();
    let client = connect_with_connector(
        ConnectTarget::Local,
        ConnectOptions::default(),
        true,
        |_, _| Ok(stream),
    )
    .unwrap();
    handshake_with_welcome(&mut server, welcome);
    assert!(matches!(event(&client), ClientEvent::Connected(_)));
    assert!(matches!(event(&client), ClientEvent::Snapshot(s) if s.revision == 7));
    (client, server)
}

fn disconnect(client: &Client) {
    client.handle.disconnect();
    await_worker_exit(client);
}

fn await_worker_exit(client: &Client) {
    assert!(matches!(
        client.events.recv_timeout(Duration::from_secs(3)),
        Err(crossbeam_channel::RecvTimeoutError::Disconnected)
    ));
}

fn two_cell_baseline() -> PaneSurfaceFrame {
    let mut frame = baseline();
    frame.frame.width = 2;
    frame.frame.cells.push(CellData {
        symbol: "z".into(),
        ..frame.frame.cells[0].clone()
    });
    frame
}

fn send_ordinary_patch(server: &mut Stream, base: &PaneSurfaceFrame) -> PaneSurfaceFrame {
    let mut expected = base.clone();
    expected.surface_revision += 1;
    expected.frame.cells[0].symbol = "y".into();
    send(
        server,
        ordinary_patch(
            base,
            vec![PaneSurfacePatchRow {
                x: 0,
                y: 0,
                cells: vec![expected.frame.cells[0].clone()],
            }],
        ),
    );
    expected
}

fn ordinary_patch(base: &PaneSurfaceFrame, rows: Vec<PaneSurfacePatchRow>) -> ServerMessage {
    ServerMessage::PaneSurfacePatch(PaneSurfacePatch {
        boot_id: base.boot_id.clone(),
        projection_revision: base.projection_revision,
        base_surface_revision: base.surface_revision,
        surface_revision: base.surface_revision + 1,
        rows,
        panes: vec![],
        cursor: None,
    })
}

fn received_surface(client: &Client) -> Arc<PaneSurfaceFrame> {
    let ClientEvent::Surface(surface) = event(client) else {
        panic!("surface must arrive in frame order");
    };
    surface
}

fn assert_ordinary_counts(
    client: &Client,
    connection_id: u64,
    cow_copies: u64,
    main_cells_copied: u64,
) {
    disconnect(client);
    let stats = client.handle.patch_stats().unwrap();
    assert_eq!(stats.connection_id, connection_id);
    assert!(!stats.overflow);
    assert_eq!(stats.ordinary.attempts, 1, "ordinary patch attempts");
    assert_eq!(stats.ordinary.successes, 1);
    assert_eq!(stats.ordinary.cow_copies, cow_copies);
    assert_eq!(stats.ordinary.main_cells_copied, main_cells_copied);
    assert_eq!(stats.ordinary.empty_baselines, 0);
    assert_eq!(stats.full_surface_messages, 1);
}

#[test]
fn retained_published_baseline_copies_every_main_cell() {
    let (client, mut server) = connected_client();
    let connection_id = client.handle.patch_stats().unwrap().connection_id;
    let base = two_cell_baseline();
    send(&mut server, ServerMessage::PaneSurface(base.clone()));
    let old = received_surface(&client);
    let expected = send_ordinary_patch(&mut server, &base);
    assert_eq!(*received_surface(&client), expected);
    assert_eq!(*old, base);
    assert_ordinary_counts(&client, connection_id, 1, 2);
}

#[test]
fn unpublished_baseline_patches_without_a_copy() {
    let (client, mut server) = connected_client();
    let connection_id = client.handle.patch_stats().unwrap().connection_id;
    let mut base = two_cell_baseline();
    base.projection_revision = 8;
    send(&mut server, ServerMessage::PaneSurface(base.clone()));
    let expected = send_ordinary_patch(&mut server, &base);
    let mut snapshot: Value = serde_json::from_str(SNAPSHOT).unwrap();
    snapshot["revision"] = base.projection_revision.into();
    send(
        &mut server,
        ServerMessage::EndpointControl {
            kind: ENDPOINT_SNAPSHOT_KIND.into(),
            data: snapshot.to_string(),
        },
    );
    assert!(matches!(event(&client), ClientEvent::Snapshot(s) if s.revision == 8));
    assert_eq!(*received_surface(&client), expected);
    assert_ordinary_counts(&client, connection_id, 0, 0);
}

#[test]
fn weak_only_baseline_relocates_without_copying_main_cells() {
    let (client, mut server) = connected_client();
    let connection_id = client.handle.patch_stats().unwrap().connection_id;
    let base = two_cell_baseline();
    send(&mut server, ServerMessage::PaneSurface(base.clone()));
    let first = received_surface(&client);
    assert_eq!(*first, base);
    let weak = Arc::downgrade(&first);
    let cells = first.frame.cells.as_ptr();
    drop(first);

    let expected = send_ordinary_patch(&mut server, &base);
    let patched = received_surface(&client);
    assert_eq!(*patched, expected);
    assert!(weak.upgrade().is_none());
    assert_eq!(patched.frame.cells.as_ptr(), cells);
    assert_ordinary_counts(&client, connection_id, 0, 0);
}

#[test]
fn cloned_handles_share_identity_and_new_connections_advance_it() {
    let (first, _server) = connected_client();
    let first_id = first.handle.patch_stats().unwrap().connection_id;
    let cloned = first.handle.clone();
    disconnect(&first);
    assert_eq!(cloned.patch_stats().unwrap().connection_id, first_id);

    let (second, _server) = connected_client();
    let second_id = second.handle.patch_stats().unwrap().connection_id;
    disconnect(&second);
    assert!(second_id > first_id);
}

#[test]
fn contended_patch_stats_are_unavailable() {
    let (client, _server) = connected_client();
    disconnect(&client);
    let locked = client
        .handle
        .inner
        .patch_stats
        .as_ref()
        .unwrap()
        .lock()
        .unwrap();
    let handle = client.handle.clone();
    let (send_result, result) = bounded(1);
    let observer = thread::spawn(move || send_result.send(handle.patch_stats()).unwrap());
    let snapshot = result.recv_timeout(Duration::from_secs(3));
    drop(locked);
    observer.join().unwrap();
    assert_eq!(snapshot.unwrap(), None);
    assert!(client.handle.patch_stats().is_some());
}

#[test]
fn poisoned_patch_stats_are_unavailable() {
    let (client, _server) = connected_client();
    disconnect(&client);
    let handle = client.handle.clone();
    let observer = thread::spawn(move || {
        let _locked = handle.inner.patch_stats.as_ref().unwrap().lock().unwrap();
        panic!("poison the stats fixture");
    });
    assert!(observer.join().is_err());
    assert_eq!(client.handle.patch_stats(), None);
}
