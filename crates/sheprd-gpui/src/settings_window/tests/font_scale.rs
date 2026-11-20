// ABOUTME: Checks that Settings follows its configured interface font size.
// ABOUTME: Uses actual row bounds to catch clipped or overlapping labels.
use super::*;
use gpui::{px, size};

#[gpui::test]
fn settings_text_grows_with_the_ui_font(cx: &mut TestAppContext) {
    let main = cx.add_window(crate::sidebar::layout_tests::fixture_window);
    let weak = cx.update(|cx| main.update(cx, |_, _, cx| cx.weak_entity()).unwrap());
    let (view, cx) = cx.add_window_view(|_, cx| {
        let mut view = fixture_new(weak, cx);
        view.section = Section::General;
        view
    });
    cx.simulate_resize(size(px(1100.), px(900.)));
    let draw = |cx: &mut VisualTestContext| {
        cx.update(|window, cx| crate::sidebar::layout_tests::full_draw(window, cx).clear(cx));
    };
    draw(cx);
    let before = cx.debug_bounds("general-ports-title").unwrap();
    view.update(cx, |view, cx| {
        view.config.ui.size = 24.;
        cx.notify();
    });
    draw(cx);
    let after = cx.debug_bounds("general-ports-title").unwrap();
    assert!(
        after.size.height >= before.size.height * 1.8,
        "{before:?} -> {after:?}"
    );
    let row = cx.debug_bounds("general-ports").unwrap();
    assert!(after.bottom() <= row.bottom(), "{after:?} outside {row:?}");
    cx.simulate_resize(size(px(640.), px(900.)));
    draw(cx);
    let narrow = cx.debug_bounds("general-ports").unwrap();
    for selector in ["general-ports-title", "general-ports-description"] {
        let text = cx.debug_bounds(selector).unwrap();
        assert!(
            text.right() <= narrow.right() && text.bottom() <= narrow.bottom(),
            "{selector}: {text:?} outside {narrow:?}"
        );
    }
    view.update(cx, |view, cx| {
        view.config.ui.size = 12.;
        cx.notify();
    });
    draw(cx);
    assert_eq!(
        cx.debug_bounds("general-ports-title").unwrap().size.height,
        before.size.height
    );
}
