use super::*;

#[gpui::test]
fn px0_editor_reopens_in_its_group_after_failure_clears(cx: &mut gpui::TestAppContext) {
    let (view, cx) = window(cx);
    let opened = cx.update(|window, cx| {
        view.update(cx, |view, cx| {
            view.open_embedded_editor(Editor::Px0, None, None, window, cx)
        })
    });
    if !crate::browser::EMBEDDED {
        assert_eq!(opened, None);
        return;
    }
    let px0 = opened.unwrap();
    view.update(cx, |view, cx| {
        view.browser.failed.insert(px0, "px0 failed".into());
        assert!(view.editor_page_settled(px0));
        view.close_editor_pages(Editor::Px0, cx);
        assert!(!view.editor_page_settled(px0));
    });
    let reopened = cx.update(|window, cx| {
        view.update(cx, |view, cx| {
            view.open_embedded_editor(Editor::Px0, None, None, window, cx)
        })
    });
    assert_eq!(reopened, Some(px0));
    assert!(cx.update(|_, cx| view.read(cx).group_shows_page(px0, cx)));
}
