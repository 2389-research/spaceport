// ABOUTME: Defines the presented-surface boundary for reporting pane output progress.
// ABOUTME: Keeps progress unavailable until the headless observation test reaches its assertion.
use super::PaneOutput;
use sheprd_client::protocol::PaneSurfaceFrame;

pub(crate) fn observe(_surface: &PaneSurfaceFrame) -> Option<Vec<PaneOutput>> {
    None
}
