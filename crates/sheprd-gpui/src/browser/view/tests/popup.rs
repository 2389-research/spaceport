use super::*;
use crate::sidebar::layout_tests::{fixture_window, snapshot};
use std::sync::Arc;

#[gpui::test]
fn page_new_window_opens_exact_url_without_creating_a_tab(cx: &mut gpui::TestAppContext) {
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
    let url = WebUrl::try_from("https://example.test/login?state=kept#step").unwrap();
    cx.update(|_, cx| {
        view.update(cx, |view, cx| view.open_page_externally(opener, url, cx));
    });
    assert_eq!(
        cx.opened_url().as_deref(),
        Some("https://example.test/login?state=kept#step")
    );
    assert_eq!(
        cx.update(|_, cx| cx.global::<Store>().tabs().count()),
        before
    );
}

#[gpui::test]
fn page_new_window_ignores_a_closed_opener(cx: &mut gpui::TestAppContext) {
    let (view, cx) = cx.add_window_view(|window, cx| fixture_window(window, cx));
    let opener = cx.update(|_, cx| {
        Store::update(cx, |store| {
            let id = store
                .open(
                    crate::browser::Scope::Legacy("closed".into()),
                    "w0",
                    Some(Location::Web {
                        url: WebUrl::try_from("https://opener.test/").unwrap(),
                    }),
                    None,
                )
                .unwrap();
            store.close(id);
            id
        })
    });
    let url = WebUrl::try_from("https://example.test/login").unwrap();
    cx.update(|_, cx| {
        view.update(cx, |view, cx| {
            view.open_page_externally(opener, url, cx);
        });
    });
    assert!(cx.opened_url().is_none());
    assert_eq!(cx.update(|_, cx| cx.global::<Store>().tabs().count()), 0);
}
