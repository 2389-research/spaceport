// ABOUTME: Navigates custom Settings choices through real window key events.
// ABOUTME: Captures typed persistence operations without touching personal settings.
use super::*;
use crate::config::{StatusIcons, backdrop::Edit, preferences::Preference};
use crate::settings_window::controls::preferences::PreferenceIo;

fn draw(cx: &mut VisualTestContext) {
    cx.update(|window, cx| crate::sidebar::layout_tests::full_draw(window, cx).clear(cx));
}

fn focus_row(cx: &mut VisualTestContext, id: &str) {
    for _ in 0..128 {
        press_key(cx, "tab");
        draw(cx);
        if cx.update(|window, _| {
            window.context_stack().iter().any(|context| {
                context
                    .get("settings-row")
                    .is_some_and(|row| row.as_ref() == id)
            })
        }) {
            return;
        }
    }
    panic!("Tab never reached {id}");
}

#[gpui::test]
fn search_results_can_be_chosen_after_leaving_the_search_field(cx: &mut TestAppContext) {
    let (view, cx) = cx.add_window_view(skill_fixture);
    let expected = cx.update(|window, cx| {
        view.update(cx, |view, cx| {
            view.set_search_query("font", cx);
            window.focus(&view.focus, cx);
            catalog::CATALOG[view.visible_results(cx)[1]].section
        })
    });
    draw(cx);
    focus_row(cx, "settings-search-result-1");
    press_key(cx, "enter");
    assert_eq!(view.read_with(cx, |view, _| view.section), expected);
    assert!(view.read_with(cx, |view, _| view.search_results.is_none()));
}

#[gpui::test]
fn indicator_cards_use_their_existing_save_callback_from_the_keyboard(cx: &mut TestAppContext) {
    let (view, cx) = cx.add_window_view(skill_fixture);
    let edits = Arc::new(Mutex::new(Vec::new()));
    let written = edits.clone();
    cx.update(|window, cx| {
        view.update(cx, |view, cx| {
            view.section = Section::Indicators;
            view.controls.preference_io = Some(PreferenceIo::recording(move |edit| {
                written.lock().unwrap().push(edit);
            }));
            window.focus(&view.focus, cx);
        });
    });
    draw(cx);
    focus_row(cx, "indicator-set-solid");
    press_key(cx, "space");
    cx.run_until_parked();
    assert_eq!(
        *edits.lock().unwrap(),
        [Preference::StatusIcons(StatusIcons::Solid)]
    );
}

#[gpui::test]
fn font_picker_opens_and_its_close_button_works_from_the_keyboard(cx: &mut TestAppContext) {
    let (view, cx) = cx.add_window_view(skill_fixture);
    cx.update(|window, cx| {
        view.update(cx, |view, cx| {
            view.section = Section::Fonts;
            window.focus(&view.focus, cx);
        });
    });
    draw(cx);
    focus_row(cx, "fonts-all-row");
    press_key(cx, "enter");
    draw(cx);
    assert!(view.read_with(cx, |view, _| view.font_picker_shown()));
    // The close button precedes the search field inside the modal.
    press_key(cx, "shift-tab");
    draw(cx);
    press_key(cx, "enter");
    assert!(!view.read_with(cx, |view, _| view.font_picker_shown()));
}

#[gpui::test]
fn backdrop_tiles_and_ruler_are_keyboard_controls(cx: &mut TestAppContext) {
    let (view, cx) = cx.add_window_view(skill_fixture);
    let edits = Arc::new(Mutex::new(Vec::new()));
    let written = edits.clone();
    cx.update(|window, cx| {
        view.update(cx, |view, cx| {
            view.section = Section::Theme;
            view.controls.backdrop_io = Some(controls::backdrop::BackdropIo {
                write: Arc::new(move |edits| {
                    written.lock().unwrap().extend(edits);
                    Ok(())
                }),
                load: Arc::new(|| {
                    let mut loaded = fixture();
                    loaded.config.backdrop.opacity = 0.85;
                    Ok(loaded)
                }),
            });
            window.focus(&view.focus, cx);
        });
    });
    draw(cx);
    focus_row(cx, "settings-backdrop-translucent");
    press_key(cx, "enter");
    cx.run_until_parked();
    assert_eq!(
        std::mem::take(&mut *edits.lock().unwrap()),
        [Edit::Opacity(0.85)]
    );
    draw(cx);
    focus_row(cx, "settings-backdrop-ruler");
    // Modifier chords belong to the window, not this slider.
    press_key(cx, "ctrl-left");
    assert!(edits.lock().unwrap().is_empty());
    press_key(cx, "left");
    cx.run_until_parked();
    assert_eq!(
        std::mem::take(&mut *edits.lock().unwrap()),
        [Edit::Opacity(0.84)]
    );
    draw(cx);
    press_key(cx, "home");
    cx.run_until_parked();
    assert_eq!(
        std::mem::take(&mut *edits.lock().unwrap()),
        [Edit::Opacity(0.1)]
    );
    draw(cx);
    press_key(cx, "end");
    cx.run_until_parked();
    assert_eq!(
        std::mem::take(&mut *edits.lock().unwrap()),
        [Edit::Opacity(1.)]
    );
}
