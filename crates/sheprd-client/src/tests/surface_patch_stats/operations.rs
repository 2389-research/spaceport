// ABOUTME: Exercises each encoded surface path through the framed client worker.
// ABOUTME: Checks that accounting preserves rejected, empty, and saturated updates.
use super::*;
use crate::{
    patch_stats::{MutationCounters, ReconstructionCounters, Snapshot},
    protocol::{
        surface_delta::{GridUpdate, SurfaceDelta},
        surface_reuse::SurfaceReuse,
    },
    tests::surface_encoding::{
        ALL, delta_message, grid, reuse_message, scroll_message, symbols, welcome_with,
    },
};

fn recorded_stats(client: &Client, connection_id: u64) -> Snapshot {
    let stats = client.handle.patch_stats().unwrap();
    assert_eq!(stats.connection_id, connection_id);
    stats
}

fn expect_protocol_disconnect(client: &Client, error: protocol::Error) {
    let ClientEvent::Disconnected { reason, ssh: None } = event(client) else {
        panic!("an invalid update must disconnect the worker without a new surface");
    };
    assert_eq!(reason, Error::Protocol(error).to_string());
    await_worker_exit(client);
}

#[test]
fn scroll_records_main_cells_in_its_own_mutation_counters() {
    let (client, mut server) = connected_client_with_welcome(&welcome_with(&ALL));
    let connection_id = client.handle.patch_stats().unwrap().connection_id;
    let base = grid();
    send(&mut server, ServerMessage::PaneSurface(base.clone()));
    let old = received_surface(&client);
    send(&mut server, scroll_message(&base));
    let next = received_surface(&client);
    assert_eq!(symbols(&next), "bbccdd");
    assert_eq!(next.surface_revision, 2);
    assert_eq!(*old, base);

    disconnect(&client);
    let stats = recorded_stats(&client, connection_id);
    assert_eq!(stats.scroll.attempts, 1, "scroll patch attempts");
    assert_eq!(stats.scroll.successes, 1);
    assert_eq!(stats.scroll.cow_copies, 1);
    assert_eq!(stats.scroll.main_cells_copied, 6);
    assert_eq!(stats.scroll.empty_baselines, 0);
    assert_eq!(stats.ordinary, MutationCounters::default());
    assert_eq!(stats.delta, ReconstructionCounters::default());
    assert_eq!(stats.reuse, ReconstructionCounters::default());
    assert_eq!(stats.full_surface_messages, 1);
    assert!(!stats.overflow);
}

#[test]
fn delta_records_its_reconstruction_and_acceptance() {
    let (client, mut server) = connected_client_with_welcome(&welcome_with(&ALL));
    let connection_id = client.handle.patch_stats().unwrap().connection_id;
    let base = grid();
    send(&mut server, ServerMessage::PaneSurface(base.clone()));
    let old = received_surface(&client);
    send(&mut server, delta_message(&base, base.projection_revision));
    let next = received_surface(&client);
    assert_eq!(symbols(&next), "aabxcc");
    assert_eq!(next.surface_revision, 2);
    assert_eq!(*old, base);

    disconnect(&client);
    let stats = recorded_stats(&client, connection_id);
    assert_eq!(stats.delta.attempts, 1, "delta reconstruction attempts");
    assert_eq!(stats.delta.reconstructions, 1);
    assert_eq!(stats.delta.accepted, 1);
    assert_eq!(stats.delta.main_cells_copied, 6);
    assert_eq!(stats.delta.popup_baselines_unsupported, 0);
    assert_eq!(stats.ordinary, MutationCounters::default());
    assert_eq!(stats.scroll, MutationCounters::default());
    assert_eq!(stats.reuse, ReconstructionCounters::default());
    assert_eq!(stats.full_surface_messages, 1);
    assert!(!stats.overflow);
}

#[test]
fn reuse_records_its_reconstruction_and_acceptance() {
    let (client, mut server) = connected_client_with_welcome(&welcome_with(&ALL));
    let connection_id = client.handle.patch_stats().unwrap().connection_id;
    let base = grid();
    send(&mut server, ServerMessage::PaneSurface(base.clone()));
    let old = received_surface(&client);
    send(&mut server, reuse_message(&base, base.projection_revision));
    let mut expected = base.clone();
    expected.surface_revision = 2;
    assert_eq!(*received_surface(&client), expected);
    assert_eq!(*old, base);

    disconnect(&client);
    let stats = recorded_stats(&client, connection_id);
    assert_eq!(stats.reuse.attempts, 1, "reuse reconstruction attempts");
    assert_eq!(stats.reuse.reconstructions, 1);
    assert_eq!(stats.reuse.accepted, 1);
    assert_eq!(stats.reuse.main_cells_copied, 6);
    assert_eq!(stats.reuse.popup_baselines_unsupported, 0);
    assert_eq!(stats.ordinary, MutationCounters::default());
    assert_eq!(stats.scroll, MutationCounters::default());
    assert_eq!(stats.delta, ReconstructionCounters::default());
    assert_eq!(stats.full_surface_messages, 1);
    assert!(!stats.overflow);
}

#[test]
fn reuse_counts_reconstruction_work_when_acceptance_fails() {
    let (client, mut server) = connected_client_with_welcome(&welcome_with(&ALL));
    let connection_id = client.handle.patch_stats().unwrap().connection_id;
    let base = grid();
    send(&mut server, ServerMessage::PaneSurface(base.clone()));
    let old = received_surface(&client);
    let mut surface = base.clone();
    surface.surface_revision += 1;
    surface.frame.cells.clear();
    surface.frame.cursor = Some(CursorState {
        x: base.frame.width,
        y: 0,
        visible: true,
        shape: 0,
    });
    let reuse = SurfaceReuse {
        base_surface_revision: base.surface_revision,
        surface,
    };
    send(
        &mut server,
        ServerMessage::EndpointControl {
            kind: surface_reuse::MESSAGE_KIND.into(),
            data: reuse.encode().unwrap(),
        },
    );
    expect_protocol_disconnect(&client, protocol::Error::CursorBounds);
    assert_eq!(*old, base);

    let stats = recorded_stats(&client, connection_id);
    assert_eq!(stats.reuse.attempts, 1);
    assert_eq!(stats.reuse.reconstructions, 1);
    assert_eq!(stats.reuse.accepted, 0);
    assert_eq!(stats.reuse.main_cells_copied, 6);
    assert_eq!(stats.full_surface_messages, 1);
    assert!(!stats.overflow);
}

#[test]
fn delta_using_old_popup_cells_flags_unsupported_measurement() {
    let (client, mut server) = connected_client_with_welcome(&welcome_with(&ALL));
    let connection_id = client.handle.patch_stats().unwrap().connection_id;
    let mut base = grid();
    base.popup = Some(Box::new(ClientShellPopupSurface {
        terminal_id: "popup-1".into(),
        title: "Popup".into(),
        width: Some(ClientShellPopupSize::Cells(1)),
        height: Some(ClientShellPopupSize::Cells(1)),
        frame: baseline().frame,
        mouse_reporting: false,
        sgr_pixel_mouse: false,
        pixel_width: 0,
        pixel_height: 0,
    }));
    send(&mut server, ServerMessage::PaneSurface(base.clone()));
    let old = received_surface(&client);
    let mut expected = base.clone();
    expected.surface_revision += 1;
    let patched_popup_cell = &mut expected.popup.as_mut().unwrap().frame.cells[0];
    patched_popup_cell.symbol = "y".into();
    let popup_row = PaneSurfacePatchRow {
        x: 0,
        y: 0,
        cells: vec![patched_popup_cell.clone()],
    };
    let mut shell = expected.clone();
    shell.frame.cells.clear();
    shell.popup.as_mut().unwrap().frame.cells.clear();
    let delta = SurfaceDelta {
        base_projection_revision: base.projection_revision,
        base_surface_revision: base.surface_revision,
        surface: shell,
        rows: vec![],
        popup_cells: Some(GridUpdate::Patch(vec![popup_row])),
    };
    send(
        &mut server,
        ServerMessage::EndpointControl {
            kind: surface_delta::MESSAGE_KIND.into(),
            data: delta.encode().unwrap(),
        },
    );
    assert_eq!(*received_surface(&client), expected);
    assert_eq!(*old, base);

    disconnect(&client);
    let stats = recorded_stats(&client, connection_id);
    assert_eq!(stats.delta.popup_baselines_unsupported, 1);
    assert_eq!(stats.delta.main_cells_copied, 6);
    assert_eq!(stats.delta.reconstructions, 1);
    assert_eq!(stats.delta.accepted, 1);
    assert!(!stats.overflow);
}

#[test]
fn rejected_ordinary_patch_records_work_without_publishing_a_surface() {
    let (client, mut server) = connected_client();
    let connection_id = client.handle.patch_stats().unwrap().connection_id;
    let base = two_cell_baseline();
    send(&mut server, ServerMessage::PaneSurface(base.clone()));
    let old = received_surface(&client);
    send(
        &mut server,
        ordinary_patch(
            &base,
            vec![PaneSurfacePatchRow {
                x: 0,
                y: base.frame.height,
                cells: vec![base.frame.cells[0].clone()],
            }],
        ),
    );
    expect_protocol_disconnect(&client, protocol::Error::PatchRowBounds);
    assert_eq!(*old, base);

    let stats = recorded_stats(&client, connection_id);
    assert_eq!(
        stats.ordinary.attempts, 1,
        "rejected ordinary patch attempts"
    );
    assert_eq!(stats.ordinary.successes, 0);
    assert_eq!(stats.ordinary.cow_copies, 1);
    assert_eq!(stats.ordinary.main_cells_copied, 2);
    assert_eq!(stats.ordinary.empty_baselines, 0);
    assert_eq!(stats.full_surface_messages, 1);
    assert!(!stats.overflow);
}

#[test]
fn shared_empty_grid_records_an_empty_baseline() {
    let (client, mut server) = connected_client();
    let connection_id = client.handle.patch_stats().unwrap().connection_id;
    let mut base = baseline();
    base.frame.width = 0;
    base.frame.height = 0;
    base.frame.cells.clear();
    send(&mut server, ServerMessage::PaneSurface(base.clone()));
    let old = received_surface(&client);
    send(&mut server, ordinary_patch(&base, vec![]));
    let mut expected = base.clone();
    expected.surface_revision = 2;
    assert_eq!(*received_surface(&client), expected);
    assert_eq!(*old, base);

    disconnect(&client);
    let stats = recorded_stats(&client, connection_id);
    assert_eq!(
        stats.ordinary.empty_baselines, 1,
        "ordinary patch empty baselines"
    );
    assert_eq!(stats.ordinary.attempts, 1);
    assert_eq!(stats.ordinary.successes, 1);
    assert_eq!(stats.ordinary.cow_copies, 0);
    assert_eq!(stats.ordinary.cow_elapsed_ns, 0);
    assert_eq!(stats.ordinary.main_cells_copied, 0);
    assert_eq!(stats.full_surface_messages, 1);
    assert!(!stats.overflow);
}

#[test]
fn copied_cell_count_saturates_and_flags_overflow() {
    let (client, mut server) = connected_client();
    let connection_id = client.handle.patch_stats().unwrap().connection_id;
    let record = client.handle.inner.patch_stats.as_ref().unwrap();
    record.lock().unwrap().ordinary.main_cells_copied = u64::MAX - 1;
    let base = two_cell_baseline();
    send(&mut server, ServerMessage::PaneSurface(base.clone()));
    let old = received_surface(&client);
    let expected = send_ordinary_patch(&mut server, &base);
    assert_eq!(*received_surface(&client), expected);
    assert_eq!(*old, base);

    disconnect(&client);
    let stats = recorded_stats(&client, connection_id);
    assert!(stats.overflow, "ordinary copied cell count overflow");
    assert_eq!(stats.ordinary.main_cells_copied, u64::MAX);
    assert_eq!(stats.ordinary.attempts, 1);
    assert_eq!(stats.ordinary.successes, 1);
    assert_eq!(stats.ordinary.cow_copies, 1);
    assert_eq!(stats.full_surface_messages, 1);
}

#[test]
fn poisoned_record_leaves_live_surface_delivery_intact() {
    let (client, mut server) = connected_client();
    poison_patch_stats(&client.handle);

    let base = two_cell_baseline();
    send(&mut server, ServerMessage::PaneSurface(base.clone()));
    let old = received_surface(&client);
    let expected = send_ordinary_patch(&mut server, &base);
    assert_eq!(*received_surface(&client), expected);
    assert_eq!(*old, base);
    disconnect(&client);
    assert_eq!(client.handle.patch_stats(), None);
}
