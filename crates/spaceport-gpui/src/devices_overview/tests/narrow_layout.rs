use super::*;
use gpui::{Bounds, Pixels, px, size};

fn bounds(cx: &mut VisualTestContext, selector: String) -> Bounds<Pixels> {
    cx.debug_bounds(Box::leak(selector.into_boxed_str()))
        .unwrap()
}

fn assert_devices_fit(cx: &mut VisualTestContext, prefix: &str) {
    let tab = bounds(cx, format!("{prefix}devices-tab"));
    let table = bounds(cx, format!("{prefix}devices-table"));
    let row = bounds(cx, format!("{prefix}devices-row-local"));
    assert!(table.left() >= tab.left() && table.right() <= tab.right());
    assert!(row.left() >= table.left() && row.right() <= table.right());
    assert!(
        cx.debug_bounds(Box::leak(
            format!("{prefix}devices-load-local").into_boxed_str()
        ))
        .is_none()
    );
}

#[gpui::test]
fn devices_fit_the_minimum_window_width(cx: &mut gpui::TestAppContext) {
    let (view, cx) = window(cx);
    cx.simulate_resize(size(px(640.), px(900.)));
    open(&view, cx);
    assert_devices_fit(cx, "");
}

#[gpui::test]
fn devices_fit_a_narrow_editor_group(cx: &mut gpui::TestAppContext) {
    let (view, cx) = window(cx);
    cx.simulate_resize(size(px(1000.), px(900.)));
    open(&view, cx);
    cx.update(|window, cx| {
        view.update(cx, |view, cx| {
            view.command(Command::SplitEditor, window, cx);
            // The new group opens without tabs; the overview moves to it.
            let slots = view.group_slots();
            let Some(crate::browser::Pick::Page(overview)) = view.group_pick(slots[0].id) else {
                panic!("the overview is shown")
            };
            view.show_browser_tab_in(Some(slots[1].id), overview, window, cx);
        })
    });
    draw(cx);
    assert_devices_fit(cx, "g1-");
}
