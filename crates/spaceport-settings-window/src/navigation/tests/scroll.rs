//! A window too short for every section scrolls the list instead of clipping
//! the last ones, and every section stays reachable.

use super::*;
use gpui::size;

#[gpui::test]
fn a_short_window_scrolls_the_section_list(cx: &mut TestAppContext) {
    let (view, cx) = cx.add_window_view(|_, _| Harness::new());
    cx.simulate_resize(size(px(680.), px(480.)));
    draw(cx);
    let last = Section::ALL.len() - 1;
    let id = leak(Section::ALL[last].selector());
    let list = bounds(cx, "settings-sections");
    let row = bounds(cx, id);
    assert!(list.bottom() <= px(480.), "{list:?}");
    assert!(row.bottom() > list.bottom(), "the fixture must overflow");

    view.update(cx, |harness, cx| {
        harness.scroll.scroll_to_item(Section::ALL[last].nav_item());
        cx.notify();
    });
    draw(cx);
    let list = bounds(cx, "settings-sections");
    let row = bounds(cx, id);
    assert!(
        row.top() >= list.top() && row.bottom() <= list.bottom() + px(1.),
        "{row:?} in {list:?}"
    );
    cx.simulate_click(row.center(), Default::default());
    cx.run_until_parked();
    view.read_with(cx, |harness, _| {
        assert_eq!(harness.selected, Section::ALL[last])
    });
}

#[gpui::test]
fn the_sections_thumb_follows_the_list_and_drags_it(cx: &mut TestAppContext) {
    let (view, cx) = cx.add_window_view(|_, _| Harness::new());
    cx.simulate_resize(size(px(680.), px(480.)));
    draw(cx);
    draw(cx);
    let thumb = bounds(cx, "settings-sections-thumb");
    let list = bounds(cx, "settings-sections");
    assert!(thumb.size.height < list.size.height, "{thumb:?}");
    let to = thumb.center() + point(px(0.), px(40.));
    cx.simulate_mouse_down(thumb.center(), MouseButton::Left, Default::default());
    cx.simulate_mouse_move(to, Some(MouseButton::Left), Default::default());
    cx.simulate_mouse_move(to, Some(MouseButton::Left), Default::default());
    cx.simulate_mouse_up(to, MouseButton::Left, Default::default());
    draw(cx);
    let scrolled = view.read_with(cx, |harness, _| harness.scroll.offset().y);
    assert!(scrolled < px(0.), "the list scrolled: {scrolled:?}");
}
