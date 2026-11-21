// ABOUTME: Reports work counted by actual populated Core cell painting.
// ABOUTME: Distinguishes cumulative paint work from projected content and CPU time.
use super::selected;
use crate::{core_terminals::tests::recording_link, performance::Counts, working_profile};
use gpui::TestAppContext;

#[gpui::test]
fn profile_paint_work_observes_real_cell_painting(cx: &mut TestAppContext) {
    let (fixture, cx) = selected(cx, false);
    let (link, _received) = recording_link();
    fixture.view.update(cx, |view, _| view.link = link);
    fixture.owner.update(cx, |owner, cx| {
        owner.sync_core_views(cx);
        owner.core_terminals.views.insert(
            (fixture.tab, fixture.terminal.clone()),
            fixture.view.clone(),
        );
    });
    let expected = cx.update(|window, cx| {
        window.draw(cx).clear(cx);
        *cx.try_global::<Counts>().unwrap()
    });
    assert!(expected.paints > 0 && expected.glyphs > 0, "{expected:?}");
    let observed = cx
        .update(|_, cx| working_profile::paint_work(cx))
        .unwrap_or_else(|| panic!("the profile must report actual populated cell work"));
    assert_eq!(observed.paints, expected.paints);
    assert_eq!(observed.quads, expected.quads);
    assert_eq!(observed.glyphs, expected.glyphs);
    assert_eq!(observed.text_paint_errors, expected.paint_errors);
    cx.update(|window, cx| {
        window.draw(cx).clear(cx);
        assert_eq!(working_profile::paint_work(cx), Some(observed));
    });
}
