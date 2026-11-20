// ABOUTME: Edits and steps Settings numbers through the real keyboard click path.
// ABOUTME: Preserves the same range and commit callbacks as pointer input.
use super::*;
use crate::settings_window::tests::press_key;

#[gpui::test]
fn number_entry_and_steppers_work_without_a_pointer(cx: &mut TestAppContext) {
    let (mut harness, seen, _, edits) = Harness::new();
    harness.look = Look::Steppers;
    harness.enabled = true;
    let (_, cx) = cx.add_window_view(|_, _| harness);
    for key in ["enter", "space", "enter"] {
        cx.update(|window, cx| {
            window.focus_next(cx);
            window.draw(cx).clear(cx);
        });
        press_key(cx, key);
    }
    assert_eq!(edits.get(), 1);
    assert_eq!(
        *seen.borrow(),
        [NumberChange::Commit(13.), NumberChange::Commit(15.)]
    );
}
