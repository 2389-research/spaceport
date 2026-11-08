use super::*;

#[gpui::test]
fn alt_web_link_still_uses_system_handler_without_a_page_tab(cx: &mut gpui::TestAppContext) {
    use gpui::{point, px};

    let (view, cx) = cx.add_window_view(|window, cx| {
        let mut view = crate::sidebar::layout_tests::fixture_window(window, cx);
        let mut shown = surface("https://example.test/docs");
        let snapshot = view.live.snapshot.as_ref().unwrap();
        shown.boot_id = snapshot.boot_id.clone();
        shown.projection_revision = snapshot.revision;
        view.live.surface = Some(Arc::new(shown));
        view
    });
    cx.update(|window, cx| {
        window.refresh();
        window.draw(cx).clear(cx);
    });
    let link = view.read_with(cx, |view, _| view.bounds.origin + point(px(1.), px(1.)));
    let mut modifiers = crate::keymap::secondary_key();
    modifiers.alt = true;
    cx.simulate_click(link, modifiers);
    assert_eq!(
        cx.opened_url().as_deref(),
        Some("https://example.test/docs")
    );
    let pages = cx.update(|_, cx| {
        cx.try_global::<crate::browser::Store>()
            .map_or(0, |store| store.tabs().count())
    });
    assert_eq!(pages, 0);
}
