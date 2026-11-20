// ABOUTME: Reads terminal group presentation for the opt-in native profile observer.
// ABOUTME: Borrows paired presented assets and keeps latest projections explicitly separate.
use super::*;
use crate::working_profile::report::{Frame, Group, marker_visible};

impl SheprdWindow {
    pub(crate) fn working_profile_groups(&self, cx: &App) -> Vec<Group> {
        let bounds = self.browser.group_bounds.borrow();
        let primary_group = self.primary_group();
        self.drawn_group_slots()
            .into_iter()
            .map(|slot| {
                let primary = Some(slot.id) == primary_group;
                let state = if primary {
                    Some((&self.live, &self.presentation))
                } else {
                    self.group_terminals
                        .parked
                        .iter()
                        .find(|parked| parked.group == slot.id)
                        .map(|parked| (&parked.live, &parked.presentation))
                };
                let presented = state.and_then(|(live, presentation)| {
                    let (surface, images) = presentation.profile_presented()?;
                    let frame = &surface.frame;
                    let snapshot = live.snapshot.as_deref();
                    Some(Frame {
                        boot: surface.boot_id.clone(),
                        projection_revision: surface.projection_revision,
                        surface_revision: surface.surface_revision,
                        columns: frame.width,
                        rows: frame.height,
                        pane_ids: surface
                            .panes
                            .iter()
                            .map(|pane| pane.pane_id.clone())
                            .collect(),
                        nonblank_cells: frame
                            .cells
                            .iter()
                            .filter(|cell| !cell.symbol.trim().is_empty())
                            .count(),
                        paired_image_assets: images.len(),
                        marker_visible: marker_visible(frame),
                        current: !presentation.stale()
                            && snapshot.is_some_and(|snapshot| {
                                snapshot.boot_id == surface.boot_id
                                    && snapshot.revision == surface.projection_revision
                            }),
                    })
                });
                Group {
                    core: match self.group_shown(slot.id, cx) {
                        Shown::Page(tab) => self.profile_core_tab(tab, cx),
                        _ => None,
                    },
                    index: slot.index,
                    primary,
                    shown_terminal: self.group_shown(slot.id, cx) == Shown::Terminal,
                    connected: state.is_some_and(|(live, _)| live.status.is_connected()),
                    bounds: bounds.get(&slot.id).map_or([0.; 4], |bounds| {
                        [
                            f32::from(bounds.left()),
                            f32::from(bounds.top()),
                            f32::from(bounds.size.width),
                            f32::from(bounds.size.height),
                        ]
                    }),
                    tab: state.and_then(|(live, _)| live.snapshot.as_ref()?.focused_tab_id.clone()),
                    projected_revision: state.and_then(|(live, _)| {
                        live.surface
                            .as_ref()
                            .map(|surface| surface.projection_revision)
                    }),
                    presented,
                }
            })
            .collect()
    }
}
