// ABOUTME: Projects the focused tab's outer pane rectangles for spatial navigation.
// ABOUTME: Omits partial or ambiguous maps while leaving pane list navigation available.
use spaceport_backend::TerminalId;
use spaceport_client::protocol::{ClientShellSnapshot, PaneSurfaceFrame};
use spaceport_panes::{CellMap, CellRect};
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
    let columns = surface.frame.width;
    let rows = surface.frame.height;
    if surface.panes.len() != expected.len()
        || usize::from(columns) * usize::from(rows) != surface.frame.cells.len()
    {
        return None;
    }
    let mut rects = Vec::with_capacity(surface.panes.len());
    for pane in &surface.panes {
        let id = TerminalId::try_from(pane.pane_id.clone()).ok()?;
        if !expected.remove(id.as_str()) {
            return None;
        }
        rects.push((
            id,
            CellRect {
                x: pane.rect.x,
                y: pane.rect.y,
                width: pane.rect.width,
                height: pane.rect.height,
            },
        ));
    }
    // The daemon's rectangles must tile the frame exactly once, which is the
    // rule every client reads its pane layout through.
    let map = CellMap::new(columns, rows, rects)?;
    Some(MobilePaneMap {
        columns: map.columns(),
        rows: map.rows(),
        panes: map
            .panes()
            .iter()
            .map(|(id, rect)| MobilePaneRect {
                id: String::from(id.clone()),
                x: rect.x,
                y: rect.y,
                width: rect.width,
                height: rect.height,
            })
            .collect(),
    })
}
