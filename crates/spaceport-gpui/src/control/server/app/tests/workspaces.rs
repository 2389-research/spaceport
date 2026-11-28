// ABOUTME: Exercises the window-scoped workspace disclosure through its wire request.
// ABOUTME: Checks repeated off requests and a toggle against the rendered list.
use super::*;
use crate::sidebar::layout_tests::{fixture_window, full_draw};

#[gpui::test]
fn workspace_disclosure_request_updates_the_rendered_list(cx: &mut gpui::TestAppContext) {
    let (view, cx) = cx.add_window_view(fixture_window);
    cx.update(|window, cx| full_draw(window, cx).clear(cx));
    let divider = cx.debug_bounds("sidebar-split-resize").unwrap();
    cx.simulate_mouse_down(
        divider.center(),
        gpui::MouseButton::Left,
        gpui::Modifiers::none(),
    );
    assert!(view.read_with(cx, |view, _| view.sidebar_drag.is_some()));
    for (switch, expanded) in [("off", false), ("off", false), ("toggle", true)] {
        cx.cx.update(|cx| {
            let op =
                serde_json::from_value(serde_json::json!({"op": "workspaces", "switch": switch}))
                    .unwrap();
            assert_eq!(code(&ui(call(op), cx)), None);
        });
        cx.update(|window, cx| full_draw(window, cx).clear(cx));
        assert!(view.read_with(cx, |view, _| view.sidebar_drag.is_none()));
        assert_eq!(cx.debug_bounds("spaces-scroll").is_some(), expanded);
    }
    view.update(cx, |view, _| {
        view.config.layout.mode = crate::config::LayoutMode::Devices
    });
    cx.cx.update(|cx| {
        assert_eq!(
            code(&ui(
                call(UiOp::Workspaces {
                    switch: Switch::Off
                }),
                cx
            )),
            Some(ErrorCode::Unsupported)
        );
    });
}
