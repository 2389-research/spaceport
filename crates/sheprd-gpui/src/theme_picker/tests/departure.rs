// ABOUTME: Provides a built-in-only theme picker fixture for departure tests.
// ABOUTME: Suppresses personal theme discovery while exercising the actual opening path.
use super::*;
use core::prelude::v1::test;

pub(crate) fn fixture(window: &mut Window, cx: &mut Context<SheprdWindow>) -> SheprdWindow {
    let mut view = crate::sidebar::layout_tests::fixture_window(window, cx);
    view.menu.themes = Some(ThemePicker {
        input: None,
        names: Vec::new(),
        filtered: Vec::new(),
        favorites: 0,
        favorites_saving: false,
        favorite_edits: Vec::new(),
        favorites_error: None,
        selected: 0,
        scroll: UniformListScrollHandle::new(),
        error: None,
        query: String::new(),
        discovering: true,
        baseline: None,
        session: 0,
        request: 0,
        desired: None,
        loaded: None,
        accepting: false,
        saving: false,
        in_flight: None,
        window: window.window_handle(),
        status_scroll: ScrollHandle::new(),
    });
    view
}

struct Preview {
    source: Entity<SheprdWindow>,
    paint: Paint,
}

impl Render for Preview {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .w(px(500.))
            .h(px(560.))
            .child(self.source.update(cx, |view, cx| {
                view.render_theme_picker_paint(&self.paint, cx)
            }))
    }
}

#[gpui::test]
fn frozen_theme_controls_cannot_change_a_reopened_picker(cx: &mut TestAppContext) {
    let (preview, cx) = cx.add_window_view(|window, cx| {
        let source = cx.new(|cx| fixture(window, cx));
        let paint = source.update(cx, |view, cx| {
            view.open_theme_picker(window, cx);
            view.menu.themes.as_mut().unwrap().selected = 0;
            let paint = view.theme_picker_paint(cx).unwrap();
            view.dismiss_menu(window, cx);
            view.open_theme_picker(window, cx);
            let picker = view.menu.themes.as_mut().unwrap();
            picker.selected = 1;
            // A broken outgoing star must remain inside this private in-memory fixture.
            picker.favorites_saving = true;
            paint
        });
        Preview { source, paint }
    });
    cx.update(|window, cx| window.draw(cx).clear(cx));
    let row = cx.debug_bounds("theme-row-0").unwrap();
    cx.simulate_mouse_move(row.center(), None, Modifiers::default());
    let star = cx.debug_bounds("theme-star-0").unwrap();
    cx.simulate_click(star.center(), Modifiers::default());
    let close = cx.debug_bounds("theme-close").unwrap();
    cx.simulate_click(close.center(), Modifiers::default());
    preview.read_with(cx, |preview, cx| {
        let view = preview.source.read(cx);
        let picker = view.menu.themes.as_ref().unwrap();
        assert_eq!(view.menu.page, Some(Page::Themes));
        assert_eq!(picker.selected, 1);
        assert!(picker.favorite_edits.is_empty());
        assert!(view.config.favorite_themes.is_empty());
    });
}

#[gpui::test]
fn old_search_events_do_not_replace_the_reopened_query(cx: &mut TestAppContext) {
    let (view, cx) = cx.add_window_view(fixture);
    let old = cx.update(|window, cx| {
        view.update(cx, |view, cx| {
            view.open_theme_picker(window, cx);
            let old = view.menu.themes.as_ref().unwrap().search().unwrap().clone();
            old.update(cx, |input, cx| input.set_text_selected("Nord", cx));
            view.dismiss_menu(window, cx);
            view.open_theme_picker(window, cx);
            old
        })
    });
    cx.run_until_parked();
    old.update(cx, |input, cx| input.set_text_selected("Dracula", cx));
    cx.run_until_parked();
    view.read_with(cx, |view, cx| {
        let picker = view.menu.themes.as_ref().unwrap();
        assert!(picker.query.is_empty());
        assert!(picker.search().unwrap().read(cx).text().is_empty());
        assert_eq!(view.config.theme, "Default");
        assert_eq!(view.theme, Theme::default());
    });
}

#[gpui::test]
fn accepted_favorite_write_finishes_after_the_input_owner_drops(cx: &mut TestAppContext) {
    let private = tempfile::tempdir().unwrap();
    let file = private.path().join("favorites.json");
    std::fs::write(&file, br#"["Solarized Dark"]"#).unwrap();
    let destination = file.clone();
    let (view, cx) = cx.add_window_view(fixture);
    let input = cx.update(|window, cx| {
        view.update(cx, |view, cx| {
            view.open_theme_picker(window, cx);
            let input = view
                .menu
                .themes
                .as_ref()
                .unwrap()
                .search()
                .unwrap()
                .downgrade();
            assert!(view.toggle_favorite_theme("Nord"));
            view.save_favorite_themes_with(
                move |edits| {
                    let mut names: Vec<String> =
                        serde_json::from_slice(&std::fs::read(&destination)?)?;
                    crate::config::favorite_themes::apply(&mut names, &edits);
                    std::fs::write(&destination, serde_json::to_vec(&names)?)?;
                    Ok(names)
                },
                cx,
            );
            view.dismiss_menu(window, cx);
            assert!(view.menu.page.is_none());
            input
        })
    });
    cx.run_until_parked();
    cx.update(|window, cx| window.draw(cx).clear(cx));
    assert!(input.upgrade().is_none());
    let saved: Vec<String> = serde_json::from_slice(&std::fs::read(file).unwrap()).unwrap();
    assert_eq!(saved, ["Solarized Dark", "Nord"]);
    view.read_with(cx, |view, _| {
        let picker = view.menu.themes.as_ref().unwrap();
        assert!(picker.search().is_none());
        assert!(!picker.favorites_saving);
        assert!(picker.favorite_edits.is_empty());
        assert_eq!(view.config.favorite_themes, saved);
    });
}
