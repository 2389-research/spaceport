//! A control request finds a window only where that window shows its daemon.
use super::*;
use crate::sidebar::layout_tests::{fixture_window, snapshot};
use gpui::{Entity, VisualTestContext};
use std::sync::Arc;

/// The fixture window, showing workspace `w0` as a connected daemon would.
fn window(cx: &mut gpui::TestAppContext) -> (Entity<SpaceportWindow>, &mut VisualTestContext) {
    cx.add_window_view(|window, cx| {
        let mut view = fixture_window(window, cx);
        let mut shown = snapshot(40);
        shown.focused_workspace_id = Some("w0".into());
        shown.focused_tab_id = Some("t0".into());
        view.live.snapshot = Some(Arc::new(shown));
        view
    })
}

#[gpui::test]
fn control_target_refuses_unrelated_daemon(cx: &mut gpui::TestAppContext) {
    let (view, cx) = window(cx);
    view.read_with(cx, |view, _| {
        let target = Target {
            daemon: Some(std::path::Path::new("/elsewhere/herdr-client.sock")),
            workspace: Some("w0"),
        };
        assert!(view.control_target(&target, true).is_none());
    });
}
