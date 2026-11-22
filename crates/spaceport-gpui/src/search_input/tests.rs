use super::*;

mod composition;
mod control_keys;
mod key_table;
mod placeholder;
mod rtl_caret;
mod word_keys;

#[test]
fn masked_offsets_count_characters_both_ways() {
    let text = "a\u{e9}\u{1f600}z";
    for (shown, (byte, _)) in text.char_indices().enumerate() {
        assert_eq!(masked_offset(text, byte), shown);
        assert_eq!(unmasked_offset(text, shown), byte);
    }
    assert_eq!(masked_offset(text, text.len()), 4);
    assert_eq!(unmasked_offset(text, 4), text.len());
    assert_eq!(unmasked_offset(text, 99), text.len());
}

#[gpui::test]
fn cmd_arrows_and_cmd_backspace_reach_the_line_ends(cx: &mut gpui::TestAppContext) {
    let (input, cx) = cx.add_window_view(|_, cx| SearchInput::new(cx));
    cx.update(|window, cx| {
        input.update(cx, |input, cx| {
            input
                .edit
                .replace(None, "one two", false, None, Newlines::Strip);
            input.edit.select_to(3, false);
            window.focus(&input.focus, cx);
        });
        window.draw(cx).clear(cx);
    });
    let edit = |cx: &mut gpui::VisualTestContext| {
        input.read_with(cx, |input, _| {
            (input.edit.text.clone(), input.edit.selection())
        })
    };
    cx.simulate_keystrokes("cmd-shift-right");
    assert_eq!(edit(cx), ("one two".into(), 3..7));
    cx.simulate_keystrokes("cmd-left");
    assert_eq!(edit(cx), ("one two".into(), 0..0));
    cx.simulate_keystrokes("cmd-right left left cmd-backspace");
    assert_eq!(edit(cx), ("wo".into(), 0..0));
    cx.simulate_keystrokes("cmd-delete");
    assert_eq!(edit(cx), (String::new(), 0..0));
}
