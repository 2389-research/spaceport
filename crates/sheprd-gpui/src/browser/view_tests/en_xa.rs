//! The `en-XA` leak test (`crate::i18n::leak_tests`) for browser tabs: a
//! blank tab with its toolbar and placeholder.
use super::*;
use crate::i18n::leak_tests::{assert_translated, draw, uncut, under_en_xa};

#[test]
fn browser_tabs_show_only_messages_under_en_xa() {
    under_en_xa("browser en-XA leak", |cx, recorder| {
        let (view, cx) = window(cx);
        draw(cx);
        recorder.take();

        cx.update(|window, cx| view.update(cx, |view, cx| view.open_browser_tab(None, window, cx)));
        draw(cx);
        assert_translated("blank browser tab", &uncut(recorder.take()), &[]);
    });
}
