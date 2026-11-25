use super::*;
use crate::parse_keystroke;

#[test]
fn special_keys_and_text_are_separate() {
    let key = |s| parse_keystroke(s).unwrap();
    for alt_keys in [false, true] {
        let code = |s| key_code(&key(s), alt_keys);
        assert_eq!(code("ctrl-c"), Some(ClientKeyCode::Char('c')));
        assert_eq!(code("shift-tab"), Some(ClientKeyCode::BackTab));
        assert_eq!(code("alt-left"), Some(ClientKeyCode::Left));
        assert_eq!(code("f12"), Some(ClientKeyCode::F(12)));
        assert_eq!(code("a"), None);
        assert_eq!(code("shift-a"), None);
        assert_eq!(code("cmd-q"), None);
        assert_eq!(code("cmd-alt-p"), None);
    }
    assert_eq!(key_code(&key("alt-e"), false), None);
    assert_eq!(key_code(&key("alt-space"), false), None);
    // A `[pane_keys]` target is checked with the same rule, Alt included.
    assert!(reaches_pane(&key("alt-b")));
    for unsendable in ["cmd-a", "a", "shift-a"] {
        assert!(!reaches_pane(&key(unsendable)), "{unsendable}");
    }
}
