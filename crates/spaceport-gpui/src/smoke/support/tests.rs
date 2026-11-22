// ABOUTME: Shared fixtures for the scaffolding's own tests.
// ABOUTME: Builds the live state a native handoff diagnostic is read from.
#![allow(clippy::unwrap_used)]
use crate::{LiveState, sidebar};
use spaceport_client::protocol::{CellData, CursorState, FrameData, PaneSurfaceFrame};
use std::sync::Arc;

mod handoff;
mod shell_ready;

/// A connected window showing one 80-column row that ends at `prompt`.
fn live(prompt: &str) -> LiveState {
    let mut snapshot = sidebar::layout_tests::snapshot(1);
    snapshot.focused_workspace_id = Some("w0".into());
    snapshot.focused_tab_id = Some("t0".into());
    let width = 80;
    let mut live = LiveState::default();
    live.surface = Some(Arc::new(PaneSurfaceFrame {
        boot_id: snapshot.boot_id.clone(),
        projection_revision: snapshot.revision,
        surface_revision: 1,
        frame: FrameData {
            width,
            height: 1,
            cells: format!("{prompt:<width$}", width = usize::from(width))
                .chars()
                .map(|ch| CellData {
                    symbol: ch.to_string(),
                    skip: false,
                    fg: 0,
                    bg: 0,
                    modifier: 0,
                    hyperlink: None,
                })
                .collect(),
            cursor: Some(CursorState {
                x: prompt.len().try_into().unwrap(),
                y: 0,
                visible: true,
                shape: 0,
            }),
            hyperlinks: vec![],
            graphics: vec![],
        },
        panes: vec![],
        splits: vec![],
        popup: None,
        graphics: Default::default(),
    }));
    live.snapshot = Some(Arc::new(snapshot));
    live
}
