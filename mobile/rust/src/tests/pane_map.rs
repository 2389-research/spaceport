// ABOUTME: Exercises the phone pane map against Herdr snapshot and surface records.
// ABOUTME: Pins asymmetric geometry, pane identity, and incomplete frame handling.
use super::*;

fn split() -> (ClientShellSnapshot, PaneSurfaceFrame) {
    let mut snapshot = snapshot();
    let mut frame = surface();
    frame.frame = cells("abcdefghijklmnopqrstuvwx", 8);
    frame.panes[0].rect = SurfaceRect {
        x: 0,
        y: 0,
        width: 5,
        height: 3,
    };
    let mut right = frame.panes[0].clone();
    right.pane_id = "w1:p2".into();
    right.rect = SurfaceRect {
        x: 5,
        y: 0,
        width: 3,
        height: 3,
    };
    right.inner_rect = right.rect;
    let mut second = snapshot.panes[0].clone();
    second.pane_id = right.pane_id.clone();
    let mut other_tab = second.clone();
    other_tab.pane_id = "w1:p3".into();
    other_tab.tab_id = "w1:t2".into();
    snapshot.panes.extend([second, other_tab]);
    frame.panes.insert(0, right);
    (snapshot, frame)
}

#[test]
fn pane_map_uses_outer_rectangles_from_the_matching_surface() {
    let (snapshot, frame) = split();
    let mut state = Projection::new(geometry(8, 3, 8, 16).unwrap());
    state.snapshot(Arc::new(snapshot));
    let mut stale = frame.clone();
    stale.projection_revision += 1;
    state.surface(Arc::new(stale));
    assert!(state.view().pane_map.is_none());
    state.surface(Arc::new(frame));
    let map = state.view().pane_map.unwrap();
    assert_eq!((map.columns, map.rows), (8, 3));
    let geometry: Vec<_> = map
        .panes
        .iter()
        .map(|p| (p.id.as_str(), p.x, p.y, p.width, p.height))
        .collect();
    assert_eq!(geometry, [("w1:p2", 5, 0, 3, 3), ("w1:p1", 0, 0, 5, 3)]);
}

#[test]
fn pane_map_requires_complete_valid_geometry_for_the_split_tab() {
    let (snapshot, frame) = split();
    let mut state = Projection::new(geometry(8, 3, 8, 16).unwrap());
    state.snapshot(Arc::new(snapshot));
    for edit in [
        |f: &mut PaneSurfaceFrame| {
            f.panes.pop();
        },
        |f: &mut PaneSurfaceFrame| f.panes[0].pane_id = "w1:p1".into(),
        |f: &mut PaneSurfaceFrame| f.panes[0].pane_id = "w1:p3".into(),
        |f: &mut PaneSurfaceFrame| f.panes[0].rect.width = 0,
        |f: &mut PaneSurfaceFrame| f.panes[0].rect.y = u16::MAX,
        |f: &mut PaneSurfaceFrame| f.panes[0].rect.x = 7,
        |f: &mut PaneSurfaceFrame| f.panes[0].rect.x = 4,
        // A rectangle that leaves a column of the frame to no pane.
        |f: &mut PaneSurfaceFrame| f.panes[0].rect.width = 2,
    ] {
        let mut bad = frame.clone();
        edit(&mut bad);
        state.surface(Arc::new(bad));
        assert!(state.view().pane_map.is_none());
        assert_eq!(state.view().panes.len(), 3);
    }
}

#[test]
fn pane_map_rejects_ambiguous_snapshot_ids() {
    let (mut snapshot, frame) = split();
    snapshot.panes.push(snapshot.panes[0].clone());
    let mut state = Projection::new(geometry(8, 3, 8, 16).unwrap());
    state.snapshot(Arc::new(snapshot));
    state.surface(Arc::new(frame));
    assert!(state.view().pane_map.is_none());
}
