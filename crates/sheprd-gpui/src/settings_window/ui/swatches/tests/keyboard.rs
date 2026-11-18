// ABOUTME: Activates actual Settings colour controls with the keyboard.
// ABOUTME: Keeps the same colour change callbacks as pointer activation.
use super::*;
use crate::settings_window::tests::press_key;

#[gpui::test]
fn swatches_can_be_cleared_and_selected_without_a_pointer(cx: &mut TestAppContext) {
    let seen = Rc::new(RefCell::new(Vec::new()));
    let (_, cx) = cx.add_window_view(|_, _| Harness {
        seen: seen.clone(),
        current: Some(SWATCHES[0]),
        host: "Fixture",
        width: None,
    });
    for key in ["space", "enter"] {
        cx.update(|window, cx| {
            window.focus_next(cx);
            window.draw(cx).clear(cx);
        });
        press_key(cx, key);
    }
    assert_eq!(
        *seen.borrow(),
        [HostColorChange::Clear, HostColorChange::Set(SWATCHES[0])]
    );
}
