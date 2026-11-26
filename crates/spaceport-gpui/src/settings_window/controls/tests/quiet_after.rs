// ABOUTME: Chooses each Quiet threshold in the Sidebar settings page.
// ABOUTME: Checks that each segmented control submits the corresponding preference.
use super::*;
use crate::config::preferences::Preference;
use crate::settings_window::controls::preferences::PreferenceIo;
use spaceport_settings::QuietAfter;
use std::sync::{Arc, Mutex};

#[gpui::test]
fn sidebar_quiet_threshold_saves_each_choice(cx: &mut TestAppContext) {
    let (view, cx) = cx.add_window_view(skill_fixture);
    let edits = Arc::new(Mutex::new(Vec::new()));
    let captured = edits.clone();
    view.update(cx, |view, _| {
        view.controls.preference_io = Some(PreferenceIo {
            write: Arc::new(move |edit| {
                captured.lock().unwrap().push(edit);
                Ok(())
            }),
            load: Arc::new(skill_load),
        });
    });
    cx.simulate_resize(size(px(960.), px(2200.)));
    for (selector, before, after) in [
        (
            "sidebar-quiet-after-control-1",
            QuietAfter::OneDay,
            QuietAfter::TwoDays,
        ),
        (
            "sidebar-quiet-after-control-2",
            QuietAfter::TwoDays,
            QuietAfter::OneWeek,
        ),
        (
            "sidebar-quiet-after-control-0",
            QuietAfter::OneWeek,
            QuietAfter::OneDay,
        ),
    ] {
        view.update(cx, |view, cx| {
            view.section = Section::Sidebar;
            view.config.quiet_after = before;
            cx.notify();
        });
        cx.update(|window, cx| crate::sidebar::layout_tests::full_draw(window, cx).clear(cx));
        let option = cx.debug_bounds(selector).unwrap();
        cx.simulate_click(option.center(), Default::default());
        cx.run_until_parked();
        assert_eq!(
            edits.lock().unwrap().pop(),
            Some(Preference::QuietAfter(after))
        );
    }
}
