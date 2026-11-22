// ABOUTME: Finds the Theme Source row through the description its page displays.
// ABOUTME: Covers platform-specific read-only text through real drawing and search.
use super::*;
use crate::i18n::{
    fl,
    leak_tests::{draw, under_en_xa},
};
use gpui::{px, size};

#[test]
fn displayed_theme_source_description_finds_its_row() {
    under_en_xa("Theme Source description search", |cx, recorder| {
        let main = cx.add_window(crate::sidebar::layout_tests::fixture_window);
        let weak = cx.update(|cx| main.update(cx, |_, _, cx| cx.weak_entity()).unwrap());
        let (view, cx) = cx.add_window_view(|_, cx| SettingsWindow::new(weak, cx));
        cx.simulate_resize(size(px(1200.), px(4000.)));
        let description = cx.update(|_, cx| {
            view.update(cx, |view, _| view.section = Section::Theme);
            if themes::Scope::Herdr.editable() {
                fl!("settings-theme-source-description")
            } else {
                fl!("settings-theme-library-read-only")
            }
        });
        draw(cx);
        assert!(
            recorder.take().contains(&description),
            "the page must display the description used by this search"
        );
        cx.update(|_, cx| {
            view.update(cx, |view, cx| {
                view.set_search_query(&description, cx);
                assert!(
                    view.visible_results(cx)
                        .iter()
                        .any(|index| catalog::CATALOG[*index].id == "theme-source"),
                    "the displayed description must find the Theme Source row"
                );
            });
        });
    });
}
