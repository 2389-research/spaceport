use super::*;

#[gpui::test]
fn page_toolbar_has_no_annotation_action(cx: &mut gpui::TestAppContext) {
    let (view, cx) = window(cx);
    cx.update(|window, cx| {
        view.update(cx, |view, cx| view.open_browser_tab(None, window, cx));
    });
    draw(cx);
    assert!(cx.debug_bounds("browser-address").is_some());
    assert!(cx.debug_bounds("browser-annotate").is_none());
}

#[gpui::test]
fn page_send_message_cannot_queue_agent_feedback(cx: &mut gpui::TestAppContext) {
    let (view, cx) = window(cx);
    cx.update(|window, cx| {
        view.update(cx, |view, cx| {
            view.open_browser_tab(None, window, cx);
            view.apply_page_post(
                crate::browser::TabId::test(0),
                r#"{"kind":"send","summary":"Picked variant B","data":{"variant":"B"}}"#,
                window,
                cx,
            );
        });
    });
    view.read_with(cx, |view, _| {
        assert!(view.tab_favicon(crate::browser::TabId::test(0)).is_none());
    });
}
