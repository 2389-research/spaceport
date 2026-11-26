use super::*;

/// A keystroke with the modifier fields move and delete by word with.
fn word(name: &str) -> Keystroke {
    let word = if cfg!(target_os = "macos") {
        "alt"
    } else {
        "ctrl"
    };
    key(&format!("{word}-{name}"))
}

#[gpui::test]
fn word_backspace_and_delete_remove_a_word(cx: &mut gpui::TestAppContext) {
    cx.update(|cx| {
        let mut input = DialogInput::new("feature/login fix".into());
        input.key(&key("end"), cx);
        input.key(&word("backspace"), cx);
        assert_eq!(input.text, "feature/login ");
        input.key(&word("backspace"), cx);
        assert_eq!(input.text, "feature/");
        assert_eq!(input.selection(), 8..8);
        input.key(&key("home"), cx);
        input.key(&word("delete"), cx);
        assert_eq!(input.text, "/");
        assert_eq!(input.selection(), 0..0);
    });
}

#[gpui::test]
fn word_arrows_move_and_shift_extends(cx: &mut gpui::TestAppContext) {
    cx.update(|cx| {
        let mut input = DialogInput::new("one two three".into());
        input.key(&key("end"), cx);
        input.key(&word("left"), cx);
        assert_eq!(input.selection(), 8..8);
        input.key(&word("shift-left"), cx);
        assert_eq!(input.selection(), 4..8);
        input.key(&word("right"), cx);
        assert_eq!(input.selection(), 13..13);
        input.key(&word("shift-left"), cx);
        input.key(&word("backspace"), cx);
        assert_eq!(input.text, "one two ");
        assert_eq!(input.selection(), 8..8);
    });
}
