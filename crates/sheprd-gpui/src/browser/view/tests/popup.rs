use super::*;
use crate::sidebar::layout_tests::{fixture_window, snapshot};
use std::sync::Arc;

#[gpui::test]
fn page_new_window_does_not_open_a_valid_https_url_or_create_a_tab(cx: &mut gpui::TestAppContext) {
    let (view, cx) = cx.add_window_view(|window, cx| {
        let mut view = fixture_window(window, cx);
        let mut shown = snapshot(40);
        shown.focused_workspace_id = Some("w0".into());
        shown.focused_tab_id = Some("t0".into());
        view.live.snapshot = Some(Arc::new(shown));
        view
    });
    let opener = cx.update(|_, cx| {
        let scope = scope(&view.read(cx).endpoints[0]);
        Store::update(cx, |store| {
            store
                .open(
                    scope,
                    "w0",
                    Some(Location::Web {
                        url: WebUrl::try_from("https://opener.test/").unwrap(),
                    }),
                    None,
                )
                .unwrap()
        })
    });
    let before = cx.update(|_, cx| cx.global::<Store>().tabs().count());
    let response = cx.update(|window, cx| {
        view.update(cx, |view, cx| {
            let response = view
                .browser
                .pages
                .request_new_window_for_test("https://example.test/login?state=kept#step");
            view.apply_page_events(window, cx);
            response
        })
    });
    assert!(matches!(response, wry::NewWindowResponse::Deny));
    assert!(cx.opened_url().is_none());
    assert_eq!(
        cx.update(|_, cx| cx.global::<Store>().tabs().count()),
        before
    );
    assert!(cx.update(|_, cx| cx.global::<Store>().get(opener).is_some()));
}
