// ABOUTME: Exercises navigation across Herdr workspaces with delayed receipts and surfaces.
// ABOUTME: Checks that obsolete destinations cannot move focus or receive terminal input.
use super::*;

#[test]
fn a_single_pane_focus_rejects_a_pane_outside_the_selected_tab() {
    let mut snap = snapshot();
    snap.panes[0].tab_id = "other-tab".into();
    assert!(!OwnedNavigationTarget::Pane("w1:p1".into()).reachable_in(&snap));
}

fn route_fixture() -> (Projection, ClientShellSnapshot, PaneSurfaceFrame) {
    let mut state = Projection::new(geometry(3, 2, 8, 16).unwrap());
    let mut snap = snapshot();
    let mut ws = snap.workspaces[0].clone();
    ws.workspace_id = "infra".into();
    snap.workspaces.push(ws);
    let mut tab = snap.tabs[0].clone();
    tab.workspace_id = "infra".into();
    tab.tab_id = "deploy".into();
    snap.tabs.push(tab);
    let mut pane = snap.panes[0].clone();
    pane.workspace_id = "infra".into();
    pane.tab_id = "deploy".into();
    pane.pane_id = "watch".into();
    snap.panes.push(pane);
    let frame = surface();
    state.snapshot(Arc::new(snap.clone()));
    state.surface(Arc::new(frame.clone()));
    state
        .begin_go_to(
            "boot-v1",
            "infra".into(),
            "deploy".into(),
            "watch".into(),
            None,
        )
        .unwrap();
    (state, snap, frame)
}

fn acknowledge(state: &mut Projection, min: u64) {
    state.receipt("choose", &serde_json::json!({"result": {}}));
    state.receipt("barrier", &serde_json::json!({"result": {"type":"client_shell_surface_set","active":true,"projection_revision":min}}));
}

#[test]
fn a_jump_fences_every_step_and_only_exposes_the_final_input_target() {
    let (mut state, mut snap, mut frame) = route_fixture();
    let request_version = state.version;
    let steps = [
        OwnedNavigationTarget::Workspace("infra".into()),
        OwnedNavigationTarget::Tab("deploy".into()),
        OwnedNavigationTarget::Pane("watch".into()),
    ];
    for step in steps {
        let issued = state.go_to_step().unwrap().unwrap();
        assert_eq!(issued, step);
        assert!(state.target().is_none());
        state.start_fence("choose".into(), issued, Ok("barrier".into()));
        snap.revision += 1;
        frame.projection_revision = snap.revision;
        match &step {
            OwnedNavigationTarget::Workspace(id) => snap.focused_workspace_id = Some(id.clone()),
            OwnedNavigationTarget::Tab(id) => snap.focused_tab_id = Some(id.clone()),
            OwnedNavigationTarget::Pane(id) => {
                snap.focused_pane_id = Some(id.clone());
                frame.panes[0].pane_id = id.clone();
            }
        }
        state.snapshot(Arc::new(snap.clone()));
        state.surface(Arc::new(frame.clone()));
        assert!(state.go_to_step().unwrap().is_none());
        assert!(state.target().is_none());
        acknowledge(&mut state, snap.revision);
        assert!(state.target().is_none());
    }
    assert!(state.go_to_step().unwrap().is_none());
    assert!(!state.view().go_to_pending);
    assert_eq!(state.view().go_to_arrival, Some(request_version));
    assert_eq!(state.target().unwrap().pane_id, "watch");
}

#[test]
fn a_moved_destination_or_canceled_jump_never_issues_another_step() {
    let (mut state, mut snap, _) = route_fixture();
    snap.revision += 1;
    snap.panes[1].tab_id = "another-tab".into();
    state.snapshot(Arc::new(snap));
    assert!(state.go_to_step().is_err());
    let (mut state, _, _) = route_fixture();
    state.cancel_go_to();
    assert!(state.go_to_step().unwrap().is_none());
}

#[test]
fn a_boot_change_or_rejected_step_stops_a_jump() {
    let (mut state, mut snap, _) = route_fixture();
    let step = state.go_to_step().unwrap().unwrap();
    state.start_fence("choose".into(), step, Ok("barrier".into()));
    snap.boot_id = "new-boot".into();
    state.snapshot(Arc::new(snap));
    assert!(state.disconnected);
    assert!(state.target().is_none());

    let (mut state, _, _) = route_fixture();
    let step = state.go_to_step().unwrap().unwrap();
    state.start_fence("choose".into(), step, Ok("barrier".into()));
    state.rejected(Some("choose"), "gone".into());
    assert!(!state.view().go_to_pending);
    assert!(state.go_to_step().unwrap().is_none());
    assert!(state.target().is_none());
}

#[test]
fn go_to_waits_for_the_surface_to_identify_the_same_focused_pane() {
    let (mut state, mut snap, mut frame) = route_fixture();
    snap.revision += 1;
    snap.focused_workspace_id = Some("infra".into());
    snap.focused_tab_id = Some("deploy".into());
    snap.focused_pane_id = Some("watch".into());
    frame.projection_revision = snap.revision;
    frame.panes[0].pane_id = "watch".into();
    frame.panes[0].focused = false;
    state.snapshot(Arc::new(snap));
    state.surface(Arc::new(frame.clone()));
    assert!(state.go_to_step().unwrap().is_none());
    assert!(state.view().go_to_pending);
    assert!(state.target().is_none());
    frame.panes[0].focused = true;
    state.surface(Arc::new(frame));
    assert!(state.go_to_step().unwrap().is_none());
    assert!(!state.view().go_to_pending);
}

#[test]
fn selector_titles_follow_the_reported_agent_and_clear_without_a_stripped_title() {
    let (mut state, mut snap, mut frame) = route_fixture();
    state.cancel_go_to();
    snap.revision += 1;
    snap.agents[0].terminal_title = Some("✳ Review checklist".into());
    snap.agents[0].terminal_title_stripped = Some("Review\n\u{202e}checklist".into());
    frame.projection_revision = snap.revision;
    state.snapshot(Arc::new(snap.clone()));
    state.surface(Arc::new(frame.clone()));
    let view = state.view();
    assert_eq!(view.panes[0].agent_name.as_deref(), Some("Claude"));
    assert_eq!(
        view.panes[0].terminal_title.as_deref(),
        Some("Review checklist")
    );
    assert!(view.panes[1].terminal_title.is_none());
    snap.revision += 1;
    snap.agents[0].terminal_title_stripped = None;
    frame.projection_revision = snap.revision;
    state.snapshot(Arc::new(snap));
    state.surface(Arc::new(frame));
    assert!(state.view().panes[0].terminal_title.is_none());
}

#[test]
fn agents_come_from_the_live_agent_table_and_routes_recheck_the_report() {
    let (mut state, mut snap, _) = route_fixture();
    state.cancel_go_to();
    let view = state.view();
    assert_eq!(view.panes[0].agent_name.as_deref(), Some("Claude"));
    assert!(view.panes[1].agent_name.is_none());
    assert_eq!(
        view.panes[0].state_change_seq,
        Some(snap.agents[0].state_change_seq)
    );
    assert_eq!(view.panes[1].state_change_seq, None);
    state
        .begin_go_to(
            "boot-v1",
            "w1".into(),
            "w1:t1".into(),
            "w1:p1".into(),
            Some("Claude".into()),
        )
        .unwrap();
    snap.revision += 1;
    snap.agents.clear();
    state.snapshot(Arc::new(snap));
    assert!(matches!(state.go_to_step(), Err(MobileError::StaleView)));
}

#[test]
fn canceled_agent_route_cannot_claim_arrival_from_an_older_coherent_surface() {
    let (mut state, mut snap, _) = route_fixture();
    state.cancel_go_to();
    let begin = |state: &mut Projection| {
        state
            .begin_go_to(
                "boot-v1",
                "w1".into(),
                "w1:t1".into(),
                "w1:p1".into(),
                Some("Claude".into()),
            )
            .unwrap()
    };
    let first = begin(&mut state);
    assert!(state.go_to_step().unwrap().is_none());
    assert_eq!(state.view().go_to_arrival, Some(first));
    let second = begin(&mut state);
    assert!(second > first);
    assert_eq!(state.view().go_to_arrival, None);

    snap.revision += 1;
    snap.agents.clear();
    state.snapshot(Arc::new(snap));
    assert!(matches!(state.go_to_step(), Err(MobileError::StaleView)));
    state.cancel_go_to();
    let view = state.view();
    assert!(view.version >= second && !view.go_to_pending);
    assert_eq!(view.input_target.unwrap().pane_id, "w1:p1");
    assert_eq!(view.panes[0].agent_name.as_deref(), Some("Claude"));
    assert_eq!(view.go_to_arrival, None);
}
