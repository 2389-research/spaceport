use super::*;

/// Old workspace metadata cannot create a page when a window ticks.
#[gpui::test]
fn stale_remote_page_tokens_do_not_open_a_tab(cx: &mut gpui::TestAppContext) {
    let (view, cx) = cx.add_window_view(fixture_window);
    view.update(cx, |view, _| {
        let mut shown = snapshot(2);
        shown.workspaces[1].tokens = vec![
            ("herdr-gpui-9-1".into(), "open w1:p3".into()),
            ("herdr-gpui-9-1_0".into(), "https://example.com/".into()),
        ];
        view.endpoints[0].never_connect();
        view.endpoints[0]
            .connection
            .inbox
            .lock()
            .unwrap()
            .apply(sheprd_client::ClientEvent::Snapshot(Arc::new(shown)));
    });
    cx.update(|window, cx| view.update(cx, |view, cx| view.tick(window, cx)));
    assert!(cx.update(|_, cx| cx.try_global::<crate::browser::Store>().is_none()));
}
