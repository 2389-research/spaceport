// ABOUTME: Exercises target and surface selection for the direct terminal.
// ABOUTME: Covers pane ownership and the native popup fallback.
use super::*;

#[test]
fn direct_terminal_uses_its_pty_size_and_keeps_popup_targeting() {
    let mut state = Projection::new(geometry(7, 4, 8, 16).unwrap());
    state.snapshot(Arc::new(snapshot()));
    let mut frame = surface();
    state.surface(Arc::new(frame.clone()));
    assert!(state.target().is_none());
    state.direct_attach = true;
    let view = state.view();
    assert!(view.terminal.is_none());
    assert_eq!(view.input_target, state.direct_target());
    assert_eq!(view.input_target.unwrap().pane_id, "w1:p1");
    frame.popup = Some(Box::new(ClientShellPopupSurface {
        terminal_id: "popup1".into(),
        title: String::new(),
        width: None,
        height: None,
        frame: cells("OK", 2),
        mouse_reporting: false,
        sgr_pixel_mouse: false,
        pixel_width: 16,
        pixel_height: 16,
    }));
    state.surface(Arc::new(frame));
    assert!(state.view().terminal.is_some());
    assert!(state.view().input_target.is_none());
    assert!(state.direct_target().is_none());
}

#[test]
fn a_same_pane_surface_gap_pauses_output_until_confirmed() {
    let mut state = Projection::new(geometry(3, 2, 8, 16).unwrap());
    let mut snap = snapshot();
    state.snapshot(Arc::new(snap.clone()));
    let mut frame = surface();
    state.surface(Arc::new(frame.clone()));
    let target = state.direct_target().unwrap();
    snap.revision += 1;
    snap.server_keybindings_toml = Some("updated".into());
    state.snapshot(Arc::new(snap.clone()));
    assert!(!state.can_read_direct(&target).unwrap());
    assert!(state.direct_target().is_none());
    frame.projection_revision = snap.revision;
    state.surface(Arc::new(frame));
    assert!(state.can_read_direct(&target).unwrap());
    snap.revision += 1;
    snap.focused_pane_id = Some("different".into());
    state.snapshot(Arc::new(snap));
    assert!(matches!(
        state.can_read_direct(&target),
        Err(MobileError::StaleView)
    ));
}
