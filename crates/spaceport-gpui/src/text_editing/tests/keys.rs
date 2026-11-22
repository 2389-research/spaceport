//! The key table itself, on every desktop's conventions: the widgets' own
//! `key_table` tests only run the desktop they are built for.

use super::*;
use gpui::Modifiers;

const PLATFORMS: [Platform; 3] = [Platform::MacOs, Platform::CtrlDesktop, Platform::Other];

/// `one two` with the caret after `one`.
fn editing() -> Editing {
    Editing {
        text: "one two".into(),
        anchor: 3,
        cursor: 3,
        marked: None,
    }
}

fn keys(table: KeyTable, platform: Platform) -> Keys {
    Keys {
        line: 0..7,
        words_hidden: false,
        table,
        platform,
    }
}

/// The command for a keystroke written as the keymap writes it.
fn command(keystroke: &'static str, table: KeyTable, platform: Platform) -> Option<EditCommand> {
    let keystroke = gpui::Keystroke::parse(keystroke).unwrap_or_default();
    edit_command(
        &keystroke.key,
        keystroke.modifiers,
        &editing(),
        &keys(table, platform),
    )
}

fn to(to: usize) -> Option<EditCommand> {
    Some(EditCommand::Move { to, extend: false })
}

#[test]
fn control_is_emacs_on_macos_and_the_edit_and_word_key_on_the_ctrl_desktops() {
    use EditCommand::{Copy, Cut, Delete, Kill, Paste, SelectAll, Yank};
    let field = |keystroke, platform| command(keystroke, KeyTable::Field, platform);
    for (keystroke, macos, ctrl, other) in [
        ("ctrl-a", to(0), Some(SelectAll), None),
        ("ctrl-e", to(7), None, None),
        ("ctrl-b", to(2), None, None),
        ("ctrl-f", to(4), None, None),
        ("ctrl-h", Some(Delete(2..3)), None, None),
        ("ctrl-d", Some(Delete(3..4)), None, None),
        ("ctrl-k", Some(Kill(3..7)), None, None),
        ("ctrl-y", Some(Yank), None, None),
        ("ctrl-c", None, Some(Copy), None),
        ("ctrl-x", None, Some(Cut), None),
        ("ctrl-v", None, Some(Paste), None),
        // The terminal's Ctrl-Shift aliases edit a field too.
        ("ctrl-shift-c", None, Some(Copy), None),
        ("ctrl-shift-a", None, Some(SelectAll), None),
        ("ctrl-left", None, to(0), to(0)),
        ("ctrl-right", None, to(7), to(7)),
        (
            "ctrl-backspace",
            None,
            Some(Delete(0..3)),
            Some(Delete(0..3)),
        ),
        ("ctrl-delete", None, Some(Delete(3..7)), Some(Delete(3..7))),
        ("alt-left", to(0), None, None),
        ("alt-right", to(7), None, None),
        ("alt-backspace", Some(Delete(0..3)), None, None),
        ("alt-delete", Some(Delete(3..7)), None, None),
    ] {
        assert_eq!(
            field(keystroke, Platform::MacOs),
            macos,
            "{keystroke} on macOS"
        );
        assert_eq!(
            field(keystroke, Platform::CtrlDesktop),
            ctrl,
            "{keystroke} with Ctrl keys"
        );
        assert_eq!(
            field(keystroke, Platform::Other),
            other,
            "{keystroke} elsewhere"
        );
    }
}

#[test]
fn the_platform_key_and_the_plain_keys_mean_the_same_on_every_desktop() {
    use EditCommand::{Copy, Cut, Delete, Move, Paste, SelectAll};
    for platform in PLATFORMS {
        for (keystroke, expected) in [
            ("cmd-a", Some(SelectAll)),
            ("cmd-c", Some(Copy)),
            ("cmd-x", Some(Cut)),
            ("cmd-v", Some(Paste)),
            ("cmd-shift-a", None),
            ("cmd-left", to(0)),
            ("cmd-right", to(7)),
            (
                "cmd-shift-right",
                Some(Move {
                    to: 7,
                    extend: true,
                }),
            ),
            ("cmd-backspace", Some(Delete(0..3))),
            ("cmd-delete", Some(Delete(3..7))),
            ("cmd-shift-backspace", None),
            ("cmd-home", None),
            ("left", to(2)),
            ("right", to(4)),
            (
                "shift-left",
                Some(Move {
                    to: 2,
                    extend: true,
                }),
            ),
            ("home", to(0)),
            ("end", to(7)),
            ("backspace", Some(Delete(2..3))),
            ("shift-delete", Some(Delete(3..4))),
            ("up", None),
            ("down", None),
            ("enter", None),
            ("escape", None),
            ("a", None),
        ] {
            assert_eq!(
                command(keystroke, KeyTable::Field, platform),
                expected,
                "{keystroke} on {platform:?}"
            );
        }
    }
}

#[test]
fn a_line_bounds_home_end_the_platform_key_and_the_kill() {
    use EditCommand::{Delete, Kill};
    // The caret is on the second line of `ab\ncd ef\ngh`.
    let edit = Editing {
        text: "ab\ncd ef\ngh".into(),
        anchor: 5,
        cursor: 5,
        marked: None,
    };
    let keys = Keys {
        line: 3..8,
        ..keys(KeyTable::Field, Platform::MacOs)
    };
    let command = |keystroke: &'static str| {
        let keystroke = gpui::Keystroke::parse(keystroke).unwrap_or_default();
        edit_command(&keystroke.key, keystroke.modifiers, &edit, &keys)
    };
    assert_eq!(command("home"), to(3));
    assert_eq!(command("end"), to(8));
    assert_eq!(command("cmd-left"), to(3));
    assert_eq!(command("cmd-right"), to(8));
    assert_eq!(command("ctrl-a"), to(3));
    assert_eq!(command("ctrl-e"), to(8));
    assert_eq!(command("cmd-backspace"), Some(Delete(3..5)));
    assert_eq!(command("cmd-delete"), Some(Delete(5..8)));
    assert_eq!(command("ctrl-k"), Some(Kill(5..8)));
    // Words and Select All are the whole text's.
    assert_eq!(command("alt-left"), to(3));
    assert_eq!(command("alt-backspace"), Some(Delete(3..5)));
    assert_eq!(command("cmd-a"), Some(EditCommand::SelectAll));
}

#[test]
fn hidden_words_make_the_text_one_word() {
    let hidden = Keys {
        words_hidden: true,
        ..keys(KeyTable::Field, Platform::MacOs)
    };
    let command = |keystroke: &'static str| {
        let keystroke = gpui::Keystroke::parse(keystroke).unwrap_or_default();
        edit_command(&keystroke.key, keystroke.modifiers, &editing(), &hidden)
    };
    assert_eq!(command("alt-left"), to(0));
    assert_eq!(command("alt-right"), to(7));
    assert_eq!(command("alt-backspace"), Some(EditCommand::Delete(0..3)));
    assert_eq!(command("alt-delete"), Some(EditCommand::Delete(3..7)));
}

#[test]
fn a_dialog_has_no_emacs_keys_and_takes_its_arrows_under_any_modifiers() {
    use EditCommand::{Delete, SelectAll};
    let dialog = |keystroke, platform| command(keystroke, KeyTable::Dialog, platform);
    for keystroke in [
        "ctrl-e", "ctrl-b", "ctrl-f", "ctrl-h", "ctrl-d", "ctrl-k", "ctrl-y",
    ] {
        for platform in PLATFORMS {
            assert_eq!(
                dialog(keystroke, platform),
                None,
                "{keystroke} on {platform:?}"
            );
        }
    }
    assert_eq!(dialog("ctrl-a", Platform::MacOs), None);
    assert_eq!(dialog("ctrl-a", Platform::CtrlDesktop), Some(SelectAll));
    // Control is no word key on macOS, so the arrow moves a character.
    assert_eq!(dialog("ctrl-left", Platform::MacOs), to(2));
    assert_eq!(dialog("ctrl-left", Platform::CtrlDesktop), to(0));
    assert_eq!(
        dialog("ctrl-backspace", Platform::MacOs),
        Some(Delete(2..3))
    );
    for platform in PLATFORMS {
        assert_eq!(dialog("cmd-home", platform), to(0));
        assert_eq!(dialog("ctrl-alt-end", platform), to(7));
        assert_eq!(dialog("alt-cmd-left", platform), to(0));
        assert_eq!(dialog("cmd-shift-backspace", platform), Some(Delete(0..3)));
        assert_eq!(dialog("alt-cmd-delete", platform), Some(Delete(3..7)));
        assert_eq!(dialog("cmd-shift-a", platform), None);
        assert_eq!(dialog("up", platform), None);
        assert_eq!(dialog("escape", platform), None);
    }
}

/// A draft has no kill buffer, so its table must never ask for one.
#[test]
fn a_dialog_never_kills_or_yanks() {
    let keys_pressed = [
        "a",
        "b",
        "c",
        "d",
        "e",
        "f",
        "h",
        "k",
        "v",
        "x",
        "y",
        "left",
        "right",
        "home",
        "end",
        "up",
        "down",
        "backspace",
        "delete",
        "enter",
        "escape",
        "tab",
    ];
    for platform in PLATFORMS {
        for bits in 0u8..32 {
            let modifiers = Modifiers {
                control: bits & 1 != 0,
                alt: bits & 2 != 0,
                shift: bits & 4 != 0,
                platform: bits & 8 != 0,
                function: bits & 16 != 0,
            };
            for key in keys_pressed {
                let command = edit_command(
                    key,
                    modifiers,
                    &editing(),
                    &keys(KeyTable::Dialog, platform),
                );
                assert!(
                    !matches!(command, Some(EditCommand::Kill(_) | EditCommand::Yank)),
                    "{key} with {modifiers:?} on {platform:?}"
                );
            }
        }
    }
}
