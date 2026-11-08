use super::*;
use crate::control::Target;

#[gpui::test]
fn control_target_refuses_unrelated_daemon(cx: &mut gpui::TestAppContext) {
    let (view, cx) = window(cx);
    view.read_with(cx, |view, _| {
        let target = Target {
            daemon: Some(std::path::Path::new("/elsewhere/herdr-client.sock")),
            workspace: Some("w0"),
            pane: None,
        };
        assert!(view.control_target(&target, true).is_none());
    });
}
