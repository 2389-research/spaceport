//! Shift-Enter reaches a pane as Alt-Enter, so a program at Herdr's legacy
//! keyboard level reads a newline chord instead of a bare CR (#504).

use super::*;

#[test]
fn shift_enter_default_can_be_removed_or_replaced() {
    let resolve = |entries: &[(&str, &str)]| {
        Keymap::with_overrides(&BTreeMap::new(), &pane_keys(entries), &no_keys()).unwrap()
    };
    assert_eq!(sent(&resolve(&[("shift-enter", "")]), "shift-enter"), None);
    assert_eq!(
        sent(&resolve(&[("shift-enter", "ctrl-j")]), "shift-enter"),
        Some(&key("ctrl-j"))
    );
    // Shift alone does not make a command shortcut, so no command can
    // already own it.
    assert!(matches!(
        Keymap::with_overrides(
            &overrides(&[("new_tab", one("shift-enter"))]),
            &BTreeMap::new(),
            &no_keys()
        ),
        Err(Error::KeystrokeWithoutModifier { .. })
    ));
}
