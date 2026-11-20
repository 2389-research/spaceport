// ABOUTME: Reads projected Core state for the explicitly enabled private profile observer.
// ABOUTME: Keeps projected content separate from capture-based visible-content qualification.
use crate::{
    SheprdWindow,
    browser::{Location, Store, TabId},
    working_profile::report::{Core, CoreProjection, marker_visible},
};
use gpui::App;

impl SheprdWindow {
    pub(crate) fn profile_core_tab(&self, tab: TabId, cx: &App) -> Option<Core> {
        let saved = cx.try_global::<Store>()?.get(tab)?;
        let Some(Location::CoreTerminal { panes, .. }) = &saved.location else {
            return None;
        };
        // The profile fixture supports one pane without changing product splits.
        if panes.len() != 1 {
            return None;
        }
        let terminal = panes.first();
        let view = self
            .core_terminals
            .views
            .get(&(tab, terminal.clone()))?
            .read(cx);
        let hub = &self.core_terminals.hub;
        let backend = hub.backend();
        let connected = match saved.core_group() {
            Some(key) => hub.backend_for(key.hub_core_id()).is_ok(),
            None => backend.is_some_and(|backend| !backend.core().is_lost()),
        };
        Some(Core {
            tab: tab.into(),
            terminal: view.id.to_string(),
            core_id: backend.map(|backend| backend.core().hello().core.id.clone()),
            connected,
            observe: view.observe,
            ended: view.ended(),
            failed: view.screen.failure.is_some(),
            last_layout: view.bounds().map(|bounds| {
                [
                    f32::from(bounds.left()),
                    f32::from(bounds.top()),
                    f32::from(bounds.size.width),
                    f32::from(bounds.size.height),
                ]
            }),
            projected: view.screen.frame.as_deref().map(|frame| CoreProjection {
                columns: frame.width,
                rows: frame.height,
                nonblank_cells: frame
                    .cells
                    .iter()
                    .filter(|cell| !cell.symbol.trim().is_empty())
                    .count(),
                projected_marker: marker_visible(frame),
            }),
        })
    }
}

#[cfg(test)]
mod tests;
