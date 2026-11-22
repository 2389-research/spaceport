// ABOUTME: Projects the focused tab's outer pane rectangles for spatial navigation.
// ABOUTME: Omits partial or ambiguous maps while leaving pane list navigation available.
use spaceport_client::protocol::{ClientShellSnapshot, PaneSurfaceFrame};
use std::collections::HashSet;

#[derive(uniffi::Record)]
pub struct MobilePaneMap {
    pub columns: u16,
    pub rows: u16,
    pub panes: Vec<MobilePaneRect>,
}

#[derive(uniffi::Record)]
pub struct MobilePaneRect {
    pub id: String,
    pub x: u16,
    pub y: u16,
    pub width: u16,
    pub height: u16,
}

pub(super) fn pane_map(
    snapshot: &ClientShellSnapshot,
    surface: &PaneSurfaceFrame,
) -> Option<MobilePaneMap> {
    if surface.panes.len() < 2 {
        return None;
    }
    let tab = snapshot.focused_tab_id.as_ref()?;
    let mut expected = HashSet::new();
    for pane in snapshot.panes.iter().filter(|pane| &pane.tab_id == tab) {
        if !expected.insert(pane.pane_id.as_str()) {
            return None;
        }
    }
    if surface.panes.len() != expected.len() {
        return None;
    }
    let columns = surface.frame.width;
    let rows = surface.frame.height;
    let stride = usize::from(columns);
    let area = stride * usize::from(rows);
    if area != surface.frame.cells.len() {
        return None;
    }
    let mut occupied = vec![false; area];
    let mut panes = Vec::with_capacity(expected.len());
    for pane in &surface.panes {
        let rect = pane.rect;
        let right = rect.x.checked_add(rect.width)?;
        let bottom = rect.y.checked_add(rect.height)?;
        if !expected.remove(pane.pane_id.as_str())
            || rect.width == 0
            || rect.height == 0
            || right > columns
            || bottom > rows
        {
            return None;
        }
        for row in rect.y..bottom {
            let start = usize::from(row) * stride;
            let span = &mut occupied[start + usize::from(rect.x)..start + usize::from(right)];
            if span.contains(&true) {
                return None;
            }
            span.fill(true);
        }
        panes.push(MobilePaneRect {
            id: pane.pane_id.clone(),
            x: rect.x,
            y: rect.y,
            width: rect.width,
            height: rect.height,
        });
    }
    Some(MobilePaneMap {
        columns,
        rows,
        panes,
    })
}
