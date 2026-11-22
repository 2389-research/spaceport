use super::*;

#[gpui::test]
fn service_page_uses_system_handler_without_replacing_a_terminal(cx: &mut gpui::TestAppContext) {
    let (view, cx) = window(cx);
    draw(cx);
    assert!(cx.debug_bounds("terminal").is_some());
    let (endpoint, scope) = view.read_with(cx, |view, _| {
        (view.endpoints[0].id.clone(), scope(&view.endpoints[0]))
    });
    let before =
        cx.update(|_, cx| Store::update(cx, |store| store.in_workspace(&scope, "w0").count()));
    let url = WebUrl::try_from("https://example.test/dashboard?view=1").unwrap();
    let opened = cx.update(|window, cx| {
        view.update(cx, |view, cx| {
            view.open_workspace_page(&endpoint, "w0", url, window, cx)
        })
    });
    assert_eq!(opened, crate::browser::PageOpened::SystemBrowser);
    assert_eq!(
        cx.opened_url().as_deref(),
        Some("https://example.test/dashboard?view=1")
    );
    let after =
        cx.update(|_, cx| Store::update(cx, |store| store.in_workspace(&scope, "w0").count()));
    assert_eq!(after, before);
    draw(cx);
    assert!(cx.debug_bounds("terminal").is_some());
}

#[gpui::test]
fn service_page_refuses_an_endpoint_outside_this_window(cx: &mut gpui::TestAppContext) {
    let (view, cx) = window(cx);
    let url = WebUrl::try_from("https://example.test/private").unwrap();
    let opened = cx.update(|window, cx| {
        view.update(cx, |view, cx| {
            view.open_workspace_page("other-endpoint", "w0", url, window, cx)
        })
    });
    assert_eq!(opened, crate::browser::PageOpened::Unreachable);
    assert!(cx.opened_url().is_none());
}
