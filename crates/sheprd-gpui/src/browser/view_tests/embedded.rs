use super::*;
use crate::controls::Command;

/// How many browser tabs the app holds; test IDs start at zero.
fn tab_count(cx: &mut VisualTestContext) -> usize {
    cx.update(|_, cx| {
        cx.try_global::<Store>().map_or(0, |store| {
            (0..64)
                .filter(|id| store.get(crate::browser::TabId::test(*id)).is_some())
                .count()
        })
    })
}

#[gpui::test]
fn a_browser_tab_covers_the_terminal_until_it_closes(cx: &mut gpui::TestAppContext) {
    let (view, cx) = window(cx);
    draw(cx);
    assert!(cx.debug_bounds("terminal").is_some());
    assert!(cx.debug_bounds("browser").is_none());

    // A blank tab needs no page, so this runs without a native web view.
    cx.update(|window, cx| {
        view.update(cx, |view, cx| view.open_browser_tab(None, window, cx));
    });
    draw(cx);
    assert!(cx.debug_bounds("browser-tab-0").is_some());
    assert!(cx.debug_bounds("browser").is_some());
    assert!(cx.debug_bounds("browser-address").is_some());
    assert!(cx.debug_bounds("browser-placeholder").is_some());
    assert!(
        cx.debug_bounds("terminal").is_none(),
        "the page replaces it"
    );

    // Clicking a Herdr tab brings its terminal back; the browser tab stays.
    let herdr_tab = cx.debug_bounds("tab-t0").unwrap();
    cx.simulate_click(herdr_tab.center(), gpui::Modifiers::none());
    draw(cx);
    assert!(cx.debug_bounds("terminal").is_some());
    assert!(cx.debug_bounds("browser-tab-0").is_some());

    // Close Tab closes the page rather than asking about the Herdr tab.
    let browser_tab = cx.debug_bounds("browser-tab-0").unwrap();
    cx.simulate_click(browser_tab.center(), gpui::Modifiers::none());
    draw(cx);
    assert!(cx.debug_bounds("browser").is_some());
    cx.update(|window, cx| {
        view.update(cx, |view, cx| view.command(Command::CloseTab, window, cx));
    });
    draw(cx);
    assert!(cx.debug_bounds("browser-tab-0").is_none());
    assert!(cx.debug_bounds("terminal").is_some());
    view.read_with(cx, |view, _| assert!(view.menu.page.is_none()));
    assert_eq!(tab_count(cx), 0);
}
