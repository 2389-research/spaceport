// ABOUTME: Exercises mobile projection using desktop protocol fixtures.
// ABOUTME: Verifies stale input, selected terminal geometry and navigation fences.
use super::*;
use spaceport_client::protocol::*;
use std::sync::Arc;

pub(crate) fn snapshot() -> ClientShellSnapshot {
    serde_json::from_str(include_str!(
        "../../../../crates/spaceport-protocol/tests/fixtures/endpoint-snapshot-v1.json"
    ))
    .unwrap()
}

fn cells(text: &str, width: u16) -> FrameData {
    let height = u16::try_from(text.chars().count() / usize::from(width)).unwrap();
    FrameData {
        cells: text
            .chars()
            .map(|c| CellData {
                symbol: c.to_string(),
                fg: 1,
                bg: 0,
                modifier: 0,
                skip: false,
                hyperlink: None,
            })
            .collect(),
        width,
        height,
        cursor: Some(CursorState {
            x: 2,
            y: 0,
            visible: true,
            shape: 0,
        }),
        hyperlinks: vec![],
        graphics: vec![],
    }
}

pub(crate) fn surface() -> PaneSurfaceFrame {
    let rect = SurfaceRect {
        x: 1,
        y: 0,
        width: 2,
        height: 2,
    };
    PaneSurfaceFrame {
        boot_id: "boot-v1".into(),
        projection_revision: 7,
        surface_revision: 1,
        frame: cells("abcdef", 3),
        panes: vec![PaneSurfacePane {
            pane_id: "w1:p1".into(),
            // Settled: an odd revision is a write in progress, which no
            // scrollback request may name.
            content_revision: 2,
            rect,
            inner_rect: rect,
            scrollbar_rect: None,
            scroll: None,
            focused: true,
            mouse_reporting: false,
            sgr_pixel_mouse: false,
            alternate_screen_active: false,
            pixel_width: 16,
            pixel_height: 32,
        }],
        splits: vec![],
        popup: None,
        graphics: Default::default(),
    }
}

#[test]
fn input_names_the_displayed_pane_and_never_the_pane_beneath_a_popup() {
    let mut state = Projection::new(geometry(3, 2, 8, 16).unwrap());
    state.snapshot(Arc::new(snapshot()));
    assert!(state.view().input_target.is_none());
    let mut frame = surface();
    state.surface(Arc::new(frame.clone()));
    let view = state.view();
    let pane_target = view.input_target.unwrap();
    let term = view.terminal.unwrap();
    assert_eq!(
        term.cells
            .iter()
            .map(|c| c.symbol.as_str())
            .collect::<String>(),
        "bcef"
    );
    assert_eq!((term.cursor_x, term.cursor_y), (Some(1), Some(0)));
    assert_eq!(view.panes[0].status, "blocked");
    frame.popup = Some(Box::new(ClientShellPopupSurface {
        terminal_id: "confirm".into(),
        title: "Confirm".into(),
        width: None,
        height: None,
        frame: cells("OK", 2),
        mouse_reporting: false,
        sgr_pixel_mouse: false,
        pixel_width: 16,
        pixel_height: 16,
    }));
    state.surface(Arc::new(frame));
    assert_ne!(state.target(), Some(pane_target));
    assert_eq!(state.target().unwrap().popup_id.as_deref(), Some("confirm"));
    let popup = state.view().terminal.unwrap();
    assert_eq!(
        popup
            .cells
            .iter()
            .map(|c| c.symbol.as_str())
            .collect::<String>(),
        "OK"
    );
}

#[test]
fn mismatched_revision_or_geometry_cannot_accept_input() {
    let mut state = Projection::new(geometry(3, 2, 8, 16).unwrap());
    state.snapshot(Arc::new(snapshot()));
    let mut frame = surface();
    frame.projection_revision += 1;
    state.surface(Arc::new(frame));
    assert!(state.target().is_none());
    state.surface(Arc::new(surface()));
    assert!(state.target().is_some());
    state.options = geometry(4, 2, 8, 16).unwrap();
    assert!(state.target().is_none());
}

#[test]
fn barrier_needs_the_acknowledged_revision_and_requested_pane() {
    let mut state = Projection::new(geometry(3, 2, 8, 16).unwrap());
    state.snapshot(Arc::new(snapshot()));
    state.surface(Arc::new(surface()));
    state.start_fence(
        "choose".into(),
        OwnedNavigationTarget::Pane("other".into()),
        Ok("fence".into()),
    );
    assert!(state.target().is_none());
    state.receipt("choose", &serde_json::json!({"result": {}}));
    state.receipt("fence", &serde_json::json!({"result": {"type":"client_shell_surface_set","active":true,"projection_revision":8}}));
    assert!(state.target().is_none());
    let mut next = snapshot();
    next.revision = 8;
    let mut frame = surface();
    frame.projection_revision = 8;
    state.snapshot(Arc::new(next.clone()));
    state.surface(Arc::new(frame.clone()));
    assert!(state.target().is_none());
    next.focused_pane_id = Some("other".into());
    next.panes[0].pane_id = "other".into();
    frame.panes[0].pane_id = "other".into();
    state.snapshot(Arc::new(next));
    state.surface(Arc::new(frame));
    assert_eq!(state.target().unwrap().pane_id, "other");
}

#[test]
fn older_connection_target_cannot_address_the_reopened_session() {
    let options = geometry(3, 2, 8, 16).unwrap();
    let mut first = Projection::new(options);
    let mut second = Projection::new(options);
    for state in [&mut first, &mut second] {
        state.snapshot(Arc::new(snapshot()));
        state.surface(Arc::new(surface()));
    }
    assert_ne!(first.target().unwrap(), second.target().unwrap());
}

#[test]
fn wheel_targets_the_displayed_program_and_bounds_a_single_gesture() {
    let mut state = Projection::new(geometry(3, 2, 8, 16).unwrap());
    state.snapshot(Arc::new(snapshot()));
    let mut frame = surface();
    state.surface(Arc::new(frame.clone()));
    let target = state.target().unwrap();
    assert!(matches!(
        state.wheel(&target, 3, 1, 0).unwrap().as_slice(),
        [ClientPaneInputEvent::Mouse {
            kind: ClientMouseKind::ScrollUp,
            lines: 3,
            position: ClientMousePosition::Cell { column: 1, row: 0 },
            ..
        }]
    ));
    assert!(state.wheel(&target, 1, 2, 0).is_err());
    frame.panes[0].alternate_screen_active = true;
    frame.panes[0].sgr_pixel_mouse = true;
    state.surface(Arc::new(frame));
    let events = state.wheel(&target, -3, 1, 1).unwrap();
    assert_eq!(events.len(), 3);
    assert!(events.iter().all(|event| matches!(
        event,
        ClientPaneInputEvent::Mouse {
            kind: ClientMouseKind::ScrollDown,
            lines: 1,
            position: ClientMousePosition::Pixels { x: 8, y: 16, .. },
            ..
        }
    )));
    assert!(state.wheel(&target, i16::MIN, 0, 0).is_err());
}

#[test]
fn newer_metadata_keeps_the_last_coherent_pair_until_its_cells_arrive() {
    let mut state = Projection::new(geometry(3, 2, 8, 16).unwrap());
    let mut snap = snapshot();
    state.snapshot(Arc::new(snap.clone()));
    state.surface(Arc::new(surface()));
    snap.revision += 1;
    snap.workspaces[0].label = "new".into();
    state.snapshot(Arc::new(snap.clone()));
    let view = state.view();
    assert!(view.input_target.is_some());
    assert!(view.terminal.is_some());
    assert_eq!(view.workspaces[0].label, "repo");
    let mut frame = surface();
    frame.projection_revision = snap.revision;
    state.surface(Arc::new(frame));
    assert_eq!(state.view().workspaces[0].label, "new");
    snap.revision += 1;
    snap.focused_pane_id = Some("replacement".into());
    state.snapshot(Arc::new(snap));
    assert!(state.target().is_none());
    assert!(state.view().terminal.is_none());
}

#[test]
fn only_a_recently_used_client_reclaims_a_shared_tab_size() {
    use std::time::{Duration, Instant};
    let mut state = Projection::new(geometry(4, 2, 8, 16).unwrap());
    state.snapshot(Arc::new(snapshot()));
    state.surface(Arc::new(surface()));
    let now = Instant::now();
    assert!(!state.reassert_due(now));
    state.note_input(now);
    assert!(!state.reassert_due(now));
    assert!(state.reassert_due(now + Duration::from_secs(1)));
    assert!(!state.reassert_due(now + Duration::from_millis(1100)));
    assert!(!state.reassert_due(now + Duration::from_secs(5)));
}

#[test]
fn initial_window_focus_waits_for_herdr_surface_but_accepts_its_existing_size() {
    let mut state = Projection::new(geometry(4, 2, 8, 16).unwrap());
    state.snapshot(Arc::new(snapshot()));
    assert!(state.initial_focus().is_none());
    state.surface(Arc::new(surface()));
    assert_eq!(state.initial_focus(), Some("boot-v1"));
    state.reported_focus = true;
    assert!(state.initial_focus().is_none());
}

mod pane_map;

mod fence;
mod go_to;

mod park_peer;
mod resume;

mod idle_evidence;

mod direct_terminal;

mod cell_fields;
mod new_terminal;
mod pane_tools;
mod roster_observation;
