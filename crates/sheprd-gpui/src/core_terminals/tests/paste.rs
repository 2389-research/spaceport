// ABOUTME: Checks Core paste shortcuts and platform-specific text probing.
// ABOUTME: Preserves unrelated terminal key chords for their original program.
use crate::core_terminals::view::paste::{Trigger, trigger};
use gpui::Keystroke;

fn key(text: &str) -> Keystroke {
    crate::keymap::parse_keystroke(text).unwrap()
}

#[test]
fn the_window_paste_keys_paste_into_a_core_pane() {
    for text in ["cmd-v", "ctrl-shift-v", "shift-insert", "ctrl-shift-insert"] {
        assert_eq!(trigger(&key(text), false), Some(Trigger::Paste), "{text}");
    }
}

#[test]
fn plain_ctrl_v_probes_only_where_it_pastes() {
    assert_eq!(trigger(&key("ctrl-v"), false), None);
    assert_eq!(trigger(&key("ctrl-v"), true), Some(Trigger::Probe));
}

#[test]
fn other_keys_go_to_the_program() {
    for text in [
        "v",
        "shift-v",
        "alt-v",
        "ctrl-alt-v",
        "cmd-shift-insert",
        "ctrl-c",
    ] {
        assert_eq!(trigger(&key(text), true), None, "{text}");
    }
}
