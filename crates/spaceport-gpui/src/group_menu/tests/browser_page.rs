use super::*;

#[gpui::test]
fn group_menu_keeps_terminal_creation_without_a_browser_page_row(cx: &mut TestAppContext) {
    let (view, cx) = window(cx);
    draw(cx);
    assert!(cx.debug_bounds("new-tab").is_some());
    let button = cx.debug_bounds("tab-actions").unwrap();
    cx.simulate_click(button.center(), Modifiers::none());
    draw(cx);
    assert!(view.read_with(cx, |view, _| view.menu.page == Some(Page::Group)));
    assert!(cx.debug_bounds("group-menu-NewBrowserTab").is_none());
    assert!(cx.debug_bounds("group-menu-Split").is_some());
}
